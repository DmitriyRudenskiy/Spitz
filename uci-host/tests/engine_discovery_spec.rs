//! Autodetection (task 4.3): find the engines that are already installed, ask each one
//! what it is, and treat the ones that do not answer as unusable rather than as errors.
//!
//! The engines here are scripts, because discovery is about *finding* a process and not
//! about evaluating a position. A script that answers the handshake and a script that
//! introduces itself and dies are the two cases that matter, and both are cheap.

use std::fs;
use std::path::{Path, PathBuf};
use uci_host::Registry;
use uci_host::discover::{self, Verdict};

/// The awk program behind every candidate. It is a file of its own so the wrappers do not
/// fight the shell over quoting.
const ENGINE_AWK: &str = r#"{
  if ($0 == "uci") {
    print "id name Fake 1.0"
    print "id author Spitz tests"
    print "option name MultiPV type spin default 1 min 1 max 8"
    print "uciok"
    fflush()
  } else if ($0 == "isready") {
    print "readyok"
    fflush()
  } else if ($0 == "quit") {
    exit 0
  }
}
"#;

/// An engine that names itself and then leaves. The handshake does not complete, and the
/// process is gone before anything could time out.
const BROKEN: &str = "#!/bin/sh\necho 'id name Broken 0.1'\n";

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uci-host-discover-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bin")).expect("a sandbox with a bin directory");
    dir
}

fn executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// A candidate that answers the handshake, at whatever path the sandbox wants.
fn engine(path: &Path) {
    let awk = path.with_extension("awk");
    fs::write(&awk, ENGINE_AWK).unwrap();
    fs::write(
        path,
        format!("#!/bin/sh\nexec awk -f '{}' \"$@\"\n", awk.display()),
    )
    .unwrap();
    executable(path);
}

#[test]
fn a_candidate_on_the_path_is_listed_with_its_name_and_version() {
    let dir = sandbox("listed");
    let bin = dir.join("bin");
    engine(&bin.join("stockfish-17.1-sse4"));

    let found = discover::scan(std::slice::from_ref(&bin));
    assert_eq!(
        found,
        vec![bin.join("stockfish-17.1-sse4")],
        "the engine in a directory on the path is the candidate"
    );

    let candidate = discover::probe(&found[0], Some(&dir.join("probe.log")));
    assert!(candidate.is_usable());
    match candidate.verdict {
        Verdict::Usable(info) => {
            assert_eq!(info.id.name, "Fake 1.0", "listed by the name it gave");
            assert_eq!(info.id.author.as_deref(), Some("Spitz tests"));
            assert!(
                info.options.iter().any(|decl| decl.name() == "MultiPV"),
                "what it declared comes back with it: {:?}",
                info.options
            );
        }
        Verdict::Unusable { reason, log } => {
            panic!("an engine that answers is not unusable: {reason}\n{log}")
        }
    }
}

#[test]
fn an_executable_that_fails_the_handshake_is_reported_unusable_with_its_log() {
    let dir = sandbox("unusable");
    let bin = dir.join("bin");
    let broken = bin.join("stockfish-pretender");
    fs::write(&broken, BROKEN).unwrap();
    executable(&broken);

    let found = discover::scan(std::slice::from_ref(&bin));
    assert_eq!(found.len(), 1);
    let candidate = discover::probe(&found[0], Some(&dir.join("pretender.log")));
    assert!(!candidate.is_usable());

    let mut registry = Registry::default();
    let refused = registry.add(
        candidate.program.clone(),
        &[] as &[&str],
        Some(dir.join("refused.log")),
    );
    assert!(refused.is_err(), "and it is not registered either");
    assert!(registry.list().is_empty());

    match candidate.verdict {
        Verdict::Unusable { reason, log } => {
            assert!(!reason.is_empty(), "the player is told why: {reason}");
            assert!(
                log.contains("> uci") && log.contains("id name Broken 0.1"),
                "the captured log shows both directions, got:\n{log}"
            );
        }
        Verdict::Usable(info) => panic!("a handshake that never finished is usable: {:?}", info),
    }
}

#[test]
fn registering_a_candidate_takes_one_action_and_records_what_it_declared() {
    let dir = sandbox("register");
    let bin = dir.join("bin");
    engine(&bin.join("arasan-26.1"));
    let candidate = discover::probe(&bin.join("arasan-26.1"), Some(&dir.join("probe.log")));

    let mut registry = Registry::default();
    let registered = registry
        .add(
            candidate.program.clone(),
            &[] as &[&str],
            Some(dir.join("registered.log")),
        )
        .expect("one action, from the candidate the scan produced");

    assert_eq!(registered.id.name, "Fake 1.0");
    assert_eq!(registry.list().len(), 1);
    assert!(
        registry.list()[0]
            .options
            .iter()
            .any(|decl| decl.name() == "MultiPV"),
        "the option set is recorded, so a control that needs an undeclared option is hidden"
    );
}

