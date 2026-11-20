//! Hosting a third-party UCI engine: the handshake, the option exchange, a search that
//! can be stopped, and the bookkeeping a session needs — what the engine declared, what
//! we asked it for, where the log is, and whether it is still alive.
//!
//! One [`Engine`] is one child process with one reader thread. Nothing here is async and
//! nothing is shared between engines: cancellation has to be independent per instance,
//! and a thread that blocks on its own engine's stdout is the simplest thing that
//! achieves that. Commands are written by the caller's thread, so a `stop` never waits
//! for anything the engine is doing.
//!
//! This crate speaks text and does not know chess. A principal variation comes back as
//! the engine's own long algebraic strings; turning them into moves belongs to whoever
//! holds the position they were searched from.

pub mod discover;
pub mod protocol;

pub use protocol::{Event, Info, InfoScore, OptionDecl};

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How long an engine may take to answer a handshake step. Engines loading a large
/// network are the reason this is not a second.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(20);

/// How long a health probe waits for its answer. Short on purpose: a probe runs while the
/// player is looking at the app. An engine that is loading a large network is not unwell,
/// and a caller that means to wait says so with [`Engine::is_ready_within`].
pub const PROBE_PATIENCE: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub enum HostError {
    /// The process could not be started or the pipe broke.
    Io(String),
    /// The engine did not answer in time but is still running.
    Unresponsive,
    /// The engine is gone — it crashed, or it was never there.
    Exited,
    /// We asked for something the engine does not offer, or outside the range it
    /// declared. Caught here, so the engine never sees it.
    BadOption { name: String, reason: String },
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostError::Io(why) => write!(f, "engine process failed: {why}"),
            HostError::Unresponsive => write!(f, "engine did not answer in time"),
            HostError::Exited => write!(f, "engine is not running"),
            HostError::BadOption { name, reason } => write!(f, "option {name}: {reason}"),
        }
    }
}

impl std::error::Error for HostError {}

impl From<io::Error> for HostError {
    fn from(why: io::Error) -> Self {
        HostError::Io(why.to_string())
    }
}

/// Who the engine says it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineId {
    pub name: String,
    pub author: Option<String>,
}

/// How long the engine may think. Every field is optional; UCI's own rule is that the
/// engine stops at the first limit it hits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Limits {
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub movetime: Option<u64>,
    pub white_ms: Option<u64>,
    pub black_ms: Option<u64>,
    /// Keep searching until told to stop.
    pub infinite: bool,
}

impl Limits {
    fn to_command(self) -> String {
        let mut command = String::from("go");
        if let Some(n) = self.depth {
            command.push_str(&format!(" depth {n}"));
        }
        if let Some(n) = self.nodes {
            command.push_str(&format!(" nodes {n}"));
        }
        if let Some(n) = self.movetime {
            command.push_str(&format!(" movetime {n}"));
        }
        if let Some(n) = self.white_ms {
            command.push_str(&format!(" wtime {n}"));
        }
        if let Some(n) = self.black_ms {
            command.push_str(&format!(" btime {n}"));
        }
        if self.infinite {
            command.push_str(" infinite");
        }
        command
    }
}

/// Where the search starts from.
#[derive(Debug, Clone, Copy)]
pub enum Position<'a> {
    StartPos { moves: &'a [String] },
    Fen { fen: &'a str, moves: &'a [String] },
}

impl Position<'_> {
    fn to_command(self) -> String {
        let (head, moves) = match self {
            Position::StartPos { moves } => ("position startpos".to_string(), moves),
            Position::Fen { fen, moves } => (format!("position fen {fen}"), moves),
        };
        if moves.is_empty() {
            head
        } else {
            format!("{head} moves {}", moves.join(" "))
        }
    }
}

/// What one search produced: every `info` line the engine sent, and its move.
#[derive(Debug, Clone, Default)]
pub struct SearchOutcome {
    pub infos: Vec<Info>,
    pub bestmove: Option<String>,
    pub ponder: Option<String>,
}

