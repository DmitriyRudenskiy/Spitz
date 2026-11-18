//! The cross-table: who met whom, what they scored, and what it says about the placing.
//!
//! The table is data; the plain-text, HTML and typeset forms in `crosstab_render` are renderings of
//! one model, which is why they cannot disagree. The thresholds that decide a mode and the caps that
//! decide what got left out are printed in the report itself.

use crate::rating_list::RatingList;
use chess_core::date::Date;
use chess_core::pgn::lexer::GameResult;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Players a table is willing to hold, and rounds it will span. Beyond them the report says what it
/// left out rather than quietly shrinking.
pub const MAX_PLAYERS: usize = 500;
pub const MAX_ROUNDS: usize = 60;

/// A score in quarter points, so a result is an integer and every renderer prints the same thing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Score(i32);

pub const WIN: Score = Score(4);
pub const DRAW: Score = Score(2);
pub const LOSS: Score = Score(0);

impl Score {
    /// "4", "3.5", and for a tie-break "2.25" — a quarter of a point is exact, so the text is too.
    pub fn text(self) -> String {
        let whole = self.0 / 4;
        match self.0 % 4 {
            0 => whole.to_string(),
            1 => format!("{whole}.25"),
            2 => format!("{whole}.5"),
            _ => format!("{whole}.75"),
        }
    }
}

impl std::fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text())
    }
}

/// A tie-break value, in the same quarter points as a score, for the renderers to print.
pub fn tie_break_text(value: i64) -> String {
    Score(value as i32).text()
}

/// One game, as the table needs it. `row` is the library row, so a cell can be clicked.
#[derive(Clone, Copy, Debug)]
pub struct GameRow {
    pub white: u32,
    pub black: u32,
    /// Zero when the game does not say.
    pub white_rating: u16,
    pub black_rating: u16,
    /// Negative when the game has no round.
    pub round: i32,
    pub date: Date,
    pub result: GameResult,
    pub row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Automatic,
    AllPlayAll,
    Swiss,
    Knockout,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Automatic => "automatic",
            Mode::AllPlayAll => "all-play-all",
            Mode::Swiss => "swiss",
            Mode::Knockout => "knockout",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Rating,
    Score,
    Federation,
}

/// What the caller asked for.
#[derive(Clone, Copy, Debug)]
pub struct Options<'a> {
    pub mode: Mode,
    pub sort: SortColumn,
    /// Title, federation and age come from here when the user supplied one.
    pub info: Option<&'a RatingList>,
    pub show_age: bool,
    pub max_players: usize,
    pub max_rounds: usize,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            mode: Mode::Automatic,
            sort: SortColumn::Name,
            info: None,
            show_age: false,
            max_players: MAX_PLAYERS,
            max_rounds: MAX_ROUNDS,
        }
    }
}

/// What one player scored against one other, game by game rather than as a total, because the
/// tie-break needs the result and not the sum.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct Pair {
    wins: u16,
    draws: u16,
    losses: u16,
}

impl Pair {
    fn games(&self) -> usize {
        (self.wins + self.draws + self.losses) as usize
    }
    fn score(&self) -> Score {
        Score(self.wins as i32 * 4 + self.draws as i32 * 2)
    }
}

/// One player's row.
#[derive(Clone, Debug)]
pub struct PlayerRow {
    pub name: String,
    pub rating: Option<u16>,
    pub title: Option<String>,
    pub federation: Option<String>,
    pub age: Option<u8>,
    pub games: usize,
    pub wins: usize,
    pub draws: usize,
    pub losses: usize,
    pub score: Score,
    pub average_opponent_rating: Option<u16>,
    pub tie_break: Option<i64>,
    /// The last rule that actually separated this row from the one above it in the placing.
    pub decided_by: &'static str,
    /// Where the score and tie-breaks put them, one for the first.
    pub rank: usize,
    /// One cell per opponent column: the score against that player, or `None` for no game. The
    /// diagonal is the player themself and is rendered as not applicable.
    pub cells: Vec<Option<Score>>,
}

/// A game between two players of the table, for the drill-down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clash {
    pub row: usize,
    pub white: usize,
    pub black: usize,
    pub result: GameResult,
    pub round: i32,
}

