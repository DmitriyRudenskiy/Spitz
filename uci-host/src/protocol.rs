//! Reading what an engine says. Everything here is a pure function over a line of text,
//! which is what makes the tolerant-stream requirement testable without a process.

/// A score as the engine reported it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoScore {
    Cp(i32),
    Mate(i16),
}

/// One `info` line: what the engine found at one depth for one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub depth: u32,
    /// Which of the MultiPV lines this is, one for engines without MultiPV.
    pub multipv: u32,
    pub score: Option<InfoScore>,
    pub nodes: u64,
    pub time_ms: u64,
    /// The principal variation in the engine's own long algebraic. This layer does not
    /// turn those into moves: only the position the search started from can say whether
    /// `e2e4` was legal, and this layer does not have it.
    pub pv: Vec<String>,
}

/// What the engine declared about itself and its options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionDecl {
    Spin {
        name: String,
        min: i64,
        max: i64,
        default: i64,
    },
    Str {
        name: String,
        default: String,
    },
    Check {
        name: String,
        default: bool,
    },
    Combo {
        name: String,
        variants: Vec<String>,
        default: String,
    },
    Button {
        name: String,
    },
}

impl OptionDecl {
    pub fn name(&self) -> &str {
        match self {
            OptionDecl::Spin { name, .. }
            | OptionDecl::Str { name, .. }
            | OptionDecl::Check { name, .. }
            | OptionDecl::Combo { name, .. }
            | OptionDecl::Button { name } => name,
        }
    }

    /// Whether this value may be sent for this option at all. Rejected here, before the
    /// engine sees it, because an engine's reaction to a nonsense Spin ranges from
    /// clamping it silently to quitting.
    pub fn accepts(&self, value: &str) -> bool {
        match self {
            OptionDecl::Spin { min, max, .. } => match value.parse::<i64>() {
                Ok(n) => (*min..=*max).contains(&n),
                Err(_) => false,
            },
            OptionDecl::Check { .. } => matches!(value, "true" | "false"),
            OptionDecl::Combo { variants, .. } => variants.iter().any(|v| v == value),
            OptionDecl::Str { .. } => true,
            // A button takes no value; sending one is a mistake we can catch.
            OptionDecl::Button { .. } => false,
        }
    }
}

/// Anything an engine can say to us.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The engine's own name, from its id name line.
    IdName(String),
    /// The engine's author, which arrives as a line of its own.
    IdAuthor(String),
    /// One option the engine declares it has.
    Option(OptionDecl),
    Info(Info),
    BestMove {
        /// `None` for the legal `bestmove (none)`, which says the engine has no move.
        bestmove: Option<String>,
        ponder: Option<String>,
    },
    Ready,
    UciOk,
    /// Anything else — a banner, a `bestmove` we cannot parse, a line from an engine that
    /// has opinions. It reaches the session log and nothing else.
    Line(String),
    /// The engine closed its output. Whatever it was, it is no longer answering.
    Exited,
}

fn number(words: &[&str], at: usize) -> Option<i64> {
    words.get(at).and_then(|w| w.parse().ok())
}

fn find(words: &[&str], word: &str) -> Option<usize> {
    words.iter().position(|w| *w == word)
}