/// Whether the engine is fit to be offered to the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Health {
    Healthy,
    /// Running, but it did not answer a readiness check.
    NotResponding,
    /// Not running.
    Down(String),
}

/// A live engine session. Dropping it takes the process with it.
struct Session {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Event>,
    /// Lines we sent. The reader thread holds its own handle for what came back.
    log: Option<File>,
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        // The reader thread is deliberately not joined. A killed engine can leave a child
        // of its own holding the pipe open, and waiting for that would block whoever
        // dropped the session — the thread ends on its own once the pipe closes or the
        // receiver is gone, whichever happens first.
    }
}

/// One hosted engine.
pub struct Engine {
    program: PathBuf,
    args: Vec<String>,
    log_path: Option<PathBuf>,
    session: Option<Session>,
    /// What the engine declared during the handshake, in the order it declared it.
    declared: Vec<OptionDecl>,
    /// What we asked for, in order, replayed verbatim after a restart.
    applied: Vec<(String, String)>,
    /// The last position command, replayed after a restart.
    position: Option<String>,
    /// The last search asked for, so a crash can be recovered by asking again.
    limits: Option<Limits>,
    /// Automatic recoveries spent on the current search. One, then the caller is told.
    recoveries: u8,
    dead: bool,
}

fn open_log(path: Option<&Path>) -> io::Result<Option<File>> {
    match path {
        None => Ok(None),
        Some(path) => Ok(Some(
            OpenOptions::new().create(true).append(true).open(path)?,
        )),
    }
}

fn log_line(log: &mut Option<File>, line: &str) {
    if let Some(file) = log {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

fn spawn_reader(
    stdout: std::process::ChildStdout,
    tx: Sender<Event>,
    mut log: Option<File>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut lines = BufReader::new(stdout).lines();
        loop {
            match lines.next() {
                Some(Ok(line)) => {
                    log_line(&mut log, &format!("< {line}"));
                    let event = protocol::parse_line(&line);
                    let is_exit = matches!(event, Event::Exited);
                    if tx.send(event).is_err() || is_exit {
                        return;
                    }
                }
                // The pipe closed or the line was not text: either way the engine is
                // done with us.
                _ => {
                    let _ = tx.send(Event::Exited);
                    return;
                }
            }
        }
    })
}

/// Start one engine process and the thread that reads it. The first spawn and every
/// restart go through here, so a restarted engine is started exactly like a new one.
fn start(program: &Path, args: &[String], log_path: Option<&Path>) -> Result<Session, HostError> {
    let mut log = open_log(log_path)?;
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|why| HostError::Io(format!("{}: {why}", program.display())))?;
    let stdin = child.stdin.take().expect("stdin is piped");
    let stdout = child.stdout.take().expect("stdout is piped");
    let (tx, rx) = mpsc::channel();
    let _ = spawn_reader(stdout, tx, log.as_ref().and_then(|f| f.try_clone().ok()));
    log_line(&mut log, &format!("! started {}", program.display()));
    Ok(Session {
        child,
        stdin,
        rx,
        log,
    })
}

impl Engine {
    /// Start the engine without speaking to it. The handshake is a separate step so a
    /// health check can report a failure of the handshake itself.
    pub fn spawn(
        program: impl AsRef<Path>,
        args: &[&str],
        log_path: Option<impl AsRef<Path>>,
    ) -> Result<Engine, HostError> {
        let program = program.as_ref().to_path_buf();
        let log_path = log_path.map(|p| p.as_ref().to_path_buf());
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let session = start(&program, &args, log_path.as_deref())?;
        Ok(Engine {
            program,
            args,
            log_path,
            session: Some(session),
            declared: Vec::new(),
            applied: Vec::new(),
            position: None,
            limits: None,
            recoveries: 0,
            dead: false,
        })
    }

    fn session(&mut self) -> Result<&mut Session, HostError> {
        if self.dead {
            return Err(HostError::Exited);
        }
        self.session
            .as_mut()
            .ok_or(HostError::Io("engine is not running".into()))
    }

