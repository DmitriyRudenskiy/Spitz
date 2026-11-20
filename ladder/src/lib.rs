//! The strength ladder: the measurement the level tables are only a guess at.
//!
//! Two engines, two levels, one position, and a winner. Adjacent levels play each other
//! with colours swapped, and the higher level has to score at least 55% of the points —
//! below that the two levels are the same opponent wearing different numbers, and the
//! ladder says which pair failed.
//!
//! Everything here is one layer above the parts: the engine is hosted by `uci-host`, the
//! move is chosen by `move-select`, the rules are `chess-core`. What this crate owns is
//! the loop, the pairing, and the arithmetic of whether a level is real.

use std::fmt;
use std::path::Path;

use chess_core::board::Board;
use chess_core::{fen, state, uci};
use move_select::{self, EngineOptions, EngineReport, Line, Score};

pub use move_select::{LEVELS, Level, Mechanism, Reason, Selection};
use uci_host::{Engine, HostError, Info, InfoScore, Limits, OptionDecl, Position};

/// The share of the points the higher level needs for a pair to count as separated.
pub const SEPARATION: f64 = 0.55;

/// What one pair of levels did against each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair {
    pub lower: u8,
    pub higher: u8,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

impl Pair {
    pub fn new(lower: Level, higher: Level) -> Pair {
        Pair {
            lower: lower.get(),
            higher: higher.get(),
            wins: 0,
            draws: 0,
            losses: 0,
        }
    }

    pub fn record(&mut self, playout: Playout) {
        match playout {
            Playout::HigherWon => self.wins += 1,
            Playout::LowerWon => self.losses += 1,
            Playout::Drawn { .. } => self.draws += 1,
        }
    }

    pub fn games(&self) -> u32 {
        self.wins + self.draws + self.losses
    }

    /// The higher level's share of the points, draws worth half.
    pub fn score_rate(&self) -> f64 {
        let games = self.games();
        if games == 0 {
            return 0.0;
        }
        (self.wins as f64 + 0.5 * self.draws as f64) / games as f64
    }

    pub fn is_separated(&self) -> bool {
        self.games() > 0 && self.score_rate() + f64::EPSILON >= SEPARATION
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "L{:>2} vs L{:<2} {:>4} games {:>6.1}%  {}",
            self.lower,
            self.higher,
            self.games(),
            self.score_rate() * 100.0,
            if self.is_separated() {
                "separated"
            } else {
                "NOT SEPARATED"
            }
        )
    }
}

/// How a game ended, from the higher level's side.
#[derive(Debug, Clone, PartialEq)]
pub enum Playout {
    HigherWon,
    LowerWon,
    Drawn { why: String },
}

/// How hard each side is told to search, independent of its level. Both sides get the
/// same budget, so the difference being measured is the level and not the clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchConfig {
    /// Games per pair. Colours swap each game, so an even number balances.
    pub games: u32,
    /// `go depth`, for both sides.
    pub depth: u32,
    /// `go nodes`, for both sides. Non-zero wins over `depth`, and usually measures better:
    /// a node budget asks both sides the same question at any strength, while a fixed depth
    /// asks a strong engine nothing at all and the Elo cap never bites.
    pub nodes: u64,
    /// A game still going after this many plies is a draw.
    pub max_plies: u16,
    /// The seed the whole match replays from.
    pub seed: u64,
}

impl Default for MatchConfig {
    fn default() -> Self {
        MatchConfig {
            games: 2,
            depth: 12,
            nodes: 0,
            max_plies: 300,
            seed: 0x517a_2c00_0000_0001,
        }
    }
}

impl MatchConfig {
    /// What each move was told to spend, for the log line.
    pub fn budget(&self) -> String {
        if self.nodes > 0 {
            format!("{} nodes", self.nodes)
        } else {
            format!("depth {}", self.depth)
        }
    }