/// Parse an `info` line. `None` means it was not a line worth an event — an `info string`
/// note, a truncated score, an engine that prints its debug log this way.
pub fn parse_info(line: &str) -> Option<Info> {
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.first() != Some(&"info") {
        return None;
    }
    let mut info = Info {
        depth: 0,
        multipv: 1,
        score: None,
        nodes: 0,
        time_ms: 0,
        pv: Vec::new(),
    };
    let mut recognised = 0usize;
    let mut i = 1;
    while i < words.len() {
        match words[i] {
            "depth" => {
                info.depth = number(&words, i + 1)?.try_into().unwrap_or(0);
                recognised += 1;
                i += 2;
            }
            "multipv" => {
                info.multipv = number(&words, i + 1)?.try_into().unwrap_or(1).max(1);
                recognised += 1;
                i += 2;
            }
            "score" => {
                match (words.get(i + 1), words.get(i + 2)) {
                    (Some(&"cp"), Some(v)) => info.score = v.parse::<i32>().ok().map(InfoScore::Cp),
                    (Some(&"mate"), Some(v)) => {
                        info.score = v.parse::<i16>().ok().map(InfoScore::Mate)
                    }
                    _ => return None,
                }
                recognised += 1;
                i += 3;
            }
            "nodes" => {
                info.nodes = number(&words, i + 1)?.try_into().unwrap_or(0);
                recognised += 1;
                i += 2;
            }
            "time" => {
                info.time_ms = number(&words, i + 1)?.try_into().unwrap_or(0);
                recognised += 1;
                i += 2;
            }
            "pv" => {
                info.pv = words[i + 1..].iter().map(|w| w.to_string()).collect();
                recognised += 1;
                break;
            }
            // nps, hashfull, seldepth, wdl, string: real fields this layer has no use
            // for. Skipping the key is enough; the line still reaches the log.
            _ => i += 1,
        }
    }
    // A pv is what makes an info line worth waking anyone up for; `info string hello` is
    // commentary and stays in the log.
    if recognised > 0 && !info.pv.is_empty() {
        Some(info)
    } else {
        None
    }
}

/// Parse an `option` line. `None` for one we cannot make sense of, which is logged and
/// otherwise ignored — an engine with a broken declaration is still usable.
///
/// ponytail: a `string` default containing spaces is read as its first word. Only the
/// path-like options are affected, none of which this app sets.
pub fn parse_option(line: &str) -> Option<OptionDecl> {
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.first() != Some(&"option") {
        return None;
    }
    let name_at = find(&words, "name")? + 1;
    let type_at = find(&words, "type")?;
    if name_at > type_at {
        return None;
    }
    let name = words[name_at..type_at].join(" ");
    let kind = *words.get(type_at + 1)?;
    let rest = &words[type_at + 2..];

    let mut default: Option<&str> = None;
    let mut min: Option<i64> = None;
    let mut max: Option<i64> = None;
    let mut variants = Vec::new();
    let mut i = 0;
    while i + 1 < rest.len() {
        match rest[i] {
            "default" => default = Some(rest[i + 1]),
            "min" => min = rest[i + 1].parse().ok(),
            "max" => max = rest[i + 1].parse().ok(),
            "var" => variants.push(rest[i + 1].to_string()),
            _ => {}
        }
        i += 2;
    }

    match kind {
        "spin" => Some(OptionDecl::Spin {
            name,
            min: min?,
            max: max?,
            default: default?.parse().ok()?,
        }),
        "check" => Some(OptionDecl::Check {
            name,
            default: default == Some("true"),
        }),
        "combo" => Some(OptionDecl::Combo {
            name,
            variants,
            default: default.unwrap_or("").to_string(),
        }),
        "button" => Some(OptionDecl::Button { name }),
        "string" => Some(OptionDecl::Str {
            name,
            default: default.unwrap_or("").to_string(),
        }),
        _ => None,
    }
}

/// One line of engine output, classified.
pub fn parse_line(line: &str) -> Event {
    let trimmed = line.trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    match words.first().copied().unwrap_or("") {
        "id" => {
            let value = trimmed.split_once(' ').map_or("", |(_, rest)| rest);
            if let Some(rest) = value.strip_prefix("name ") {
                Event::IdName(rest.to_string())
            } else if let Some(rest) = value.strip_prefix("author ") {
                Event::IdAuthor(rest.to_string())
            } else {
                Event::Line(line.to_string())
            }
        }
        "option" => match parse_option(trimmed) {
            Some(decl) => Event::Option(decl),
            None => Event::Line(line.to_string()),
        },
        "uciok" => Event::UciOk,
        "readyok" => Event::Ready,
        "bestmove" => match words.get(1) {
            Some(&"(none)") | None => Event::BestMove {
                bestmove: None,
                ponder: None,
            },
            Some(bestmove) => Event::BestMove {
                bestmove: Some(bestmove.to_string()),
                ponder: words
                    .iter()
                    .position(|w| *w == "ponder")
                    .and_then(|p| words.get(p + 1))
                    .map(|w| w.to_string()),
            },
        },
        "info" => parse_info(trimmed)
            .map(Event::Info)
            .unwrap_or(Event::Line(line.to_string())),
        _ => Event::Line(line.to_string()),
    }
}
