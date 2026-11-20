//! Finding the engines that are already installed.
//!
//! Discovery never runs something by accident. The only way to learn whether a binary
//! speaks UCI is to run it, and `PATH` is full of binaries that are nobody's business, so
//! a file becomes a candidate only if its name is one of the engines this project knows.
//! Anything else is registered by pointing at it — [`Registry::add`](crate::Registry::add)
//! handshakes before it accepts, and so does [`probe`].

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{Engine, EngineInfo, HostError};

/// Names that make a file worth running, matched as lowercased substrings so
/// `stockfish-17.1-sse4`, `Arasan-26.1-linux-x86-64` and `Viridithas-v20` all count.
/// Every entry is a name no other software ships under: this list decides what the app is
/// allowed to execute without being told to, and the first machine it ran on found
/// `/Applications/Obsidian.app` and a Homebrew X11 front-end under the words that were
/// common enough to be product names. An engine missing here is still registrable by path.
pub const ENGINE_NAMES: &[&str] = &[
    "stockfish",
    "arasan",
    "viridithas",
    "viriarthor",
    "smallshark",
    "stormfish",
    "pandettix",
    "invictor",
    "tolc",
    "booot",
    "ruyng",
    "mctc",
    "bobcat",
    "dementor",
    "profab",
    "kepler",
    "naum",
    "komodo",
    "leela",
    "crafty",
    "ethereal",
    "godel",
    "adamant",
    "cheng",
    "wala",
    "bastian",
];

/// How far below a search directory a binary may be. Three is
/// `Stockfish.app/Contents/MacOS/stockfish` and no deeper than anything installs.
const MAX_DEPTH: usize = 3;

/// How long a candidate is given to say `uciok` and `readyok`. An engine that is thinking
/// about a network download is not going to make a first run wait twenty seconds; the
/// player who means to install something slow registers it by path, where a handshake is
/// given all it wants.
pub const DISCOVERY_PATIENCE: Duration = Duration::from_secs(5);

/// A file discovery ran, and what it said when asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub program: PathBuf,
    pub verdict: Verdict,
}

/// Either what the engine called itself and what it declared it can be told, or why it is
/// not an engine. The unusable case carries the session log, because "it failed" is not
/// something a player can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Usable(EngineInfo),
    Unusable { reason: String, log: String },
}

impl Candidate {
    pub fn is_usable(&self) -> bool {
        matches!(self.verdict, Verdict::Usable(_))
    }
}

/// Where to look before being told anything: `PATH`, then the directories a package
/// manager or a chess package fills on this platform.
pub fn default_search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = env::split_paths(&env::var_os("PATH").unwrap_or_default())
        .filter(|dir| !dir.as_os_str().is_empty())
        .collect();
    let home = env::var_os("HOME").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    {
        dirs.extend([
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/Applications"),
        ]);
        if let Some(home) = home {
            dirs.push(home.join("Applications"));
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        dirs.extend([
            PathBuf::from("/usr/games/bin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/local/games"),
        ]);
        if let Some(home) = home {
            dirs.push(home.join(".local/bin"));
        }
    }
    // Windows has no agreed place for an engine, and `PATH` is where a Windows engine
    // installer puts one anyway.
    dirs.retain(|dir| dir.is_dir());
    dirs.dedup();
    dirs
}

/// The executables in these directories whose names are on the list. Symlinks to the same
/// file count once.
pub fn scan(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    for dir in dirs {
        walk(dir, 0, &mut found, &mut seen);
    }
    found
}

fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            if !inside_a_foreign_bundle(&path) {
                walk(&path, depth + 1, found, seen);
            }
        } else if looks_like_an_engine(name)
            && is_executable(&path)
            && seen.insert(canonical(&path))
        {
            found.push(path);
        }
    }
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// A `.app` bundle is entered only when the bundle is named for an engine. Without this,
/// whatever sits in `/Applications` whose name happens to be on the list becomes a
/// candidate, and asking it for a move means launching the player's editor.
fn inside_a_foreign_bundle(dir: &Path) -> bool {
    let Some(name) = dir.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(bundle) = name.strip_suffix(".app") else {
        return false;
    };
    !looks_like_an_engine(bundle)
}

pub fn looks_like_an_engine(name: &str) -> bool {
    let name = name.to_lowercase();
    ENGINE_NAMES.iter().any(|engine| name.contains(engine))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}

/// Ask one file whether it is an engine. Where the log goes is the caller's choice; pass
/// `None` for one under the system temp directory, named after the candidate, so that an
/// unusable engine still has a transcript to show.
pub fn probe(program: &Path, log_path: Option<&Path>) -> Candidate {
    let log = log_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| probe_log(program));
    let mut engine = match Engine::spawn(program, &[], Some(&log)) {
        Ok(engine) => engine,
        Err(why) => return unusable(program, &log, &why),
    };
    // The handshake is the gate, and `isready` is part of it: an engine that names itself
    // and then never answers is not usable for play.
    let ready = engine
        .handshake_within(DISCOVERY_PATIENCE)
        .and_then(|id| engine.is_ready_within(DISCOVERY_PATIENCE).map(|()| id));
    // Dropping the engine at the end of this function kills the process, whether or not
    // it turned out to be an engine.
    match ready {
        Err(why) => unusable(program, &log, &why),
        Ok(id) => Candidate {
            program: program.to_path_buf(),
            verdict: Verdict::Usable(EngineInfo {
                id,
                options: engine.declared().to_vec(),
            }),
        },
    }
}

/// Every candidate, asked in turn. Slow by nature — each one is a process that may take
/// seconds to answer — so this belongs off the UI thread.
pub fn discover(dirs: &[PathBuf], log_dir: Option<&Path>) -> Vec<Candidate> {
    scan(dirs)
        .into_iter()
        .map(|program| {
            let log = log_dir.map(|dir| dir.join(probe_file_name(&program)));
            probe(&program, log.as_deref())
        })
        .collect()
}

fn probe_log(program: &Path) -> PathBuf {
    env::temp_dir().join(probe_file_name(program))
}

fn probe_file_name(program: &Path) -> PathBuf {
    let stem = program
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("engine");
    PathBuf::from(format!(
        "uci-host-discover-{stem}-{}.log",
        std::process::id()
    ))
}

fn unusable(program: &Path, log: &Path, why: &HostError) -> Candidate {
    Candidate {
        program: program.to_path_buf(),
        verdict: Verdict::Unusable {
            reason: why.to_string(),
            log: fs::read_to_string(log).unwrap_or_default(),
        },
    }
}