    /// The budget handed to the engine — the same for both sides, which is the whole point:
    /// the pair differs by its level and by nothing else.
    pub(crate) fn limits(&self) -> Limits {
        Limits {
            depth: (self.nodes == 0).then(|| self.depth.max(1)),
            nodes: (self.nodes > 0).then_some(self.nodes),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub enum LadderError {
    Engine(HostError),
    /// The engine reported a line whose first move the position refuses, or none at all.
    /// Either the engine is broken or the harness is asking about the wrong position;
    /// both are worth stopping for.
    NoPlayableLine {
        fen: String,
        infos: usize,
    },
}

impl fmt::Display for LadderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LadderError::Engine(why) => write!(f, "engine: {why}"),
            LadderError::NoPlayableLine { fen, infos } => {
                write!(f, "{infos} info line(s), none playable in {fen}")
            }
        }
    }
}

impl std::error::Error for LadderError {}

impl From<HostError> for LadderError {
    fn from(why: HostError) -> Self {
        LadderError::Engine(why)
    }
}

/// What the engine declared, in the shape the selection layer maps onto.
pub fn declared_options(declared: &[OptionDecl]) -> EngineOptions {
    let has = |name: &str| {
        declared
            .iter()
            .any(|decl| decl.name().eq_ignore_ascii_case(name))
    };
    let elo = declared.iter().find_map(|decl| match decl {
        OptionDecl::Spin { name, min, max, .. } if name.eq_ignore_ascii_case("UCI_Elo") => {
            Some(*min as i32..=*max as i32)
        }
        _ => None,
    });
    EngineOptions {
        multipv: has("MultiPV"),
        uci_elo: elo,
        uci_limit_strength: has("UCI_LimitStrength"),
        depth: has("Depth"),
        nodes: has("Nodes"),
    }
}

/// The report the sampler reads, from what the engine printed. A line whose first move
/// the position refuses is not a line — that is the whole reason this function needs the
/// board and `uci-host` deliberately does not have it.
pub fn report(infos: &[Info], board: &Board) -> Result<EngineReport, LadderError> {
    let mut lines: Vec<Line> = Vec::new();
    for info in infos {
        let (Some(first), Some(score)) = (info.pv.first(), info.score) else {
            continue;
        };
        let Ok(mv) = uci::parse(board, first) else {
            continue;
        };
        let score = match score {
            InfoScore::Cp(cp) => Score::Cp(cp),
            InfoScore::Mate(ply) => Score::Mate(ply),
        };
        match lines.iter_mut().find(|line| line.mv == mv) {
            Some(known) => known.score = score,
            None => lines.push(Line { mv, score }),
        }
    }
    let fen = fen::format(board);
    EngineReport::new(lines).ok_or(LadderError::NoPlayableLine {
        fen,
        infos: infos.len(),
    })
}

/// One side of a match: an engine, the level it is configured for, and the name the
/// selection record is filed under.
pub struct Contender {
    pub engine: Engine,
    pub level: Level,
    pub id: String,
    pub plan: move_select::Plan,
}

impl Contender {
    /// Start an engine and set it to a level: handshake, read what it declared, send the
    /// plan that level maps onto. The plan comes back because the player is told which
    /// mechanism is in use, and on an engine without `UCI_Elo` it is not the one the
    /// level table names.
    pub fn start(
        program: &Path,
        level: Level,
        log_path: Option<&Path>,
    ) -> Result<Contender, LadderError> {
        let mut engine = Engine::spawn(program, &[] as &[&str], log_path)?;
        let id = engine.handshake()?;
        let options = declared_options(engine.declared());
        let plan = options.plan_for(level);
        for setting in &plan.settings {
            // The declaration was the engine's own, so a refusal here is an engine bug
            // worth seeing in the log rather than a reason to stop the match.
            let _ = engine.set_option(setting.name, &setting.value);
        }
        Ok(Contender {
            engine,
            level,
            id: id.name,
            plan,
        })
    }
}

/// The report for the position a side is to play, and the move the level chose from it.
pub fn choose(
    contender: &mut Contender,
    board: &Board,
    moves: &[String],
    seed: u64,
    limits: &Limits,
) -> Result<move_select::Selection, LadderError> {
    contender
        .engine
        .set_position(Position::StartPos { moves })?;
    let searched = contender.engine.search(limits)?;
    let report = report(&searched.infos, board)?;
    Ok(move_select::select(
        &report,
        contender.level,
        seed,
        &contender.id,
    ))
}