/// The report.
#[derive(Clone, Debug)]
pub struct Crosstab {
    /// In display order, which is the sort the user chose.
    pub players: Vec<PlayerRow>,
    /// The opponent columns: as many as there are players, the diagonal included.
    pub columns: usize,
    pub mode: Mode,
    /// The rule that picked the mode, for the header.
    pub mode_reason: String,
    pub games: usize,
    pub excluded_players: usize,
    pub excluded_games: usize,
    pub exclusion_rule: &'static str,
    clashes: Vec<Vec<Vec<Clash>>>,
    pairs: Vec<Vec<Pair>>,
}

impl Crosstab {
    /// The games one player played against another, oldest first.
    pub fn clash(&self, a: usize, b: usize) -> &[Clash] {
        self.clashes.get(a).and_then(|row| row.get(b)).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The score a player made against another, for the cell text.
    pub fn pair_score(&self, a: usize, b: usize) -> Option<Score> {
        self.pairs.get(a).and_then(|row| row.get(b)).filter(|pair| pair.games() > 0).map(Pair::score)
    }

    /// A player's own column, which the renderers mark as not applicable.
    pub fn is_diagonal(&self, a: usize, b: usize) -> bool {
        a == b
    }
}

/// Build the table over a filtered game set. `name_of` resolves a name id, and is only called for
/// players who end up in the table.
pub fn build<'a>(
    games: &[GameRow],
    name_of: &impl Fn(u32) -> Option<&'a str>,
    options: &Options,
) -> Crosstab {
    let max_players = options.max_players.max(1);
    let max_rounds = options.max_rounds.max(1);

    // Round cap first: a set spanning more rounds than the table can show keeps the earliest.
    let mut rounds: Vec<i32> = games.iter().map(|game| game.round).filter(|round| *round >= 0).collect();
    rounds.sort_unstable();
    rounds.dedup();
    let round_cut = if rounds.len() <= max_rounds { i32::MAX } else { rounds[max_rounds - 1] };
    let mut kept: Vec<&GameRow> = Vec::with_capacity(games.len());
    let mut excluded_games = 0;
    for game in games {
        if game.round > round_cut {
            excluded_games += 1;
        } else {
            kept.push(game);
        }
    }

    // Player cap: the busiest players stay, because a cross-table is about the players who played.
    let mut games_per_player: HashMap<u32, usize> = HashMap::new();
    for game in &kept {
        *games_per_player.entry(game.white).or_default() += 1;
        *games_per_player.entry(game.black).or_default() += 1;
    }
    let name = |id: u32| name_of(id).unwrap_or("");
    let mut ids: Vec<u32> = games_per_player.keys().copied().collect();
    let mut excluded_players = 0;
    let exclusion_rule = if ids.len() > max_players {
        ids.sort_by(|a, b| games_per_player[b].cmp(&games_per_player[a]).then_with(|| name(*a).cmp(name(*b))));
        let dropped: HashSet<u32> = ids.split_off(max_players).into_iter().collect();
        let before = kept.len();
        kept.retain(|game| !dropped.contains(&game.white) && !dropped.contains(&game.black));
        excluded_games += before - kept.len();
        excluded_players = dropped.len();
        ids.sort_by(|a, b| name(*a).cmp(name(*b)));
        "players are capped at the busiest; the rest and their games are excluded"
    } else {
        ids.sort_by(|a, b| name(*a).cmp(name(*b)));
        "no exclusion was needed"
    };

    let mut index: HashMap<u32, usize> = HashMap::with_capacity(ids.len());
    for (at, id) in ids.iter().enumerate() {
        index.insert(*id, at);
    }
    let count = ids.len();

    let mut rows: Vec<PlayerRow> = ids
        .iter()
        .map(|id| {
            let name = name(*id).to_owned();
            let listed = options.info.and_then(|list| list.player(&name));
            PlayerRow {
                title: listed.and_then(|p| p.title.clone()),
                federation: listed.and_then(|p| p.federation.clone()),
                rating: listed.and_then(|p| p.rating),
                name,
                age: None,
                games: 0,
                wins: 0,
                draws: 0,
                losses: 0,
                score: Score::default(),
                average_opponent_rating: None,
                tie_break: None,
                decided_by: "score",
                rank: 0,
                cells: vec![None; count],
            }
        })
        .collect();

    let mut pairs = vec![vec![Pair::default(); count]; count];
    let mut clashes = vec![vec![Vec::new(); count]; count];
    let mut opponent_rating_sum: Vec<u64> = vec![0; count];
    let mut opponent_rating_games: Vec<usize> = vec![0; count];
    let mut best_rating: Vec<u16> = vec![0; count];
    let mut latest: Option<Date> = None;

    for game in &kept {
        let (Some(&white), Some(&black)) = (index.get(&game.white), index.get(&game.black)) else {
            continue;
        };
        let (white_score, black_score) = match game.result {
            GameResult::WhiteWins => (WIN, LOSS),
            GameResult::BlackWins => (LOSS, WIN),
            GameResult::Draw => (DRAW, DRAW),
            GameResult::Unfinished => continue,
        };
        for (at, opponent, score, opponent_rating) in [
            (white, black, white_score, game.black_rating),
            (black, white, black_score, game.white_rating),
        ] {
            let row = &mut rows[at];
            row.games += 1;
            row.score = Score(row.score.0 + score.0);
            match score {
                _ if score == WIN => row.wins += 1,
                _ if score == DRAW => row.draws += 1,
                _ => row.losses += 1,
            }
            let pair = &mut pairs[at][opponent];
            if score == WIN {
                pair.wins += 1;
            } else if score == DRAW {
                pair.draws += 1;
            } else {
                pair.losses += 1;
            }
            if opponent_rating > 0 {
                opponent_rating_sum[at] += opponent_rating as u64;
                opponent_rating_games[at] += 1;
            }
        }
        let clash = Clash { row: game.row, white, black, result: game.result, round: game.round };
        clashes[white][black].push(clash);
        clashes[black][white].push(clash);
        best_rating[white] = best_rating[white].max(game.white_rating);
        best_rating[black] = best_rating[black].max(game.black_rating);
        if !game.date.is_unknown() && latest.is_none_or(|current| game.date.sort_key() > current.sort_key()) {
            latest = Some(game.date);
        }
    }

    for (at, row) in rows.iter_mut().enumerate() {
        row.average_opponent_rating = (opponent_rating_games[at] > 0)
            .then(|| (opponent_rating_sum[at] / opponent_rating_games[at] as u64) as u16);
        // A rating the game itself carries beats the list's.
        row.rating = (best_rating[at] > 0).then_some(best_rating[at]).or(row.rating);
        for (b, pair) in pairs[at].iter().enumerate() {
            if pair.games() > 0 {
                row.cells[b] = Some(pair.score());
            }
        }
        if options.show_age {
            row.age = latest.and_then(|date| options.info.and_then(|list| list.age_at(&row.name, date)));
        }
    }

    let (mode, mode_reason) = detect_mode(options.mode, count, kept.len());
    apply_tie_breaks(&mut rows, &pairs, mode);

    let mut ranked: Vec<usize> = (0..count).collect();
    ranked.sort_by(|a, b| rank_order(&rows, &pairs, *a, *b));
    for (place, at) in ranked.iter().enumerate() {
        rows[*at].rank = place + 1;
        if place > 0 {
            rows[*at].decided_by = decided_by(&rows, &pairs, ranked[place - 1], *at);
        }
    }

    sort_for_display(&mut rows, options.sort);

    Crosstab {
        columns: count,
        mode,
        mode_reason,
        games: kept.len(),
        excluded_players,
        excluded_games,
        exclusion_rule,
        players: rows,
        clashes,
        pairs,
    }
}

