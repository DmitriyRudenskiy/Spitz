//! Library sessions: opening a library, holding a cooperative lock on it, and keeping a bounded
//! number of them open.
//!
//! A library is a set of files sharing a base name. Opening one for writing takes a lock naming
//! this host and process; opening it read-only never writes a byte, which is what makes a library
//! on a mounted disc safe to browse.

use crate::header_record::{HeaderRecord, HEADER_RECORD_SIZE, INDEX_FORMAT_VERSION};
use crate::name_dict::{NameDictionary, NameKind};
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// How a library was opened.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    ReadWrite,
    ReadOnly,
}

/// The files of one library, named by their base.
#[derive(Clone, Debug)]
pub struct LibraryFiles {
    base: PathBuf,
}

impl LibraryFiles {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self { base: base.into() }
    }

    /// The index: version byte, count, then one [`HeaderRecord`] per game.
    pub fn index(&self) -> PathBuf {
        self.with_extension("gidx")
    }

    /// The four name dictionaries.
    pub fn names(&self) -> PathBuf {
        self.with_extension("gnam")
    }

    /// The game records, concatenated; the index points into it by offset and length.
    pub fn games(&self) -> PathBuf {
        self.with_extension("ggames")
    }

    /// The cooperative write lock. Only a writable session creates it.
    pub fn lock(&self) -> PathBuf {
        self.with_extension("glock")
    }

    pub fn base(&self) -> &Path {
        &self.base
    }

    fn with_extension(&self, extension: &str) -> PathBuf {
        let mut path = self.base.clone().into_os_string();
        path.push(format!(".{extension}"));
        PathBuf::from(path)
    }

    /// Every file that belongs to the library, whether or not it exists yet.
    pub fn all(&self) -> [PathBuf; 4] {
        [self.index(), self.names(), self.games(), self.lock()]
    }

    /// The library's own files, excluding the lock.
    pub fn data(&self) -> [PathBuf; 3] {
        [self.index(), self.names(), self.games()]
    }
}

/// What a library looks like on disc before it is opened.
pub fn exists(files: &LibraryFiles) -> bool {
    files.index().exists() && files.games().exists()
}

