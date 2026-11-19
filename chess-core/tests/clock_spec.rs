//! The spec's game-clock requirements: the four controls, and the flag that falls the instant it
//! reaches zero.

use std::time::Duration;

use chess_core::board::Colour;
use chess_core::clock::{Clock, Control, Event};

const fn seconds(n: u64) -> Duration {
    Duration::from_secs(n)
}

#[test]
fn the_increment_arrives_only_after_a_completed_move() {
    let mut clock = Clock::new(Control::Increment {
        base: seconds(300),
        increment: seconds(10),
    });

    // Thinking for twelve seconds shows the bank going down and credits nothing.
    assert_eq!(
        clock.time_taken(Colour::White, seconds(12)),
        Event::Running {
            spent: seconds(12),
            remaining: seconds(288)
        }
    );
    assert_eq!(clock.remaining(Colour::White), Some(seconds(300)));

    // The move being completed is where the top-up happens.
    assert_eq!(
        clock.complete_move(Colour::White, seconds(12)),
        Event::Running {
            spent: seconds(12),
            remaining: seconds(298)
        }
    );
    assert_eq!(clock.remaining(Colour::White), Some(seconds(298)));
    assert_eq!(clock.turn(), Colour::Black);

    // A move that flags does not collect its increment first.
    let mut clock = Clock::new(Control::Increment {
        base: seconds(10),
        increment: seconds(30),
    });
    assert_eq!(
        clock.complete_move(Colour::White, seconds(10)),
        Event::FlagFell {
            colour: Colour::White,
            spent: seconds(10)
        }
    );
    assert_eq!(clock.fallen(), Some(Colour::White));
}

#[test]
fn the_delay_charges_only_the_time_past_it() {
    let mut clock = Clock::new(Control::Delay {
        base: seconds(300),
        delay: seconds(5),
    });

    // Three seconds is inside the grace, so the bank is untouched.
    assert_eq!(
        clock.complete_move(Colour::White, seconds(3)),
        Event::Running {
            spent: seconds(3),
            remaining: seconds(300)
        }
    );
    // Nine seconds is four past the grace, and four is what goes.
    assert_eq!(
        clock.complete_move(Colour::Black, seconds(9)),
        Event::Running {
            spent: seconds(9),
            remaining: seconds(296)
        }
    );
    // Exactly on the grace is still inside it.
    assert_eq!(
        clock.complete_move(Colour::White, seconds(5)),
        Event::Running {
            spent: seconds(5),
            remaining: seconds(300)
        }
    );
}

#[test]
fn a_flag_fall_names_the_player_and_the_time_state() {
    let mut clock = Clock::new(Control::SuddenDeath { base: seconds(10) });

    // The instant is the moment the time reaches zero, not the moment a move is finally played.
    assert_eq!(
        clock.time_taken(Colour::Black, seconds(12)),
        Event::FlagFell {
            colour: Colour::Black,
            spent: seconds(12)
        }
    );
    assert_eq!(
        clock.fallen(),
        None,
        "looking at a clock does not end a game"
    );

    assert_eq!(
        clock.complete_move(Colour::Black, seconds(12)),
        Event::FlagFell {
            colour: Colour::Black,
            spent: seconds(12)
        }
    );
    assert_eq!(clock.fallen(), Some(Colour::Black));
    assert_eq!(clock.remaining(Colour::Black), Some(Duration::ZERO));
    assert_eq!(clock.remaining(Colour::White), Some(seconds(10)));

    // Whatever is tried after the flag is reported as the same flag, and changes no clock.
    assert_eq!(
        clock.complete_move(Colour::White, seconds(1)),
        Event::FlagFell {
            colour: Colour::Black,
            spent: seconds(1)
        }
    );
    assert_eq!(clock.remaining(Colour::White), Some(seconds(10)));
    // A flag ends the game where it found it: white was to move when black's flag fell, and still
    // is.
    assert_eq!(clock.turn(), Colour::White);
}

#[test]
fn time_used_up_exactly_is_a_flag() {
    let mut clock = Clock::new(Control::SuddenDeath { base: seconds(10) });
    assert_eq!(
        clock.complete_move(Colour::White, seconds(10)),
        Event::FlagFell {
            colour: Colour::White,
            spent: seconds(10)
        }
    );
    assert_eq!(clock.remaining(Colour::White), Some(Duration::ZERO));
}

#[test]
fn a_game_without_a_clock_still_has_moves() {
    let mut clock = Clock::new(Control::None);
    assert_eq!(clock.remaining(Colour::White), None);
    assert_eq!(
        clock.complete_move(Colour::White, seconds(40)),
        Event::NoClock
    );
    assert_eq!(clock.turn(), Colour::Black);
    assert_eq!(clock.fallen(), None);
    assert_eq!(clock.time_taken(Colour::Black, seconds(1)), Event::NoClock);
}

#[test]
fn a_resumed_clock_carries_where_the_game_got_to() {
    let mut clock = Clock::resumed(
        Control::Increment {
            base: seconds(300),
            increment: seconds(30),
        },
        seconds(41),
        seconds(19),
        Colour::Black,
    );
    assert_eq!(clock.turn(), Colour::Black);
    assert_eq!(
        clock.complete_move(Colour::Black, seconds(9)),
        Event::Running {
            spent: seconds(9),
            remaining: seconds(40)
        }
    );
    assert_eq!(clock.turn(), Colour::White);
}
