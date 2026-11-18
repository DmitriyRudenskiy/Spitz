//! The three renderings of a cross-table: plain text, HTML, and typeset source.
//!
//! They all take the same `Table`, built once from the `Crosstab`, so a number cannot be one value
//! on screen and another on paper. The plain-text form is the one that has to fit a page.

use crate::crosstab::{Crosstab, Score, tie_break_text};

/// The name column in the matrix block: enough for a surname, short enough that twenty opponents
/// still fit a page.
const NAME_COLUMN: usize = 20;

const DIAGONAL: &str = "-";
const NO_GAME: &str = "";

/// One rendering-independent table: a title, the notes about what it left out, a summary block, and
/// the matrix of who beat whom.
#[derive(Clone, Debug)]
pub struct Table {
    pub title: String,
    pub notes: Vec<String>,
    pub summary: Vec<String>,
    pub opponents: Vec<String>,
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub summary: Vec<String>,
    pub matrix: Vec<String>,
}

impl Table {
    /// Every matrix cell in reading order, which is what the three formats are checked against.
    pub fn matrix_cells(&self) -> Vec<&str> {
        self.rows.iter().flat_map(|row| row.matrix.iter().map(String::as_str)).collect()
    }

    /// The name of a row, taken from the summary block so both blocks agree.
    pub fn name(&self, at: usize) -> String {
        self.rows.get(at).map(|row| row.summary[1].clone()).unwrap_or_default()
    }
}

/// Turn a cross-table into the model the renderers share.
pub fn model(table: &Crosstab, show_age: bool) -> Table {
    let mut summary: Vec<String> =
        ["rank", "name", "rating", "title", "fed", "g", "w", "d", "l", "score", "avg", "tb", "by"].map(String::from).to_vec();
    if show_age {
        summary.insert(5, "age".to_owned());
    }
    let opponents = (0..table.columns).map(|b| table.players[b].name.clone()).collect();
    let mut rows = Vec::with_capacity(table.players.len());
    for a in 0..table.players.len() {
        let row = &table.players[a];
        let mut cells = vec![
            row.rank.to_string(),
            row.name.clone(),
            row.rating.map(|v| v.to_string()).unwrap_or_else(|| "?".into()),
            row.title.clone().unwrap_or_default(),
            row.federation.clone().unwrap_or_default(),
        ];
        if show_age {
            cells.push(row.age.map(|v| v.to_string()).unwrap_or_default());
        }
        cells.extend([
            row.games.to_string(),
            row.wins.to_string(),
            row.draws.to_string(),
            row.losses.to_string(),
            row.score.text(),
            row.average_opponent_rating.map(|v| v.to_string()).unwrap_or("?".into()),
            row.tie_break.map(tie_break_text).unwrap_or_default(),
            row.decided_by.to_string(),
        ]);
        let matrix = (0..table.columns)
            .map(|b| {
                if table.is_diagonal(a, b) {
                    DIAGONAL.to_owned()
                } else {
                    table.pair_score(a, b).map(cell_text).unwrap_or(NO_GAME.to_owned())
                }
            })
            .collect();
        rows.push(Row { summary: cells, matrix });
    }
    Table {
        title: format!("Cross-table — {}", table.mode.label()),
        notes: vec![
            format!("{} games. {}", table.games, table.mode_reason),
            format!(
                "{} players and {} games excluded: {}",
                table.excluded_players, table.excluded_games, table.exclusion_rule
            ),
        ],
        summary,
        opponents,
        rows,
    }
}

/// A pair's score: "1", "0.5", "0".
fn cell_text(score: Score) -> String {
    score.text()
}

/// Plain text, in two blocks: the summary, then the matrix. Both fit a page for a table of twenty.
pub fn to_plain(table: &Crosstab, show_age: bool) -> String {
    let model = model(table, show_age);
    let mut out = String::new();
    out.push_str(&model.title);
    out.push('\n');
    for note in &model.notes {
        out.push_str(note);
        out.push('\n');
    }

    let mut widths: Vec<usize> = model.summary.iter().map(|heading| heading.chars().count()).collect();
    for row in &model.rows {
        for (at, cell) in row.summary.iter().enumerate() {
            widths[at] = widths[at].max(cell.chars().count());
        }
    }
    // The name column is the one worth room; the rest are numbers.
    let name_at = 1;
    widths[name_at] = widths[name_at].min(24);
    let mut line = String::new();
    for (at, heading) in model.summary.iter().enumerate() {
        let width = widths[at];
        let text: String = heading.chars().take(width).collect();
        if at == name_at {
            line.push_str(&format!("{text:<width$} "));
        } else {
            line.push_str(&format!("{:>width$} ", text));
        }
    }
    out.push_str(line.trim_end());
    out.push('\n');
    for row in &model.rows {
        for (at, cell) in row.summary.iter().enumerate() {
            let width = widths[at];
            let text: String = cell.chars().take(width).collect();
            if at == name_at {
                out.push_str(&format!("{text:<width$} "));
            } else {
                out.push_str(&format!("{:>width$} ", text));
            }
        }
        out.push('\n');
    }

    out.push('\n');
    out.push_str(&" ".repeat(NAME_COLUMN));
    for name in &model.opponents {
        let short = short_name(name);
        out.push_str(&format!("{short:>3} "));
    }
    out.push('\n');
    for (a, row) in model.rows.iter().enumerate() {
        let name: String = model.name(a).chars().take(NAME_COLUMN).collect();
        let width = NAME_COLUMN;
        out.push_str(&format!("{name:<width$}"));
        for cell in &row.matrix {
            out.push_str(&format!("{cell:>3} "));
        }
        out.push('\n');
    }
    out
}