/// Why a library could not be opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenError {
    /// Nothing to open at that path.
    NotFound(PathBuf),
    /// Another session holds the write lock. `offering_read_only` is always true: the caller can
    /// always browse the library, only not change it.
    Locked { lock: PathBuf, host: String, pid: u32 },
    /// The index file is not a library we understand.
    Version(u8),
    /// A file was shorter than it claimed.
    Truncated(PathBuf),
    Io(String),
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenError::NotFound(path) => write!(f, "no library at {}", path.display()),
            OpenError::Locked { lock, host, pid } => write!(
                f,
                "{} is locked by process {pid} on {host}; it can be opened read-only",
                lock.display()
            ),
            OpenError::Version(v) => write!(f, "library version {v}, expected {INDEX_FORMAT_VERSION}"),
            OpenError::Truncated(path) => write!(f, "{} is truncated", path.display()),
            OpenError::Io(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for OpenError {}

impl From<SessionError> for OpenError {
    fn from(why: SessionError) -> Self {
        match why {
            SessionError::ReadOnly => OpenError::Io("library is open read-only".to_owned()),
            SessionError::NoSuchRow(row) => OpenError::Io(format!("no game at row {row}")),
            SessionError::Io(why) => OpenError::Io(why),
        }
    }
}

impl From<io::Error> for OpenError {
    fn from(why: io::Error) -> Self {
        OpenError::Io(why.to_string())
    }
}

/// One open library.
#[derive(Debug)]
pub struct Session {
    files: LibraryFiles,
    mode: Mode,
    records: Vec<HeaderRecord>,
    names: Names,
    games: Vec<u8>,
    /// Bytes added since the last flush, which is what makes a close worth doing.
    dirty: bool,
}

/// The four dictionaries a library carries.
#[derive(Clone, Debug)]
pub struct Names {
    pub players: NameDictionary,
    pub events: NameDictionary,
    pub sites: NameDictionary,
    pub rounds: NameDictionary,
}

impl Names {
    fn new() -> Self {
        Self {
            players: NameDictionary::new(NameKind::Player),
            events: NameDictionary::new(NameKind::Event),
            sites: NameDictionary::new(NameKind::Site),
            rounds: NameDictionary::new(NameKind::Round),
        }
    }

    pub fn of(&self, kind: NameKind) -> &NameDictionary {
        match kind {
            NameKind::Player => &self.players,
            NameKind::Event => &self.events,
            NameKind::Site => &self.sites,
            NameKind::Round => &self.rounds,
        }
    }

    pub fn of_mut(&mut self, kind: NameKind) -> &mut NameDictionary {
        match kind {
            NameKind::Player => &mut self.players,
            NameKind::Event => &mut self.events,
            NameKind::Site => &mut self.sites,
            NameKind::Round => &mut self.rounds,
        }
    }
}

impl Session {
    /// Open an existing library. A library that cannot be written — by permission or by mount —
    /// opens read-only whatever was asked for.
    pub fn open(files: LibraryFiles, wanted: Mode) -> Result<Self, OpenError> {
        if !files.index().exists() || !files.games().exists() {
            return Err(OpenError::NotFound(files.base().to_path_buf()));
        }
        let mode = acquire(&files, wanted, &files.index())?;
        let mut session = Session {
            files,
            mode,
            records: Vec::new(),
            names: Names::new(),
            games: Vec::new(),
            dirty: false,
        };
        session.read_index()?;
        session.read_names()?;
        session.read_games()?;
        Ok(session)
    }

    /// Create a new empty library, taking the lock if it can be written.
    pub fn create(files: LibraryFiles) -> Result<Self, OpenError> {
        let mode = acquire(&files, Mode::ReadWrite, files.base())?;
        let mut session = Session {
            files,
            mode,
            records: Vec::new(),
            names: Names::new(),
            games: Vec::new(),
            dirty: true,
        };
        session.flush()?;
        Ok(session)
    }

    pub fn files(&self) -> &LibraryFiles {
        &self.files
    }

    pub fn base(&self) -> &Path {
        &self.files.base
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == Mode::ReadOnly
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn record(&self, row: usize) -> Option<HeaderRecord> {
        self.records.get(row).copied()
    }

    pub fn records(&self) -> &[HeaderRecord] {
        &self.records
    }

    pub fn names(&self) -> &Names {
        &self.names
    }

    pub fn names_mut(&mut self) -> &mut Names {
        &mut self.names
    }

    /// The stored bytes of one game.
    pub fn game(&self, row: usize) -> Option<&[u8]> {
        let record = self.records.get(row)?;
        let start = record.offset as usize;
        let end = start.checked_add(record.length as usize)?;
        self.games.get(start..end)
    }

    /// Append a game. The header's offset and length are filled in by the library; the name ids
    /// are the caller's, from [`Self::names_mut`].
    pub fn add_game(&mut self, mut header: HeaderRecord, bytes: &[u8]) -> Result<usize, SessionError> {
        if self.is_read_only() {
            return Err(SessionError::ReadOnly);
        }
        header.offset = self.games.len() as u64;
        header.length = bytes.len() as u32;
        let row = self.records.len();
        self.games.extend_from_slice(bytes);
        self.records.push(header);
        self.dirty = true;
        Ok(row)
    }

    /// Drop a game's index entry. Its bytes stay until the library is rebuilt, which is what
    /// every database of this shape does and why rebuild exists.
    pub fn remove_game(&mut self, row: usize) -> Result<HeaderRecord, SessionError> {
        if self.is_read_only() {
            return Err(SessionError::ReadOnly);
        }
        if row >= self.records.len() {
            return Err(SessionError::NoSuchRow(row));
        }
        self.dirty = true;
        Ok(self.records.remove(row))
    }

    /// Write the index and the name dictionaries. The game file only ever grows by appending, so
    /// it is written whole here rather than tracked byte by byte.
    pub fn flush(&mut self) -> Result<(), SessionError> {
        if self.is_read_only() {
            return Ok(());
        }
        if !self.dirty {
            return Ok(());
        }
        let mut index = Vec::with_capacity(5 + self.records.len() * HEADER_RECORD_SIZE);
        index.push(INDEX_FORMAT_VERSION);
        index.extend_from_slice(&(self.records.len() as u32).to_le_bytes());
        for record in &self.records {
            index.extend_from_slice(&record.encode());
        }
        write_atomic(&self.files.index(), &index).map_err(|why| SessionError::Io(why.to_string()))?;

        let mut names = Vec::new();
        for dictionary in [
            &self.names.players,
            &self.names.events,
            &self.names.sites,
            &self.names.rounds,
        ] {
            let bytes = dictionary.encode();
            names.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            names.extend_from_slice(&bytes);
        }
        write_atomic(&self.files.names(), &names).map_err(|why| SessionError::Io(why.to_string()))?;
        write_atomic(&self.files.games(), &self.games)
            .map_err(|why| SessionError::Io(why.to_string()))?;
        self.dirty = false;
        Ok(())
    }

    /// Close the library: flush what is owed, release the lock.
    pub fn close(mut self) -> Result<(), SessionError> {
        let result = self.flush();
        if self.mode == Mode::ReadWrite {
            let _ = fs::remove_file(self.files.lock());
        }
        result
    }

    fn read_index(&mut self) -> Result<(), OpenError> {
        let bytes = read(&self.files.index())?;
        let Some((version, rest)) = bytes.split_first() else {
            return Err(OpenError::Truncated(self.files.index()));
        };
        if *version != INDEX_FORMAT_VERSION {
            return Err(OpenError::Version(*version));
        }
        if rest.len() < 4 {
            return Err(OpenError::Truncated(self.files.index()));
        }
        let count = u32::from_le_bytes(rest[..4].try_into().unwrap()) as usize;
        let rest = &rest[4..];
        if rest.len() < count * HEADER_RECORD_SIZE {
            return Err(OpenError::Truncated(self.files.index()));
        }
        self.records = rest[..count * HEADER_RECORD_SIZE]
            .chunks(HEADER_RECORD_SIZE)
            .map(HeaderRecord::decode)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| OpenError::Truncated(self.files.index()))?;
        Ok(())
    }

    fn read_names(&mut self) -> Result<(), OpenError> {
        let path = self.files.names();
        let Ok(bytes) = read(&path) else { return Ok(()) };
        let mut rest = bytes.as_slice();
        let read_one = |rest: &mut &[u8]| -> Result<NameDictionary, OpenError> {
            if rest.len() < 8 {
                return Err(OpenError::Truncated(path.clone()));
            }
            let len = u64::from_le_bytes(rest[..8].try_into().unwrap()) as usize;
            if rest.len() < 8 + len {
                return Err(OpenError::Truncated(path.clone()));
            }
            let dictionary = NameDictionary::decode(&rest[8..8 + len])
                .map_err(|_| OpenError::Truncated(path.clone()))?;
            *rest = &rest[8 + len..];
            Ok(dictionary)
        };
        self.names.players = read_one(&mut rest)?;
        self.names.events = read_one(&mut rest)?;
        self.names.sites = read_one(&mut rest)?;
        self.names.rounds = read_one(&mut rest)?;
        Ok(())
    }

    fn read_games(&mut self) -> Result<(), OpenError> {
        self.games = read(&self.files.games())?;
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // A session that is dropped without `close` still owes its writes and still must not
        // leave a lock behind.
        if self.mode == Mode::ReadWrite {
            let _ = self.flush();
            let _ = fs::remove_file(self.files.lock());
        }
    }
}

#[derive(Debug)]
pub enum SessionError {
    ReadOnly,
    NoSuchRow(usize),
    Io(String),
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::ReadOnly => write!(f, "library is open read-only"),
            SessionError::NoSuchRow(row) => write!(f, "no game at row {row}"),
            SessionError::Io(why) => write!(f, "{why}"),
        }
    }
}