/// One game between two configured contenders.
pub fn play_game(
    lower: &mut Contender,
    higher: &mut Contender,
    config: &MatchConfig,
    game: u32,
    higher_is_white: bool,
) -> Result<Playout, LadderError> {
    let mut board = Board::start();
    let mut moves: Vec<String> = Vec::new();
    let higher_colour = if higher_is_white {
        chess_core::board::Colour::White
    } else {
        chess_core::board::Colour::Black
    };

    for ply in 0..config.max_plies {
        let outcome = state::outcome(&board);
        if outcome != state::Outcome::Ongoing {
            return Ok(match outcome {
                state::Outcome::Checkmate { winner } => {
                    if winner == higher_colour {
                        Playout::HigherWon
                    } else {
                        Playout::LowerWon
                    }
                }
                other => Playout::Drawn {
                    why: format!("{other:?}"),
                },
            });
        }
        if board.halfmove() >= 100 {
            return Ok(Playout::Drawn {
                why: "fifty moves".into(),
            });
        }

        let higher_to_move = board.turn() == higher_colour;
        let contender = if higher_to_move {
            &mut *higher
        } else {
            &mut *lower
        };
        let seed = config.seed
            ^ ((game as u64) << 32)
            ^ ((ply as u64) << 1)
            ^ (higher_to_move as u64) << 40;
        let chosen = choose(contender, &board, &moves, seed, &config.limits())?;

        let Some(mv) = chosen.mv else {
            // A resignation or a draw offer is a decision, and it ends the game.
            if higher_to_move && chosen.reason == move_select::Reason::Resign {
                return Ok(Playout::LowerWon);
            }
            return Ok(Playout::Drawn {
                why: format!("{:?}", chosen.reason),
            });
        };
        board.apply(mv).map_err(|_| LadderError::NoPlayableLine {
            fen: fen::format(&board),
            infos: 1,
        })?;
        moves.push(uci::format(mv));
    }

    Ok(Playout::Drawn {
        why: "ply limit".into(),
    })
}

/// One pair of levels, `games` games with colours swapped. The two engines stay up for
/// the whole pair; a match over twenty levels is forty processes, not forty thousand.
pub fn play_pair(
    program: &Path,
    lower: Level,
    higher: Level,
    config: &MatchConfig,
    log_dir: Option<&Path>,
) -> Result<Pair, LadderError> {
    let log = |side: &str| -> Option<std::path::PathBuf> {
        log_dir.map(|dir| dir.join(format!("ladder-L{}-{}.log", higher.get(), side)))
    };
    let mut low = Contender::start(program, lower, log("lower").as_deref())?;
    let mut high = Contender::start(program, higher, log("higher").as_deref())?;
    let mut pair = Pair::new(lower, higher);
    for game in 0..config.games {
        let playout = play_game(&mut low, &mut high, config, game, game % 2 == 0)?;
        pair.record(playout);
    }
    let _ = low.engine.quit();
    let _ = high.engine.quit();
    Ok(pair)
}

/// Every adjacent pair from `levels`, in order.
pub fn pairs(levels: std::ops::RangeInclusive<u8>) -> Vec<(Level, Level)> {
    let levels: Vec<Level> = levels
        .filter_map(|level| Level::new(level as i32))
        .collect();
    levels
        .iter()
        .zip(levels.iter().skip(1))
        .map(|(lower, higher)| (*lower, *higher))
        .collect()
}

/// The whole ladder: each adjacent pair plays, and the run is a pass only if every pair
/// is separated.
pub fn run(
    program: &Path,
    levels: std::ops::RangeInclusive<u8>,
    config: &MatchConfig,
    log_dir: Option<&Path>,
    mut report_to: impl FnMut(&Pair),
) -> Result<Vec<Pair>, LadderError> {
    let mut results = Vec::new();
    for (lower, higher) in pairs(levels) {
        let pair = play_pair(program, lower, higher, config, log_dir)?;
        report_to(&pair);
        results.push(pair);
    }
    Ok(results)
}