    fn send(&mut self, command: &str) -> Result<(), HostError> {
        let written = self.session().and_then(|session| {
            log_line(&mut session.log, &format!("> {command}"));
            writeln!(session.stdin, "{command}")
                .and_then(|()| session.stdin.flush())
                // A broken pipe means the engine stopped listening.
                .map_err(|_| HostError::Exited)
        });
        if written.is_err() {
            self.dead = true;
        }
        written
    }

    /// Take the next event, giving up after `timeout` when one is given.
    fn next_event(&mut self, timeout: Option<Duration>) -> Result<Event, HostError> {
        let session = self.session()?;
        let event = match timeout {
            Some(limit) => session.rx.recv_timeout(limit).map_err(|_| {
                if session.child.try_wait().ok().flatten().is_some() {
                    HostError::Exited
                } else {
                    HostError::Unresponsive
                }
            })?,
            None => session.rx.recv().map_err(|_| HostError::Exited)?,
        };
        if matches!(event, Event::Exited) {
            self.dead = true;
        }
        Ok(event)
    }

    /// `uci` and `uciok`, collecting what the engine declares about itself and its
    /// options. The engine's name and author are returned for the session to show.
    pub fn handshake(&mut self) -> Result<EngineId, HostError> {
        self.handshake_within(HANDSHAKE_TIMEOUT)
    }

    /// The same, for a caller that means to wait less than a handshake is normally given.
    /// Discovery uses this: a candidate that has not said `uciok` in a few seconds is not
    /// worth the app's first run.
    pub fn handshake_within(&mut self, patience: Duration) -> Result<EngineId, HostError> {
        self.send("uci")?;
        let mut id = EngineId {
            name: String::new(),
            author: None,
        };
        loop {
            match self.next_event(Some(patience))? {
                Event::UciOk => return Ok(id),
                Event::IdName(name) => id.name = name,
                Event::IdAuthor(author) => id.author = Some(author),
                Event::Option(decl) => self.declared.push(decl),
                // Anything else before uciok is a banner, an early readyok, or an engine
                // that likes to talk. Logged, and none of our business.
                _ => {}
            }
        }
    }

    /// `isready` and `readyok`, waiting as long as a handshake may.
    pub fn is_ready(&mut self) -> Result<(), HostError> {
        self.is_ready_within(HANDSHAKE_TIMEOUT)
    }

    /// The engine answers readiness only once it has finished whatever it was setting up,
    /// so this is how a caller synchronises — and how long it is prepared to wait.
    pub fn is_ready_within(&mut self, patience: Duration) -> Result<(), HostError> {
        self.send("isready")?;
        loop {
            match self.next_event(Some(patience))? {
                Event::Ready => return Ok(()),
                Event::Line(_) | Event::Info(_) => {}
                _ => return Err(HostError::Exited),
            }
        }
    }

    /// Tell the engine to forget its previous game.
    pub fn new_game(&mut self) -> Result<(), HostError> {
        self.send("ucinewgame")
    }

    pub fn declared(&self) -> &[OptionDecl] {
        &self.declared
    }

    /// Set an option the engine declared, refusing anything it did not.
    pub fn set_option(&mut self, name: &str, value: &str) -> Result<(), HostError> {
        let decl = self
            .declared
            .iter()
            .find(|decl| decl.name() == name)
            .ok_or_else(|| HostError::BadOption {
                name: name.to_string(),
                reason: "the engine does not declare this option".into(),
            })?;
        if !decl.accepts(value) {
            return Err(HostError::BadOption {
                name: name.to_string(),
                reason: format!("{value:?} is not a value this engine accepts"),
            });
        }
        self.send(&format!("setoption name {name} value {value}"))?;
        if let Some(entry) = self.applied.iter_mut().find(|(set, _)| set == name) {
            entry.1 = value.to_string();
        } else {
            self.applied.push((name.to_string(), value.to_string()));
        }
        Ok(())
    }