/// Whether the permission bits let this path be written. A path that does not exist yet is
/// judged by the directory that would hold it; a read-only mount is caught later, when the lock
/// file is actually created.
pub fn writable(path: &Path) -> bool {
    let target = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent().map_or_else(|| PathBuf::from("."), PathBuf::from)
    };
    fs::metadata(&target).is_ok_and(|meta| !meta.permissions().readonly())
}

/// The machine this lock belongs to, for the message that names who holds it.
fn host_name() -> String {
    if let Ok(host) = std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME"))
        && !host.is_empty()
    {
        return host;
    }
    if let Ok(host) = fs::read_to_string("/etc/hostname") {
        let host = host.trim();
        if !host.is_empty() {
            return host.to_owned();
        }
    }
    "unknown host".to_owned()
}

fn process_alive(pid: u32) -> bool {
    // Linux has the process table in the filesystem; elsewhere ask the process server.
    if Path::new("/proc").exists() { return Path::new(&format!("/proc/{pid}")).exists(); }
    std::process::Command::new("/bin/kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Decide the mode to open in, which means taking the write lock when one was asked for and the
/// library can be written at all.
fn acquire(files: &LibraryFiles, wanted: Mode, probe: &Path) -> Result<Mode, OpenError> {
    if wanted == Mode::ReadOnly || !writable(probe) {
        return Ok(Mode::ReadOnly);
    }
    let lock = files.lock();
    if let Ok(body) = fs::read_to_string(&lock) {
        let (host, pid) = parse_lock(&body);
        if host == host_name() && !process_alive(pid) {
            fs::remove_file(&lock).ok(); // an unclean exit: the holder is gone
        } else {
            return Err(OpenError::Locked { lock, host, pid });
        }
    }
    match File::create(&lock) {
        Ok(mut body) => {
            write_all(&mut body, format!("{}\n{}\n", host_name(), std::process::id()).as_bytes())?;
            Ok(Mode::ReadWrite)
        }
        // A read-only mount refuses the lock the same way it refuses any other write, and the
        // library is browsable either way.
        Err(why) if matches!(why.kind(), io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem) => {
            Ok(Mode::ReadOnly)
        }
        Err(why) => Err(OpenError::Io(format!("cannot write {}: {why}", lock.display()))),
    }
}

fn parse_lock(body: &str) -> (String, u32) {
    let mut lines = body.lines();
    let host = lines.next().unwrap_or("unknown host").to_owned();
    let pid = lines.next().and_then(|line| line.trim().parse().ok()).unwrap_or(0);
    (host, pid)
}

/// Who holds the lock, if anybody.
pub fn lock_holder(files: &LibraryFiles) -> Option<(String, u32)> {
    fs::read_to_string(files.lock()).ok().map(|body| parse_lock(&body))
}

fn read(path: &Path) -> Result<Vec<u8>, OpenError> {
    let mut file = File::open(path).map_err(|_| OpenError::NotFound(path.to_path_buf()))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|why| OpenError::Io(why.to_string()))?;
    Ok(bytes)
}

fn write_all(file: &mut File, bytes: &[u8]) -> Result<(), OpenError> {
    file.write_all(bytes).map_err(|why| OpenError::Io(why.to_string()))
}

/// Write a file by replacing it, so an interrupted flush leaves the previous library readable.
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temp = path.with_extension("gtmp");
    {
        let mut file = File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(&temp, path)
}

/// A library's size and modification time, for the test that proves nothing was written.
pub fn fingerprint(path: &Path) -> Option<(u64, SystemTime)> {
    let meta: Metadata = fs::metadata(path).ok()?;
    Some((meta.len(), meta.modified().ok()?))
}

/// The open libraries, least-recently-used closed when the limit is reached.
pub struct SessionPool {
    limit: usize,
    sessions: HashMap<PathBuf, Session>,
    /// Base names, oldest first.
    order: Vec<PathBuf>,
}

impl SessionPool {
    /// A pool holding at most `limit` libraries. The app's default is eight; anything below one
    /// is treated as one.
    pub fn new(limit: usize) -> Self {
        Self { limit: limit.max(1), sessions: HashMap::new(), order: Vec::new() }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn is_open(&self, base: &Path) -> bool {
        self.sessions.contains_key(base)
    }

    /// Open a library, closing the least recently used one if the pool is full.
    pub fn open(&mut self, files: LibraryFiles, wanted: Mode) -> Result<&mut Session, OpenError> {
        let base = files.base.clone();
        if self.sessions.len() >= self.limit && !self.sessions.contains_key(&base) {
            self.close_least_recently_used()?;
        }
        if !self.sessions.contains_key(&base) {
            let session = Session::open(files, wanted)?;
            self.sessions.insert(base.clone(), session);
        }
        self.touch(&base);
        Ok(self.sessions.get_mut(&base).expect("just inserted"))
    }

    /// An open library, marking it the most recently used.
    pub fn get(&mut self, base: &Path) -> Option<&mut Session> {
        if !self.sessions.contains_key(base) {
            return None;
        }
        self.touch(base);
        self.sessions.get_mut(base)
    }

    /// Close one library, flushing what it owes.
    pub fn close(&mut self, base: &Path) -> Result<(), SessionError> {
        self.order.retain(|path| path != base);
        match self.sessions.remove(base) {
            Some(session) => session.close(),
            None => Ok(()),
        }
    }

    fn touch(&mut self, base: &Path) {
        self.order.retain(|path| path != base);
        self.order.push(base.to_path_buf());
    }

    fn close_least_recently_used(&mut self) -> Result<(), OpenError> {
        let Some(victim) = self.order.first().cloned() else { return Ok(()) };
        self.close(&victim).map_err(|why| OpenError::Io(why.to_string()))
    }

    pub fn open_bases(&self) -> &[PathBuf] {
        &self.order
    }
}

impl Drop for SessionPool {
    fn drop(&mut self) {
        // Every session would flush and unlock in its own Drop; this makes the order explicit and
        // keeps a panic in one library from leaving the rest locked.
        for (_, session) in self.sessions.drain() {
            let _ = session.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("spitz-session-{tag}-{n}-{}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn base(&self, name: &str) -> LibraryFiles {
            LibraryFiles::new(self.0.join(name))
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn library_with(games: usize) -> (TempDir, LibraryFiles) {
        let dir = TempDir::new("lib");
        let files = dir.base("games");
        {
            let mut session = Session::create(files.clone()).unwrap();
            for i in 0..games {
                let header = HeaderRecord {
                    white: session.names().players.len() as u32,
                    half_moves: i as u16,
                    ..Default::default()
                };
                session.add_game(header, format!("game {i}").as_bytes()).unwrap();
            }
            session.names_mut().players.intern("Anand");
            session.close().unwrap();
        }
        (dir, files)
    }

    #[test]
    fn a_library_survives_being_closed_and_reopened() {
        let (_dir, files) = library_with(5);
        let session = Session::open(files.clone(), Mode::ReadWrite).unwrap();
        assert_eq!(session.len(), 5);
        assert_eq!(session.mode(), Mode::ReadWrite);
        assert_eq!(session.game(3), Some(&b"game 3"[..]));
        assert_eq!(session.record(4).unwrap().half_moves, 4);
        assert_eq!(session.names().players.name(1), Some("Anand"));
    }

    #[test]
    fn a_writable_opening_locks_the_library() {
        let (_dir, files) = library_with(1);
        let session = Session::open(files.clone(), Mode::ReadWrite).unwrap();
        assert!(lock_holder(&files).is_some(), "the lock names this host and process");
        drop(session);
        assert!(lock_holder(&files).is_none(), "closing releases it");
    }

    #[test]
    fn a_second_writer_is_refused_and_nothing_changes() {
        let (_dir, files) = library_with(3);
        let held = Session::open(files.clone(), Mode::ReadWrite).unwrap();
        let before: Vec<_> = files.data().iter().map(|path| fingerprint(path)).collect();

        let error = Session::open(files.clone(), Mode::ReadWrite).unwrap_err();
        match &error {
            OpenError::Locked { lock, host, pid } => {
                assert_eq!(lock, &files.lock(), "the message names the locked path");
                assert!(!host.is_empty());
                assert_eq!(*pid, std::process::id());
            }
            other => panic!("expected a refusal, got {other:?}"),
        }
        let after: Vec<_> = files.data().iter().map(|path| fingerprint(path)).collect();
        assert_eq!(before, after, "a refused opening writes nothing");

        // The same library can always be browsed.
        let mut browsing = Session::open(files.clone(), Mode::ReadOnly).unwrap();
        assert!(browsing.is_read_only());
        assert_eq!(browsing.len(), 3);
        assert!(browsing.add_game(HeaderRecord::default(), b"no").is_err());
        let after_read: Vec<_> = files.data().iter().map(|path| fingerprint(path)).collect();
        assert_eq!(before, after_read, "browsing writes nothing either");
        drop(held);
    }

    #[test]
    fn a_read_only_library_opens_without_writing_and_leaves_no_lock() {
        let (_dir, files) = library_with(4);
        for path in files.data() {
            fs::set_permissions(&path, PermissionsExt::from_mode(0o444)).unwrap();
        }
        let before: Vec<_> = files.data().iter().map(|path| fingerprint(path)).collect();
        let session = Session::open(files.clone(), Mode::ReadWrite).unwrap();
        assert_eq!(session.mode(), Mode::ReadOnly, "the permissions decide");
        assert_eq!(session.len(), 4);
        assert!(session.game(0).is_some());
        assert!(lock_holder(&files).is_none(), "a read-only opening takes no lock");
        drop(session);
        let after: Vec<_> = files.data().iter().map(|path| fingerprint(path)).collect();
        assert_eq!(before, after, "size and modification time are untouched");
        assert!(!files.lock().exists());
    }

    #[test]
    fn a_lock_left_by_a_dead_process_is_reclaimed() {
        let (_dir, files) = library_with(1);
        // An unclean exit: the lock is there and nobody is home. The pid is one no process can
        // have, and the host is this one so the staleness test is the one that decides.
        fs::write(files.lock(), format!("{}\n99999999\n", host_name())).unwrap();
        let session = Session::open(files.clone(), Mode::ReadWrite).unwrap();
        assert_eq!(session.mode(), Mode::ReadWrite);
        let (host, pid) = lock_holder(&files).unwrap();
        assert_eq!(host, host_name());
        assert_eq!(pid, std::process::id());
    }

    #[test]
    fn a_lock_held_on_another_host_is_not_messaged_with() {
        let (_dir, files) = library_with(1);
        fs::write(files.lock(), "some-other-machine\n42\n").unwrap();
        let error = Session::open(files.clone(), Mode::ReadWrite).unwrap_err();
        assert!(matches!(error, OpenError::Locked { pid: 42, .. }));
    }

    #[test]
    fn a_foreign_or_damaged_index_is_refused() {
        let dir = TempDir::new("bad");
        let files = dir.base("empty");
        fs::write(files.index(), []).unwrap();
        fs::write(files.games(), []).unwrap();
        assert!(matches!(Session::open(files.clone(), Mode::ReadOnly), Err(OpenError::Truncated(_))));
        fs::write(files.index(), [9u8, 0, 0, 0, 0]).unwrap();
        assert!(matches!(Session::open(files.clone(), Mode::ReadOnly), Err(OpenError::Version(9))));
        assert!(matches!(Session::open(dir.base("nothing"), Mode::ReadOnly), Err(OpenError::NotFound(_))));
    }

    #[test]
    fn eight_libraries_stay_open_and_the_oldest_is_the_one_closed() {
        let dir = TempDir::new("pool");
        let mut bases = Vec::new();
        for i in 0..9 {
            let files = dir.base(&format!("lib{i}"));
            let mut session = Session::create(files.clone()).unwrap();
            session.add_game(HeaderRecord::default(), b"one game").unwrap();
            session.close().unwrap();
            bases.push(files);
        }
        let mut pool = SessionPool::new(8);
        for files in &bases {
            pool.open(files.clone(), Mode::ReadWrite).unwrap();
        }
        assert_eq!(pool.len(), 8, "the pool holds eight, not seven");
        assert!(!pool.is_open(&bases[0].base), "the least recently used gave its place up");
        assert!(pool.is_open(&bases[8].base));
        assert!(!bases[0].lock().exists(), "an evicted library is unlocked");
        assert!(pool.get(&bases[1].base).is_some());

        // The ninth open evicted lib1's place, so lib1 is back and something else gave way.
        assert_eq!(pool.open(bases[0].clone(), Mode::ReadWrite).unwrap().len(), 1);
        assert_eq!(pool.len(), 8);
    }

    #[test]
    fn eviction_flushes_before_it_forgets() {
        let dir = TempDir::new("evict");
        let mut pool = SessionPool::new(1);
        let first = dir.base("first");
        Session::create(first.clone()).unwrap().close().unwrap();
        pool.open(first.clone(), Mode::ReadWrite).unwrap();
        // Unflushed work in the evicted session must reach the disc.
        pool.get(first.base()).unwrap().add_game(HeaderRecord::default(), b"owed").unwrap();
        let second = dir.base("second");
        {
            Session::create(second.clone()).unwrap().close().unwrap();
        }
        pool.open(second.clone(), Mode::ReadWrite).unwrap();
        assert!(!pool.is_open(first.base()));
        let reopened = Session::open(first.clone(), Mode::ReadOnly).unwrap();
        assert_eq!(reopened.len(), 1, "the owed write was flushed on eviction");
    }

    #[test]
    fn dropping_the_pool_leaves_no_locks_behind() {
        let dir = TempDir::new("drop");
        let mut pool = SessionPool::new(8);
        let mut bases = Vec::new();
        for i in 0..4 {
            let files = dir.base(&format!("lib{i}"));
            {
                Session::create(files.clone()).unwrap().close().unwrap();
            }
            pool.open(files.clone(), Mode::ReadWrite).unwrap();
            bases.push(files);
        }
        assert!(bases.iter().all(|files| files.lock().exists()));
        drop(pool);
        for files in &bases {
            assert!(!files.lock().exists(), "{} left locked", files.lock().display());
        }
    }
}