/// The documented thresholds: more than 30 players, fewer than 5 games a player, or fewer than half
/// the games of a complete round robin means Swiss.
fn detect_mode(wanted: Mode, players: usize, games: usize) -> (Mode, String) {
    if wanted != Mode::Automatic {
        return (wanted, "chosen by the user".to_owned());
    }
    let complete = players.saturating_sub(1) * players / 2;
    let per_player = players.checked_div(1).map_or(0, |_| games * 2 / players.max(1));
    if players > 30 {
        (Mode::Swiss, format!("{players} players is more than 30"))
    } else if per_player < 5 {
        (Mode::Swiss, format!("{per_player} games per player is fewer than 5"))
    } else if games < complete / 2 {
        (Mode::Swiss, format!("{games} games is fewer than half the {complete} of a round robin"))
    } else {
        (Mode::AllPlayAll, format!("{players} players playing {games} games looks like a round robin"))
    }
}

/// Sonnenborn-Berger for an all-play-all, Buchholz for a Swiss, nothing for a knockout.
fn apply_tie_breaks(rows: &mut [PlayerRow], pairs: &[Vec<Pair>], mode: Mode) {
    if matches!(mode, Mode::Knockout | Mode::Automatic) {
        return;
    }
    for a in 0..rows.len() {
        let mut sum = 0i64;
        for b in 0..rows.len() {
            if a == b {
                continue;
            }
            let pair = pairs[a][b];
            if pair.games() == 0 {
                continue;
            }
            let opponent = rows[b].score.0 as i64;
            sum += match mode {
                // Their whole score for a win, half for a draw, nothing for a loss.
                Mode::AllPlayAll => pair.wins as i64 * opponent + pair.draws as i64 * opponent / 2,
                // Their score once per game played, whoever won it.
                Mode::Swiss => pair.games() as i64 * opponent,
                _ => 0,
            };
        }
        rows[a].tie_break = Some(sum);
    }
}