    /// Press a button. A button carries no value — `setoption name Clear Hash` is the whole
    /// command — so it cannot go through `set_option`, and there is nothing to replay after
    /// a restart: a pressed button is not a state.
    pub fn press_option(&mut self, name: &str) -> Result<(), HostError> {
        match self.declared.iter().find(|decl| decl.name() == name) {
            Some(OptionDecl::Button { .. }) => self.send(&format!("setoption name {name}")),
            Some(_) => Err(HostError::BadOption {
                name: name.to_string(),
                reason: "only a button can be pressed".into(),
            }),
            None => Err(HostError::BadOption {
                name: name.to_string(),
                reason: "the engine does not declare this option".into(),
            }),
        }
    }

    /// The value we last set for an option, which is the state the app shows: UCI engines
    /// are not required to report their options back.
    pub fn option_value(&self, name: &str) -> Option<&str> {
        self.applied
            .iter()
            .find(|(set, _)| set == name)
            .map(|(_, value)| value.as_str())
    }

    /// Where the search starts from. Remembered so a restart can put the engine back.
    pub fn set_position(&mut self, position: Position) -> Result<(), HostError> {
        let command = position.to_command();
        self.send(&command)?;
        self.position = Some(command);
        Ok(())
    }

    /// Start a search and return immediately; the engine's `info` lines queue up.
    pub fn start_search(&mut self, limits: &Limits) -> Result<(), HostError> {
        self.recoveries = 0;
        self.start_again(limits)
    }

    /// Re-issue a search without re-arming the recovery budget, so a crashed engine
    /// cannot turn one request into an endless restart loop.
    fn start_again(&mut self, limits: &Limits) -> Result<(), HostError> {
        self.limits = Some(*limits);
        self.send(&limits.to_command())
    }

    /// Tell this engine — only this engine — to stop.
    pub fn stop(&mut self) -> Result<(), HostError> {
        self.send("stop")
    }

    /// Collect `info` lines until the engine's `bestmove`.
    /// Collect `info` lines until the engine's `bestmove`.
    ///
    /// If the engine dies mid-search it is restarted once and the search is asked for
    /// again against the restored position and options. A second death is reported rather
    /// than recovered from: an engine that crashes every search is a broken engine, and
    /// restarting it forever would hide that from the player.
    pub fn await_bestmove(
        &mut self,
        timeout: Option<Duration>,
    ) -> Result<SearchOutcome, HostError> {
        loop {
            match self.collect(timeout) {
                Err(HostError::Exited) if self.recoveries == 0 && self.limits.is_some() => {
                    self.recoveries += 1;
                    let limits = self.limits.unwrap_or_default();
                    self.restart()?;
                    self.start_again(&limits)?;
                }
                other => return other,
            }
        }
    }

    fn collect(&mut self, timeout: Option<Duration>) -> Result<SearchOutcome, HostError> {
        let mut outcome = SearchOutcome::default();
        loop {
            match self.next_event(timeout)? {
                Event::Info(info) => outcome.infos.push(info),
                Event::BestMove { bestmove, ponder } => {
                    outcome.bestmove = bestmove;
                    outcome.ponder = ponder;
                    return Ok(outcome);
                }
                // Tolerant by requirement: an engine's nonsense is logged by the reader
                // thread and skipped here, never fatal to the search.
                Event::Line(_) => {}
                Event::Exited => return Err(HostError::Exited),
                _ => {}
            }
        }
    }

    /// Search to the given limits and wait for the move.
    pub fn search(&mut self, limits: &Limits) -> Result<SearchOutcome, HostError> {
        self.start_search(limits)?;
        self.await_bestmove(None)
    }

    /// Is this engine still usable? A process that is running but silent is reported
    /// separately, because the app offers different actions for the two.
    pub fn health(&mut self) -> Health {
        self.health_within(PROBE_PATIENCE)
    }

