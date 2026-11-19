//! The clock a game runs on.
//!
//! A clock is a rule about how spent time becomes remaining time, and the rule differs enough
//! between the controls that the control is an enum rather than a bag of optional numbers: an
//! increment arrives after a move is completed, a delay is a grace that only the time past it is
//! charged for, and sudden death charges everything. `Control::None` is a real control and not a
//! missing clock — a casual game, or a move list being reviewed, has no clock to show.
//!
//! The caller owns the stopwatch. Both entry points are told how long the move has taken *so far*,
//! which keeps the arithmetic in one place and makes the display path harmless to call twice:
//! [`time_taken`] answers what the clock would show without committing anything, and
//! [`complete_move`] commits the same number, credits what the control credits, and passes the turn.

use std::fmt;
use std::time::Duration;

use crate::board::Colour;

/// How spent time becomes remaining time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Control {
    /// No clock. Nothing runs, nothing falls.
    None,
    /// One bank of time, and when it is gone the game is over.
    SuddenDeath { base: Duration },
    /// One bank of time, topped up after every completed move.
    Increment { base: Duration, increment: Duration },
    /// One bank of time, and a grace period at the start of each move that is only charged for
    /// the time spent past it.
    Delay { base: Duration, delay: Duration },
}

impl Control {
    /// The time each player starts with, or `None` when there is no clock to start.
    pub fn base(&self) -> Option<Duration> {
        match self {
            Control::None => None,
            Control::SuddenDeath { base } => Some(*base),
            Control::Increment { base, .. } => Some(*base),
            Control::Delay { base, .. } => Some(*base),
        }
    }
}

/// What the clock says about time that has been spent.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    /// The clock is still running. `remaining` is what the player has left if the move ends now.
    Running {
        spent: Duration,
        remaining: Duration,
    },
    /// The player's time reached zero, which ends the game whoever notices it.
    FlagFell { colour: Colour, spent: Duration },
    /// There is no clock.
    NoClock,
}

impl Event {
    /// The remaining time the event reports, when there is a clock to report it from.
    pub fn remaining(&self) -> Option<Duration> {
        match self {
            Event::NoClock => None,
            Event::Running { remaining, .. } => Some(*remaining),
            Event::FlagFell { .. } => Some(Duration::ZERO),
        }
    }
}

/// The two clocks of one game, and whose move it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Clock {
    control: Control,
    white: Duration,
    black: Duration,
    turn: Colour,
    /// The player whose flag has fallen, which ends the clock: nothing after it is counted.
    fallen: Option<Colour>,
}

impl Clock {
    /// A clock at the start of a game. The control's own base is what both players get; the times
    /// a caller brings are for resuming, and are read through [`Clock::set_remaining`].
    pub fn new(control: Control) -> Clock {
        let base = control.base().unwrap_or(Duration::ZERO);
        Clock {
            control,
            white: base,
            black: base,
            turn: Colour::White,
            fallen: None,
        }
    }

    /// A clock part-way through a game, as a saved game leaves it: the banks the players have left,
    /// and whoever is to move.
    pub fn resumed(control: Control, white: Duration, black: Duration, turn: Colour) -> Clock {
        Clock {
            control,
            white,
            black,
            turn,
            fallen: None,
        }
    }

    pub const fn control(&self) -> Control {
        self.control
    }

    pub const fn turn(&self) -> Colour {
        self.turn
    }

    /// What a player has left right now, or `None` when the game has no clock.
    pub fn remaining(&self, colour: Colour) -> Option<Duration> {
        match self.control {
            Control::None => None,
            _ => Some(match colour {
                Colour::White => self.white,
                Colour::Black => self.black,
            }),
        }
    }

    /// The player whose flag has fallen, which is the game over.
    pub const fn fallen(&self) -> Option<Colour> {
        self.fallen
    }

    /// What the clock would show if the move in progress took `spent`. Changes nothing, so a
    /// display may ask it as often as it redraws.
    pub fn time_taken(&self, colour: Colour, spent: Duration) -> Event {
        self.answer(colour, spent, false)
    }

    /// The move in progress took `spent`, and is now played: charge the player, credit what the
    /// control credits, and pass the turn. A move after a flag has fallen changes nothing.
    pub fn complete_move(&mut self, colour: Colour, spent: Duration) -> Event {
        let event = self.answer(colour, spent, true);
        if let Event::FlagFell { colour, .. } = event {
            self.set_remaining_to(colour, Duration::ZERO);
            self.fallen = Some(colour);
            return event;
        }
        if let Event::Running { remaining, .. } = event {
            self.set_remaining_to(colour, remaining);
        }
        self.turn = colour.other();
        event
    }

    /// The arithmetic, once: what the control does to this player's bank when `spent` goes by.
    /// `completed` says whether the move is being played, which is the difference between showing
    /// a top-up and getting one.
    fn answer(&self, colour: Colour, spent: Duration, completed: bool) -> Event {
        if let Some(fallen) = self.fallen {
            // The game ended at the flag. Anything after it is not played on a clock.
            return Event::FlagFell {
                colour: fallen,
                spent,
            };
        }
        let Some(remaining) = self.remaining(colour) else {
            return Event::NoClock;
        };
        let left = match self.control {
            Control::None => return Event::NoClock,
            Control::SuddenDeath { .. } => remaining.saturating_sub(spent),
            // The grace is not a bank to spend from: only the time past it is charged.
            Control::Delay { delay, .. } => remaining.saturating_sub(spent.saturating_sub(delay)),
            Control::Increment { increment, .. } => {
                let left = remaining.saturating_sub(spent);
                if left.is_zero() {
                    // A move that flags does not collect its increment first.
                    return Event::FlagFell { colour, spent };
                }
                if completed {
                    left + increment
                } else {
                    left
                }
            }
        };
        if left.is_zero() {
            Event::FlagFell { colour, spent }
        } else {
            Event::Running {
                spent,
                remaining: left,
            }
        }
    }

    fn set_remaining_to(&mut self, colour: Colour, left: Duration) {
        match colour {
            Colour::White => self.white = left,
            Colour::Black => self.black = left,
        }
    }
}

impl fmt::Display for Control {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Control::None => write!(out, "no clock"),
            Control::SuddenDeath { base } => write!(out, "{}", minutes(base)),
            Control::Increment { base, increment } => {
                write!(out, "{} + {}", minutes(base), seconds(increment))
            }
            Control::Delay { base, delay } => {
                write!(out, "{} with {} delay", minutes(base), seconds(delay))
            }
        }
    }
}

/// The name a player recognises for a bank of time: whole minutes when it divides, seconds when
/// it does not.
fn minutes(base: &Duration) -> String {
    let whole = base.as_secs();
    if whole > 0 && whole.is_multiple_of(60) {
        format!("{} min", whole / 60)
    } else {
        seconds(base)
    }
}

fn seconds(time: &Duration) -> String {
    if time.subsec_nanos() == 0 {
        format!("{} s", time.as_secs())
    } else {
        format!("{time:?}")
    }
}