/// The rule that put `above` over `below`, in the order the rules were tried.
fn decided_by(rows: &[PlayerRow], pairs: &[Vec<Pair>], above: usize, below: usize) -> &'static str {
    if rows[above].score != rows[below].score {
        "score"
    } else if rows[above].tie_break != rows[below].tie_break {
        "tie-break"
    } else if pairs[above][below].score() != pairs[below][above].score() {
        "head-to-head"
    } else {
        "name"
    }
}

fn rank_order(rows: &[PlayerRow], pairs: &[Vec<Pair>], a: usize, b: usize) -> Ordering {
    rows[b]
        .score
        .cmp(&rows[a].score)
        .then_with(|| rows[b].tie_break.cmp(&rows[a].tie_break))
        .then_with(|| pairs[b][a].score().cmp(&pairs[a][b].score()))
        .then_with(|| rows[a].name.cmp(&rows[b].name))
}

fn sort_for_display(rows: &mut [PlayerRow], sort: SortColumn) {
    rows.sort_by(|a, b| match sort {
        SortColumn::Name => a.name.cmp(&b.name),
        SortColumn::Rating => b.rating.cmp(&a.rating).then_with(|| a.name.cmp(&b.name)),
        SortColumn::Score => b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)),
        SortColumn::Federation => a.federation.cmp(&b.federation).then_with(|| a.name.cmp(&b.name)),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES: [&str; 8] = [
        "Alpha, A",
        "Bravo, B",
        "Charlie, C",
        "Delta, D",
        "Echo, E",
        "Foxtrot, F",
        "Golf, G",
        "Hotel, H",
    ];

    fn name_of(id: u32) -> Option<&'static str> {
        NAMES.get(id as usize).copied()
    }

    /// A round robin of `n` players, each meeting each other once, the lower-numbered player always
    /// winning.
    fn round_robin(n: usize) -> Vec<GameRow> {
        let mut games = Vec::new();
        for (row, (white, black)) in (0..n).flat_map(|white| (white + 1..n).map(move |black| (white, black))).enumerate() {
            games.push(GameRow {
                white: white as u32,
                black: black as u32,
                white_rating: 1500 + white as u16 * 20,
                black_rating: 1500 + black as u16 * 20,
                round: ((white + black) % n.max(1)) as i32 + 1,
                date: Date::parse("2024.05.01").unwrap(),
                result: GameResult::WhiteWins,
                row,
            });
        }
        games
    }

    fn build_over(games: &[GameRow], options: &Options) -> Crosstab {
        build(games, &name_of, options)
    }

    #[test]
    fn a_round_robin_shows_every_player_against_every_other() {
        let games = round_robin(12);
        let table = build_over(&games, &Options::default());
        assert_eq!(table.players.len(), 12);
        assert_eq!(table.columns, 12, "11 opponents and the diagonal");
        assert_eq!(table.mode, Mode::AllPlayAll, "{}", table.mode_reason);
        for row in &table.players {
            assert_eq!(row.games, 11);
            assert_eq!(row.wins * 2 + row.draws, (row.score.0 / 2) as usize, "score is wins plus half draws");
            assert_eq!(row.cells.iter().filter(|cell| cell.is_some()).count(), 11, "every pair met, the diagonal did not");
        }
        let top = table.players.iter().max_by_key(|row| row.score).unwrap();
        assert_eq!(top.score, Score(44), "eleven wins");
        assert_eq!(top.wins, 11);
        assert_eq!(top.losses, 0);
    }

    #[test]
    fn the_diagonal_is_not_a_result() {
        let table = build_over(&round_robin(4), &Options::default());
        for at in 0..table.columns {
            assert_eq!(table.pair_score(at, at), None, "a player meets themselves nowhere");
            assert!(table.is_diagonal(at, at));
            assert!(table.clash(at, at).is_empty());
        }
        assert_eq!(table.pair_score(0, 1), Some(WIN));
        assert_eq!(table.pair_score(1, 0), Some(LOSS));
        assert_eq!(table.pair_score(0, 1).unwrap().text(), "1");
    }

    #[test]
    fn automatic_mode_follows_the_documented_thresholds() {
        let table = build_over(&round_robin(8), &Options::default());
        assert_eq!(table.mode, Mode::AllPlayAll);
        assert!(table.mode_reason.contains("round robin"), "{}", table.mode_reason);

        // 60 players, 220 games: more than 30 players, so Swiss.
        let games: Vec<GameRow> = (0..220)
            .map(|i| GameRow {
                white: (i % 60) as u32,
                black: ((i + 1) % 60) as u32,
                white_rating: 0,
                black_rating: 0,
                round: -1,
                date: Date::UNKNOWN,
                result: GameResult::WhiteWins,
                row: i,
            })
            .collect();
        let table = build(&games, &|_| Some("Player, P"), &Options::default());
        assert_eq!(table.mode, Mode::Swiss);
        assert!(table.mode_reason.contains("more than 30"), "{}", table.mode_reason);

        // Four players, six games: three games each, fewer than five.
        let table = build_over(&round_robin(4), &Options::default());
        assert_eq!(table.mode, Mode::Swiss, "{}", table.mode_reason);
        assert!(table.mode_reason.contains("fewer than 5"), "{}", table.mode_reason);
    }

    #[test]
    fn an_explicit_mode_overrides_the_guess() {
        let table = build_over(&round_robin(8), &Options { mode: Mode::Knockout, ..Options::default() });
        assert_eq!(table.mode, Mode::Knockout);
        assert_eq!(table.mode_reason, "chosen by the user");
        assert!(table.players.iter().all(|row| row.tie_break.is_none()), "knockout has no tie-break");
    }

    #[test]
    fn head_to_head_breaks_an_equal_score_and_names_itself() {
        // A beat B, B beat C, C beat A: all three scored the same, so the meeting decides.
        let games = [
            (0u32, 1u32, GameResult::WhiteWins),
            (1, 2, GameResult::WhiteWins),
            (2, 0, GameResult::WhiteWins),
        ]
        .into_iter()
        .enumerate()
        .map(|(row, (white, black, result))| GameRow {
            white,
            black,
            white_rating: 1500,
            black_rating: 1500,
            round: row as i32 + 1,
            date: Date::UNKNOWN,
            result,
            row,
        })
        .collect::<Vec<_>>();
        let table = build_over(&games, &Options { mode: Mode::AllPlayAll, ..Options::default() });
        let mut ranked = table.players.clone();
        ranked.sort_by_key(|row| row.rank);
        let scores: Vec<i32> = ranked.iter().map(|row| row.score.0).collect();
        assert!(scores.windows(2).all(|pair| pair[0] == pair[1]), "all three scored the same: {scores:?}");
        let places: Vec<&str> = ranked.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(places, vec!["Alpha, A", "Bravo, B", "Charlie, C"], "{places:?}");
        assert_eq!(ranked[1].decided_by, "head-to-head", "their meeting decided it");
        assert_eq!(ranked[2].decided_by, "head-to-head");
        assert_eq!(ranked[0].rank, 1);
        assert_eq!(ranked[2].rank, 3);
    }

    #[test]
    fn tie_breaks_are_the_documented_ones_per_mode() {
        let games = vec![
            (0u32, 1u32, GameResult::WhiteWins),
            (0, 2, GameResult::WhiteWins),
            (1, 2, GameResult::WhiteWins),
            (2, 1, GameResult::Draw),
        ]
        .into_iter()
        .enumerate()
        .map(|(row, (white, black, result))| GameRow {
            white,
            black,
            white_rating: 1500,
            black_rating: 1500,
            round: row as i32 + 1,
            date: Date::UNKNOWN,
            result,
            row,
        })
        .collect::<Vec<_>>();
        let all = build_over(&games, &Options { mode: Mode::AllPlayAll, ..Options::default() });
        let swiss = build_over(&games, &Options { mode: Mode::Swiss, ..Options::default() });
        let break_of = |table: &Crosstab, name: &str| {
            table.players.iter().find(|row| row.name == name).unwrap().tie_break.unwrap()
        };
        for table in [&all, &swiss] {
            assert!(table.players.iter().all(|row| row.tie_break.is_some()));
        }
        // Sonnenborn-Berger counts an opponent's score only for the games that player lost or drew,
        // so A, who beat both of them, is ahead. Buchholz counts every game played, so C, who met
        // the field most, is ahead. Same games, different question.
        assert!(break_of(&all, "Alpha, A") > break_of(&all, "Charlie, C"));
        assert!(break_of(&swiss, "Charlie, C") > break_of(&swiss, "Alpha, A"));
        assert_eq!(tie_break_text(break_of(&all, "Alpha, A")), "2");
        assert_eq!(tie_break_text(break_of(&swiss, "Alpha, A")), "2");
    }

    #[test]
    fn an_oversized_set_is_cut_at_the_cap_and_says_what_it_cut() {
        let games: Vec<GameRow> = (0..3000)
            .map(|i| GameRow {
                white: (i % 900) as u32,
                black: ((i + 900) % 900) as u32,
                white_rating: 0,
                black_rating: 0,
                round: -1,
                date: Date::UNKNOWN,
                result: GameResult::Draw,
                row: i,
            })
            .collect();
        let table = build(&games, &|_| Some("Player, P"), &Options::default());
        assert_eq!(table.players.len(), MAX_PLAYERS);
        assert_eq!(table.excluded_players, 900 - MAX_PLAYERS);
        assert!(table.excluded_games >= table.excluded_players);
        assert!(table.exclusion_rule.contains("busiest"));
        assert_eq!(table.excluded_games + table.games, 3000, "nothing vanishes uncounted");
    }

    #[test]
    fn more_rounds_than_the_table_spans_are_left_out_and_counted() {
        let games: Vec<GameRow> = (0..200)
            .map(|i| GameRow {
                white: 0,
                black: 1,
                white_rating: 0,
                black_rating: 0,
                round: i as i32 + 1,
                date: Date::UNKNOWN,
                result: GameResult::Draw,
                row: i,
            })
            .collect();
        let table = build(&games, &|_| Some("Player, P"), &Options::default());
        assert_eq!(table.games, MAX_ROUNDS, "the first {MAX_ROUNDS} rounds are the table");
        assert_eq!(table.excluded_games, 200 - MAX_ROUNDS);
    }

    #[test]
    fn a_clash_between_two_players_of_the_table_can_be_read_back() {
        let games = round_robin(4);
        let table = build_over(&games, &Options::default());
        let clashes = table.clash(0, 1);
        assert_eq!(clashes.len(), 1);
        assert_eq!(clashes[0].result, GameResult::WhiteWins);
        assert!(table.clash(0, 0).is_empty(), "nobody meets themselves");
        assert_eq!(games[clashes[0].row].white, 0);
    }

    #[test]
    fn the_table_sorts_by_the_column_the_user_chose() {
        let games = round_robin(6);
        let by_name = build_over(&games, &Options { sort: SortColumn::Name, ..Options::default() });
        let by_score = build_over(&games, &Options { sort: SortColumn::Score, ..Options::default() });
        assert!(by_name.players.windows(2).all(|pair| pair[0].name <= pair[1].name));
        assert!(by_score.players.windows(2).all(|pair| pair[0].score >= pair[1].score));
        assert_eq!(by_score.players[0].score, Score(20), "five wins");
        assert_eq!(by_score.players[0].rank, 1);
        assert_eq!(by_name.players[0].name, "Alpha, A");
    }

    #[test]
    fn average_opponent_rating_and_the_list_attributes_come_through() {
        let games = round_robin(4);
        let (list, issues) = RatingList::parse(
            "name,rating,title,federation,birth\n\"Alpha, A\",1600,GM,ENG,1990.01.01\n",
        );
        assert!(issues.is_empty(), "{issues:?}");
        let table = build(
            &games,
            &name_of,
            &Options { info: Some(&list), show_age: true, ..Options::default() },
        );
        let alpha = table.players.iter().find(|row| row.name == "Alpha, A").unwrap();
        assert_eq!(alpha.title.as_deref(), Some("GM"));
        assert_eq!(alpha.federation.as_deref(), Some("ENG"));
        assert_eq!(alpha.average_opponent_rating, Some(1540));
        assert_eq!(alpha.rating, Some(1500), "the game's own rating beats the list's");
        assert_eq!(alpha.age, Some(34), "age at the date of the games");
        let bravo = table.players.iter().find(|row| row.name == "Bravo, B").unwrap();
        assert_eq!(bravo.age, None, "no birth date is no age");
        assert_eq!(bravo.federation, None);
    }
}