#[test]
fn a_binary_that_is_not_a_known_engine_is_never_run() {
    let dir = sandbox("innocent");
    let bin = dir.join("bin");
    let sentinel = dir.join("was-run");
    let innocent = bin.join("notanengine");
    fs::write(
        &innocent,
        format!("#!/bin/sh\n: > '{}'\n", sentinel.display()),
    )
    .unwrap();
    executable(&innocent);
    engine(&bin.join("stockfish"));

    let found = discover::scan(std::slice::from_ref(&bin));
    assert_eq!(
        found,
        vec![bin.join("stockfish")],
        "only the name on the list is a candidate"
    );
    assert!(
        !sentinel.exists(),
        "discovery finds candidates; it is probing that runs them, and it probes only \
         what the scan listed"
    );

    let not_a_file = bin.join("stockfish-notes.txt");
    fs::write(&not_a_file, "stockfish\n").unwrap();
    assert!(
        !discover::scan(std::slice::from_ref(&bin))
            .iter()
            .any(|found| found == &not_a_file),
        "a file that merely mentions an engine is not one"
    );
}

#[test]
fn an_engine_inside_an_app_bundle_is_still_found() {
    let dir = sandbox("bundle");
    let bin = dir.join("bin");
    let inside = bin.join("Stockfish.app/Contents/MacOS");
    fs::create_dir_all(&inside).unwrap();
    engine(&inside.join("stockfish"));
    // One level deeper than any bundle, and out of the depth budget.
    let too_deep = bin.join("share/opt/engines/here");
    fs::create_dir_all(&too_deep).unwrap();
    engine(&too_deep.join("stockfish"));

    let found = discover::scan(std::slice::from_ref(&bin));
    assert_eq!(found, vec![inside.join("stockfish")]);
}

#[test]
fn a_foreign_app_bundle_is_never_entered() {
    let dir = sandbox("foreign");
    let bin = dir.join("bin");
    let inside = bin.join("Safari.app/Contents/MacOS");
    fs::create_dir_all(&inside).unwrap();
    engine(&inside.join("stockfish"));

    assert!(
        discover::scan(std::slice::from_ref(&bin)).is_empty(),
        "a bundle named for something else is not an engine, and finding it would mean \
         launching the player's browser to ask for a move"
    );
}

#[test]
fn the_same_engine_reached_two_ways_counts_once() {
    let dir = sandbox("dedup");
    let bin = dir.join("bin");
    engine(&bin.join("stockfish"));
    #[cfg(unix)]
    std::os::unix::fs::symlink(bin.join("stockfish"), bin.join("viridithas-link")).unwrap();

    let found = discover::scan(std::slice::from_ref(&bin));
    assert_eq!(
        found.len(),
        1,
        "one engine is one candidate, however it is spelled: {found:?}"
    );
}

#[test]
#[ignore = "depends on what is installed on the machine running it; run by hand with --ignored"]
fn the_machines_own_engines_are_found() {
    let mut dirs = discover::default_search_dirs();
    // Extra directories for a machine whose engines are not installed anywhere.
    if let Some(extra) = std::env::var_os("SPITZ_DISCOVER_DIRS") {
        dirs.extend(std::env::split_paths(&extra));
    }
    eprintln!("searching {} director(ies)", dirs.len());
    for candidate in discover::discover(&dirs, None) {
        match candidate.verdict {
            Verdict::Usable(info) => eprintln!(
                "{}: {} ({} option(s))",
                candidate.program.display(),
                info.id.name,
                info.options.len()
            ),
            Verdict::Unusable { reason, .. } => {
                eprintln!("{}: unusable — {reason}", candidate.program.display())
            }
        }
    }
}

#[test]
fn discovery_reports_every_candidate_it_found_usable_or_not() {
    let dir = sandbox("all");
    let bin = dir.join("bin");
    engine(&bin.join("stockfish"));
    let broken = bin.join("naum-pretender");
    fs::write(&broken, BROKEN).unwrap();
    executable(&broken);

    let found = discover::discover(std::slice::from_ref(&bin), Some(&dir));
    assert_eq!(found.len(), 2);
    assert_eq!(found.iter().filter(|c| c.is_usable()).count(), 1);
    assert_eq!(found.iter().filter(|c| !c.is_usable()).count(), 1);
    let logs = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("uci-host-discover-")
        })
        .count();
    assert_eq!(logs, 2, "each candidate was asked in its own log");
}
