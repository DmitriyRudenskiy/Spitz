//! The ladder as a command: `ladder --engine ./stockfish --games 1000`.
//!
//! Exits non-zero when a pair of adjacent levels is not separated, which is what makes it
//! a nightly gate and not a chart nobody reads.

use std::path::PathBuf;
use std::process::ExitCode;

use ladder::{Level, MatchConfig, Pair, SEPARATION, run};

const USAGE: &str = "usage: ladder [--engine PATH] [--games N] [--levels A-B]\n\
                           [--depth N | --nodes N] [--max-plies N] [--seed N] [--log-dir DIR]\n\
       SPITZ_ENGINES (colon-separated) is used when --engine is absent.";

#[derive(Debug)]
struct Args {
    engine: Option<PathBuf>,
    config: MatchConfig,
    levels: (u8, u8),
    log_dir: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(why) => {
            eprintln!("{why}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    let Some(engine) = args.engine.or_else(first_engine_on_the_path) else {
        eprintln!("no engine: pass --engine PATH or set SPITZ_ENGINES\n\n{USAGE}");
        return ExitCode::FAILURE;
    };

    println!(
        "{}: levels {}-{}, {} game(s) per pair at {}, seed {:#x}",
        engine.display(),
        args.levels.0,
        args.levels.1,
        args.config.games,
        args.config.budget(),
        args.config.seed
    );
    let results = run(
        &engine,
        args.levels.0..=args.levels.1,
        &args.config,
        args.log_dir.as_deref(),
        |pair| println!("{pair}"),
    );
    let results = match results {
        Ok(results) => results,
        Err(why) => {
            eprintln!("ladder stopped: {why}");
            return ExitCode::FAILURE;
        }
    };

    let failed: Vec<&Pair> = results.iter().filter(|pair| !pair.is_separated()).collect();
    let games: u32 = results.iter().map(Pair::games).sum();
    println!(
        "{games} game(s), {}/{} pair(s) separated (>= {:.0}%)",
        results.len() - failed.len(),
        results.len(),
        SEPARATION * 100.0
    );
    for pair in &failed {
        eprintln!(
            "level {} is not stronger than level {}: {:.1}%",
            pair.higher,
            pair.lower,
            pair.score_rate() * 100.0
        );
    }
    if failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The first engine that is actually there — the same environment contract the compliance
/// suite runs on, so one fetch script feeds both.
fn first_engine_on_the_path() -> Option<PathBuf> {
    std::env::var_os("SPITZ_ENGINES")
        .iter()
        .flat_map(|paths| std::env::split_paths(paths))
        .find(|path| path.is_file())
}

fn value(args: &[String], at: &mut usize, name: &str) -> Result<String, String> {
    *at += 1;
    args.get(*at)
        .cloned()
        .ok_or_else(|| format!("{name} needs a value"))
}

fn number(name: &str, text: &str) -> Result<u64, String> {
    text.parse()
        .map_err(|_| format!("{name} wants a number, got {text}"))
}

fn parse(given: impl Iterator<Item = String>) -> Result<Args, String> {
    let args: Vec<String> = given.collect();
    let mut parsed = Args {
        engine: None,
        config: MatchConfig::default(),
        levels: (1, ladder::LEVELS),
        log_dir: None,
    };
    let mut at = 0;
    while at < args.len() {
        let flag = args[at].as_str();
        match flag {
            "--engine" => parsed.engine = Some(PathBuf::from(value(&args, &mut at, flag)?)),
            "--games" => {
                let text = value(&args, &mut at, flag)?;
                parsed.config.games = number(flag, &text)? as u32;
            }
            "--depth" => {
                let text = value(&args, &mut at, flag)?;
                parsed.config.depth = number(flag, &text)? as u32;
            }
            "--nodes" => {
                let text = value(&args, &mut at, flag)?;
                parsed.config.nodes = number(flag, &text)?;
            }
            "--max-plies" => {
                let text = value(&args, &mut at, flag)?;
                parsed.config.max_plies = number(flag, &text)? as u16;
            }
            "--seed" => {
                let text = value(&args, &mut at, flag)?;
                parsed.config.seed = number(flag, &text)?;
            }
            "--levels" => {
                let text = value(&args, &mut at, flag)?;
                let (low, high) = text
                    .split_once('-')
                    .ok_or_else(|| format!("--levels wants A-B, got {text}"))?;
                parsed.levels = (number(flag, low)? as u8, number(flag, high)? as u8);
            }
            "--log-dir" => parsed.log_dir = Some(PathBuf::from(value(&args, &mut at, flag)?)),
            "--help" | "-h" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        at += 1;
    }

    if parsed.config.games == 0 {
        return Err(String::from("--games wants at least one game"));
    }
    let (low, high) = parsed.levels;
    let inside = |level: u8| (Level::MIN.get()..=Level::MAX.get()).contains(&level);
    if !inside(low) || !inside(high) || low >= high {
        return Err(format!(
            "--levels wants a range inside {}-{}, got {low}-{high}",
            Level::MIN.get(),
            Level::MAX.get()
        ));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::parse;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn a_match_defaults_to_the_whole_ladder_and_two_games() {
        let parsed = parse(args("--engine /bin/true").into_iter()).unwrap();
        assert_eq!(parsed.levels, (1, ladder::LEVELS));
        assert_eq!(parsed.config.games, 2);
        assert_eq!(parsed.engine.unwrap().to_str().unwrap(), "/bin/true");
    }

    #[test]
    fn a_range_outside_the_ladder_is_refused_before_a_process_starts() {
        for bad in [
            "--levels 0-3",
            "--levels 19-21",
            "--levels 7-7",
            "--levels 8",
        ] {
            assert!(
                parse(args(bad).into_iter()).is_err(),
                "{bad} should not be a ladder"
            );
        }
    }

    #[test]
    fn a_flag_with_no_value_is_a_message_and_not_a_panic() {
        let err = parse(args("--games").into_iter()).unwrap_err();
        assert!(err.contains("needs a value"), "{err}");
        let err = parse(args("--games many").into_iter()).unwrap_err();
        assert!(err.contains("wants a number"), "{err}");
    }
}
