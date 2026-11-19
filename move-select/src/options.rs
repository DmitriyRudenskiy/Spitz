//! Turning a strength level into the options an engine will honour.
//!
//! The engine declares its options at handshake; this maps a level onto the ones it
//! actually declared and says which mechanism ended up in use, because "level 12" means
//! something different on an engine that cannot limit its Elo and the player is entitled
//! to be told which one they are playing.

use crate::Level;

/// How far the engine will let us shape its strength, as far as this layer cares. Built
/// from the handshake, not from a list of engines this crate has to keep up with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineOptions {
    /// `MultiPV` — without it the engine reports one line and there is nothing to sample.
    pub multipv: bool,
    /// `UCI_Elo`, with the range the engine accepts.
    pub uci_elo: Option<std::ops::RangeInclusive<i32>>,
    /// `UCI_LimitStrength` — the switch that makes `UCI_Elo` mean anything.
    pub uci_limit_strength: bool,
    /// `Depth`.
    pub depth: bool,
    /// `Nodes`.
    pub nodes: bool,
}

/// An `option name value` line for the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting {
    pub name: &'static str,
    pub value: String,
}

impl Setting {
    fn new(name: &'static str, value: impl Into<String>) -> Self {
        Setting {
            name,
            value: value.into(),
        }
    }
}

/// What the level is being realized by. The UI names this, so a player on an engine
/// without Elo limiting is not told they are playing "1400".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mechanism {
    /// The engine caps itself at `elo`, which is the mechanism the ladder calibrates
    /// against when the engine supports it.
    EloCapped { elo: i32 },
    /// No Elo limiting, so the level is a search budget and the sampling width.
    DepthAndNodes { depth: u32, nodes: u64 },
    /// Nothing the engine declared can slow it down: the level is sampling alone, which
    /// is the weakest thing this layer can do and worth showing the player.
    SamplingOnly,
}

/// What to send the engine for one level, and what it means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub settings: Vec<Setting>,
    /// How many lines to ask for. One when the engine has no `MultiPV`, which leaves the
    /// sampler with the engine's own first line and nothing else.
    pub multipv: u32,
    pub mechanism: Mechanism,
}

impl EngineOptions {
    /// The strongest applicable set of declared options for this level: Elo limiting when
    /// the engine offers it, otherwise the search caps, and `MultiPV` width either way so
    /// the sampler has lines to choose from.
    pub fn plan_for(&self, level: Level) -> Plan {
        let mut settings = Vec::new();
        let multipv = if self.multipv {
            let width = level.multipv_width();
            settings.push(Setting::new("MultiPV", width.to_string()));
            width
        } else {
            1
        };

        let mechanism = if let Some(range) = self.uci_elo.clone() {
            if self.uci_limit_strength {
                // An engine that refuses 3200 should not be asked for it; the level is
                // then as strong as the engine allows, and the mechanism says so.
                let elo = level.elo().clamp(*range.start(), *range.end());
                settings.push(Setting::new("UCI_LimitStrength", "true"));
                settings.push(Setting::new("UCI_Elo", elo.to_string()));
                Mechanism::EloCapped { elo }
            } else {
                self.search_caps(level, &mut settings)
            }
        } else {
            self.search_caps(level, &mut settings)
        };

        Plan {
            settings,
            multipv,
            mechanism,
        }
    }

    fn search_caps(&self, level: Level, settings: &mut Vec<Setting>) -> Mechanism {
        if self.depth {
            settings.push(Setting::new("Depth", level.depth().to_string()));
        }
        if self.nodes {
            settings.push(Setting::new("Nodes", level.nodes().to_string()));
        }
        if self.depth || self.nodes {
            Mechanism::DepthAndNodes {
                depth: level.depth(),
                nodes: level.nodes(),
            }
        } else {
            Mechanism::SamplingOnly
        }
    }
}