    /// As [`Engine::health`], with the caller's own patience.
    pub fn health_within(&mut self, patience: Duration) -> Health {
        let Some(session) = self.session.as_mut() else {
            return Health::Down("not running".into());
        };
        if let Ok(Some(status)) = session.child.try_wait() {
            self.dead = true;
            return Health::Down(format!("exited with {status}"));
        }
        if self.dead {
            return Health::Down("its output ended".into());
        }
        match self.is_ready_within(patience) {
            Ok(()) => Health::Healthy,
            Err(HostError::Unresponsive) => Health::NotResponding,
            Err(_) => Health::Down("its output ended".into()),
        }
    }

    /// Bring the engine back after a crash: a fresh process, the same game, the same
    /// options, the same position. The player should lose their search, not their game.
    pub fn restart(&mut self) -> Result<EngineId, HostError> {
        self.session.take();
        self.session = Some(start(&self.program, &self.args, self.log_path.as_deref())?);
        self.dead = false;
        self.declared.clear();

        let id = self.handshake()?;
        self.new_game()?;
        let applied = self.applied.clone();
        for (name, value) in applied {
            self.send(&format!("setoption name {name} value {value}"))?;
        }
        if let Some(position) = self.position.clone() {
            self.send(&position)?;
        }
        Ok(id)
    }

    /// Say goodbye and wait for the process — briefly. An engine that ignores `quit` does
    /// not get to hang the application with it.
    pub fn quit(mut self) -> Result<(), HostError> {
        let _ = self.send("quit");
        if let Some(mut session) = self.session.take() {
            let deadline = Instant::now() + Duration::from_secs(2);
            while session.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            let _ = session.child.kill();
            let _ = session.child.wait();
        }
        Ok(())
    }
}

/// Try an executable once: start it, handshake, ask if it is ready, leave. This is what
/// registration runs, and what discovery runs against every candidate on `PATH`.
pub fn health_check(
    program: impl AsRef<Path>,
    args: &[&str],
    log_path: Option<impl AsRef<Path>>,
) -> Result<EngineInfo, HostError> {
    let mut engine = Engine::spawn(program, args, log_path)?;
    let id = engine.handshake()?;
    engine.is_ready()?;
    let info = EngineInfo {
        id,
        options: engine.declared().to_vec(),
    };
    engine.quit()?;
    Ok(info)
}

/// Who the engine said it is, and the option set it said it can be told. The second half
/// is what lets a feature that needs an absent option be hidden instead of silently
/// ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInfo {
    pub id: EngineId,
    pub options: Vec<OptionDecl>,
}

/// The engines the player has registered. Persistence is the app's business; this is the
/// list the app shows and the gate every entry passed on the way in.
#[derive(Debug, Default)]
pub struct Registry {
    engines: Vec<Registered>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registered {
    pub id: EngineId,
    pub program: PathBuf,
    pub args: Vec<String>,
    /// What this engine declared, recorded on the way in. The strength controls and the
    /// MultiPV control are offered from this list, not from hope.
    pub options: Vec<OptionDecl>,
}

impl Registry {
    /// Register an executable, which means it answered a handshake. Anything else is
    /// reported and not added.
    pub fn add(
        &mut self,
        program: impl AsRef<Path>,
        args: &[&str],
        log_path: Option<impl AsRef<Path>>,
    ) -> Result<Registered, HostError> {
        let info = health_check(program.as_ref(), args, log_path)?;
        let registered = Registered {
            id: info.id,
            program: program.as_ref().to_path_buf(),
            args: args.iter().map(|a| a.to_string()).collect(),
            options: info.options,
        };
        self.engines.retain(|e| e.program != registered.program);
        self.engines.push(registered.clone());
        Ok(registered)
    }

    pub fn remove(&mut self, program: &Path) -> bool {
        let before = self.engines.len();
        self.engines.retain(|e| e.program != program);
        self.engines.len() < before
    }

    pub fn list(&self) -> &[Registered] {
        &self.engines
    }
}