/// A column heading for the matrix: the surname, in three characters.
fn short_name(name: &str) -> String {
    let surname = name.split(',').next().unwrap_or(name);
    surname.chars().take(3).collect()
}

/// HTML for the browser, with the matrix as a second table so the summary stays readable.
pub fn to_html(table: &Crosstab, show_age: bool) -> String {
    let model = model(table, show_age);
    let mut out = String::from("<!doctype html>\n<html lang=\"en\">\n<meta charset=\"utf-8\">\n<title>");
    out.push_str(&escape_html(&model.title));
    out.push_str("</title>\n<h1>");
    out.push_str(&escape_html(&model.title));
    out.push_str("</h1>\n");
    for note in &model.notes {
        out.push_str("<p class=\"note\">");
        out.push_str(&escape_html(note));
        out.push_str("</p>\n");
    }
    out.push_str("<table>\n<thead><tr>");
    for heading in &model.summary {
        out.push_str(&format!("<th>{}</th>", escape_html(heading)));
    }
    out.push_str("</tr></thead>\n<tbody>\n");
    for row in &model.rows {
        out.push_str("<tr>");
        for cell in &row.summary {
            out.push_str(&format!("<td>{}</td>", escape_html(cell)));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody></table>\n<table class=\"matrix\">\n<thead><tr><th></th>");
    for name in &model.opponents {
        out.push_str(&format!("<th title=\"{}\">{}</th>", escape_html(name), escape_html(&short_name(name))));
    }
    out.push_str("</tr></thead>\n<tbody>\n");
    for (a, row) in model.rows.iter().enumerate() {
        out.push_str(&format!("<tr><th>{}</th>", escape_html(&model.name(a))));
        for cell in &row.matrix {
            out.push_str(&format!("<td>{}</td>", escape_html(cell)));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody></table>\n</html>\n");
    out
}

/// Typeset source, one tabular: summary columns then the matrix, which is what a printed
/// cross-table looks like.
pub fn to_latex(table: &Crosstab, show_age: bool) -> String {
    let model = model(table, show_age);
    let mut out = String::from("\\begin{longtable}{");
    out.push('r');
    out.push_str(&"l|".repeat(model.summary.len() - 1));
    out.push_str(&"c".repeat(model.opponents.len()));
    out.push_str("}\n\\caption{");
    out.push_str(&escape_latex(&model.title));
    out.push_str("} \\\\\n\\hline\n");
    out.push_str(&model.summary.iter().chain(model.opponents.iter()).map(|heading| escape_latex(heading)).collect::<Vec<_>>().join(" & "));
    out.push_str(" \\\\\n\\hline\n");
    for row in &model.rows {
        let cells = row
            .summary
            .iter()
            .map(|cell| escape_latex(cell))
            .chain(row.matrix.iter().map(|cell| escape_latex(cell)))
            .collect::<Vec<_>>()
            .join(" & ");
        out.push_str(&cells);
        out.push_str(" \\\\\n");
    }
    out.push_str("\\hline\n\\end{longtable}\n");
    out
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn escape_latex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => {
                out.push('\\');
                out.push(ch);
            }
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            '\\' => out.push_str("\\textbackslash{}"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crosstab::{GameRow, Mode, Options, SortColumn, build};
    use chess_core::date::Date;
    use chess_core::pgn::lexer::GameResult;

    const NAMES: [&str; 20] = {
        let mut names = ["Unnamed, U"; 20];
        names[0] = "Alpha, A";
        names[1] = "Bravo, B";
        names[2] = "Charlie, C";
        names[3] = "Delta, D";
        names[4] = "Echo, E";
        names
    };

    fn name_of(id: u32) -> Option<&'static str> {
        NAMES.get(id as usize).copied()
    }

    fn round_robin(n: usize) -> Vec<GameRow> {
        (0..n)
            .flat_map(|white| (white + 1..n).map(move |black| (white, black)))
            .enumerate()
            .map(|(row, (white, black))| GameRow {
                white: white as u32,
                black: black as u32,
                white_rating: 1500 + white as u16 * 20,
                black_rating: 1500 + black as u16 * 20,
                round: ((white + black) % n) as i32 + 1,
                date: Date::parse("2024.05.01").unwrap(),
                result: if white % 3 == black % 3 { GameResult::Draw } else if white < black { GameResult::WhiteWins } else { GameResult::BlackWins },
                row,
            })
            .collect()
    }

    fn table(n: usize) -> Crosstab {
        let games = round_robin(n);
        build(&games, &name_of, &Options { sort: SortColumn::Score, ..Options::default() })
    }

    /// The matrix cells as each format prints them, for the equality check.
    fn plain_cells(text: &str) -> Vec<String> {
        let lines: Vec<&str> = text.lines().collect();
        let blank = lines.iter().position(|line| line.is_empty()).expect("a blank line between the blocks");
        lines[blank + 2..]
            .iter()
            .flat_map(|line| {
                line.chars()
                    .skip(20)
                    .collect::<String>()
                    .split(' ')
                    .filter(|cell| !cell.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn html_cells(html: &str) -> Vec<String> {
        let matrix = html.split("class=\"matrix\"").nth(1).unwrap();
        matrix
            .lines()
            .filter(|line| line.contains("<td>"))
            .flat_map(|line| {
                line.split("<td>")
                    .skip(1)
                    .map(|cell| cell.split('<').next().unwrap_or(cell).replace("&amp;", "&").to_owned())
            })
            .collect()
    }

    fn latex_cells(latex: &str) -> Vec<String> {
        latex
            .lines()
            .filter(|line| line.contains(" & ") && !line.contains("\\hline") && !line.contains("\\caption") && !line.contains("rank"))
            .flat_map(|line| {
                line.trim_end_matches(" \\\\")
                    .split(" & ")
                    .skip(13) // the summary columns
                    .map(|cell| cell.replace("\\_", "_"))
            })
            .collect()
    }

    #[test]
    fn all_three_formats_print_exactly_the_same_numbers() {
        let table = table(12);
        let model = model(&table, false);
        let expected: Vec<String> = model.matrix_cells().iter().map(|cell| cell.to_string()).collect();
        assert_eq!(plain_cells(&to_plain(&table, false)), expected, "plain text");
        assert_eq!(html_cells(&to_html(&table, false)), expected, "html");
        assert_eq!(latex_cells(&to_latex(&table, false)), expected, "typeset source");
    }

    #[test]
    fn the_plain_text_form_of_twenty_players_fits_one_page() {
        let table = table(20);
        let text = to_plain(&table, true);
        let lines: Vec<&str> = text.lines().collect();
        let widest = lines.iter().map(|line| line.chars().count()).max().unwrap();
        assert!(lines.len() <= 66, "{} lines is more than a page", lines.len());
        assert!(widest <= 100, "{widest} columns is wider than a page");
        assert_eq!(lines.iter().filter(|line| line.contains(", ")).count(), 40, "20 players twice, once per block");
    }

    #[test]
    fn the_html_escapes_what_needs_escaping_and_labels_the_diagonal() {
        let mut table = table(4);
        table.players[0].name = "O'Brien & Co, <B>".into();
        let html = to_html(&table, false);
        assert!(html.contains("O&apos;") || html.contains("O'Brien"));
        assert!(html.contains("O'Brien &amp; Co, &lt;B&gt;"), "the name is escaped");
        assert!(html.contains("<th></th>") || html.contains("class=\"matrix\""));
        let latex = to_latex(&table, false);
        assert!(latex.contains("O'Brien \\& Co, <B>"), "latex escapes the ampersand, not the angle");
    }

    #[test]
    fn the_notes_say_what_the_table_left_out() {
        let table = table(8);
        let text = to_plain(&table, false);
        assert!(text.contains("no exclusion was needed"), "{text}");
        assert!(text.contains("all-play-all") || text.contains("swiss"), "the mode is in the title");
        assert!(text.contains(&table.mode_reason), "and why it was picked");
    }

    #[test]
    fn a_tied_placing_shows_the_rule_that_decided_it() {
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
        let table = build(&games, &name_of, &Options { mode: Mode::AllPlayAll, ..Options::default() });
        let text = to_plain(&table, false);
        assert!(text.contains("head-to-head"), "the deciding rule is printed\n{text}");
        assert!(to_html(&table, false).contains("head-to-head"));
        assert!(to_latex(&table, false).contains("head-to-head"));
    }
}
