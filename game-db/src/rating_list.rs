//! The rating list: a user-supplied file of player attributes, and the lookups a view does with it.
//!
//! `docs/rating-list-format.md` is the format note. The list is an outside source, so a value taken
//! from it carries that with it, and an attribute the list does not have stays unknown instead of
//! becoming zero.

use chess_core::date::Date;
use crate::collation;
use std::collections::HashMap;
use std::fmt;

/// One player's attributes. Every field but the name may be absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerInfo {
    pub name: String,
    pub rating: Option<u16>,
    pub peak: Option<u16>,
    pub title: Option<String>,
    pub federation: Option<String>,
    pub birth: Option<Date>,
}

/// A row that could not be used, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatingListIssue {
    pub line: usize,
    pub reason: &'static str,
}

impl fmt::Display for RatingListIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.reason)
    }
}

/// The list, indexed by the same collation the library sorts names by.
#[derive(Clone, Default, Debug)]
pub struct RatingList {
    players: Vec<PlayerInfo>,
    /// Folded and rotation-keyed, so case, diacritics and surname order do not lose a row.
    index: HashMap<String, u32>,
}

impl RatingList {
    /// Parse the file, returning what loaded and every row that did not.
    pub fn parse(text: &str) -> (Self, Vec<RatingListIssue>) {
        let mut list = Self::default();
        let mut issues = Vec::new();
        let mut columns: Option<Columns> = None;
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let text = raw.split('#').next().unwrap_or("").trim();
            if text.is_empty() {
                continue;
            }
            let fields = split_csv(text);
            let Some(header) = &columns else {
                match Columns::of(&fields) {
                    Some(header) => columns = Some(header),
                    None => {
                        issues.push(RatingListIssue { line, reason: "no name column in the header row" });
                        return (list, issues);
                    }
                }
                continue;
            };
            let Some(name) = header.field(&fields, Column::Name).filter(|name| !name.is_empty()) else {
                issues.push(RatingListIssue { line, reason: "row has no name" });
                continue;
            };
            let mut info = PlayerInfo {
                name: name.to_owned(),
                rating: None,
                peak: None,
                title: None,
                federation: None,
                birth: None,
            };
            let mut bad = None;
            if let Some(value) = header.field(&fields, Column::Rating) {
                match rating(value) {
                    Ok(value) => info.rating = value,
                    Err(reason) => bad = Some(reason),
                }
            }
            if bad.is_none() && let Some(value) = header.field(&fields, Column::Peak) {
                match rating(value) {
                    Ok(value) => info.peak = value,
                    Err(reason) => bad = Some(reason),
                }
            }
            if bad.is_none()
                && let Some(value) = header.field(&fields, Column::Birth).filter(|value| !value.is_empty())
            {
                match Date::parse(value) {
                    Ok(date) if date.year().is_some() => info.birth = Some(date),
                    Ok(_) => bad = Some("birth date has no year"),
                    Err(_) => bad = Some("birth date is not a date"),
                }
            }
            if let Some(reason) = bad {
                issues.push(RatingListIssue { line, reason });
                continue;
            }
            if let Some(value) = header.field(&fields, Column::Title).filter(|value| !value.is_empty()) {
                info.title = Some(value.to_ascii_uppercase());
            }
            match header.field(&fields, Column::Federation) {
                Some("") => {}
                Some(value) if value.len() == 3 && value.chars().all(char::is_alphabetic) => {
                    info.federation = Some(value.to_ascii_uppercase());
                }
                Some(_) => {
                    issues.push(RatingListIssue { line, reason: "federation is not three letters" });
                    continue;
                }
                None => {}
            }
            list.push(info);
        }
        (list, issues)
    }

    fn push(&mut self, info: PlayerInfo) {
        let key = lookup_key(&info.name);
        let at = self.players.len() as u32;
        self.players.push(info);
        self.index.entry(key).or_insert(at); // the first row for a name wins
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    pub fn players(&self) -> &[PlayerInfo] {
        &self.players
    }

    /// The row for a name, ignoring case, diacritics and surname-given order.
    pub fn player(&self, name: &str) -> Option<&PlayerInfo> {
        self.index.get(&lookup_key(name)).map(|&at| &self.players[at as usize])
    }

    /// The rating to show for a game: the game's own when it has one, otherwise the list's,
    /// labelled so the view can say where it came from.
    pub fn rating_for(&self, name: &str, game_rating: Option<u16>) -> Option<ShownRating> {
        if let Some(value) = game_rating.filter(|value| *value > 0) {
            return Some(ShownRating { value, source: RatingSource::Game });
        }
        self.player(name)
            .and_then(|player| player.rating)
            .map(|value| ShownRating { value, source: RatingSource::RatingList })
    }

    /// The peak rating to show, which is the list's business alone: a game record does not carry it.
    pub fn peak_for(&self, name: &str, game_rating: Option<u16>) -> Option<u16> {
        self.player(name).and_then(|player| player.peak.or(player.rating).or(game_rating))
    }

    /// Whole years old at the date of a game. A player with no birth year has no age, which is
    /// shown as unknown and never as zero.
    pub fn age_at(&self, name: &str, date: Date) -> Option<u8> {
        let birth = self.player(name)?.birth?;
        let birth_year = birth.year()?;
        // A birth date with no month counts from January, and no day from the first: the age can
        // then be a year out, which the format note says out loud.
        let birth_month = birth.month().unwrap_or(1);
        let game_year = date.year()?;
        let mut years = game_year.checked_sub(birth_year)?;
        let birth_day = birth.day().unwrap_or(1);
        let after_birthday = match (date.month(), date.day()) {
            (Some(month), Some(day)) => (month, day) >= (birth_month, birth_day),
            (Some(month), None) => month >= birth_month,
            (None, _) => true,
        };
        if !after_birthday {
            years = years.checked_sub(1)?;
        }
        (years <= u8::MAX as u16).then_some(years as u8)
    }
}

/// A rating and where it came from, because a value borrowed from the list is not a value the
/// library stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShownRating {
    pub value: u16,
    pub source: RatingSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingSource {
    Game,
    RatingList,
}

fn lookup_key(name: &str) -> String {
    collation::surname_key(name)
}

fn rating(value: &str) -> Result<Option<u16>, &'static str> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("rating is not a number");
    }
    match value.parse::<u32>() {
        // A zero rating is not a rating; it is the absence of one.
        Ok(0) => Ok(None),
        Ok(value) if value <= u16::MAX as u32 => Ok(Some(value as u16)),
        Ok(_) => Err("rating is too large"),
        Err(_) => Err("rating is not a number"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Column {
    Name,
    Rating,
    Peak,
    Title,
    Federation,
    Birth,
}

struct Columns(Vec<Option<Column>>);

impl Columns {
    fn of(fields: &[String]) -> Option<Self> {
        let columns: Vec<Option<Column>> = fields
            .iter()
            .map(|field| match field.trim().to_ascii_lowercase().as_str() {
                "name" | "player" => Some(Column::Name),
                "rating" => Some(Column::Rating),
                "peak" | "peak_rating" => Some(Column::Peak),
                "title" => Some(Column::Title),
                "federation" | "fed" => Some(Column::Federation),
                "birth" | "birth_date" | "born" => Some(Column::Birth),
                _ => None,
            })
            .collect();
        columns.contains(&Some(Column::Name)).then_some(Self(columns))
    }

    fn field<'a>(&self, fields: &'a [String], want: Column) -> Option<&'a str> {
        self.0
            .iter()
            .position(|column| *column == Some(want))
            .and_then(|at| fields.get(at).map(String::as_str))
    }
}

/// Split one CSV row, honouring quoted fields and `""` inside them.
fn split_csv(row: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = row.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ',' => fields.push(std::mem::take(&mut current)),
            '"' => loop {
                match chars.next() {
                    Some('"') if chars.peek() == Some(&'"') => {
                        current.push('"');
                        chars.next();
                    }
                    Some('"') | None => break,
                    Some(c) => current.push(c),
                }
            },
            c => current.push(c),
        }
    }
    fields.push(current);
    for field in &mut fields {
        *field = field.trim().to_owned();
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "\
# a rating list
name,rating,peak,title,federation,birth
\"Ivanov, Peter\",2410,2455,GM,BUL,1970.03.14
\"Nilsen, Ann\",1980,,,,1988
\"Chénier, Marie\",2210,,FM,FRA,1995.07
\"Berg, Kurt\",,,,,
";

    fn list() -> RatingList {
        let (list, issues) = RatingList::parse(LIST);
        assert!(issues.is_empty(), "{issues:?}");
        list
    }

    #[test]
    fn rows_load_with_their_attributes() {
        let list = list();
        assert_eq!(list.len(), 4);
        let ivanov = list.player("Ivanov, Peter").unwrap();
        assert_eq!(ivanov.rating, Some(2410));
        assert_eq!(ivanov.peak, Some(2455));
        assert_eq!(ivanov.title.as_deref(), Some("GM"));
        assert_eq!(ivanov.federation.as_deref(), Some("BUL"));
        assert_eq!(ivanov.birth, Some(Date::parse("1970.03.14").unwrap()));
        let ann = list.player("Nilsen, Ann").unwrap();
        assert_eq!(ann.birth, Some(Date::parse("1988").unwrap()));
        assert_eq!(ann.title, None, "an empty field stays unknown");
        assert_eq!(list.player("Berg, Kurt").unwrap().rating, None);
    }

    #[test]
    fn a_name_in_the_other_order_or_another_spelling_of_its_letters_still_matches() {
        let list = list();
        assert_eq!(list.player("Peter Ivanov").map(|p| p.rating), Some(Some(2410)));
        assert_eq!(list.player("CHENIER, MARIE").map(|p| p.rating), Some(Some(2210)));
        assert_eq!(list.player("Chénier, Marie").map(|p| p.rating), Some(Some(2210)));
        assert!(list.player("Nobody, Nobody").is_none());
    }

    #[test]
    fn a_missing_game_rating_is_filled_and_labelled_and_the_game_is_untouched() {
        let list = list();
        let shown = list.rating_for("Ivanov, Peter", None).unwrap();
        assert_eq!(shown.value, 2410);
        assert_eq!(shown.source, RatingSource::RatingList);
        let own = list.rating_for("Ivanov, Peter", Some(2380)).unwrap();
        assert_eq!((own.value, own.source), (2380, RatingSource::Game));
        assert_eq!(list.rating_for("Nobody, Nobody", None), None);
        assert_eq!(list.rating_for("Berg, Kurt", None), None, "unknown stays unknown");
        assert_eq!(list.rating_for("Berg, Kurt", Some(0)), None, "a zero rating is no rating");
    }

    #[test]
    fn age_is_counted_in_whole_years_and_unknown_is_not_zero() {
        let list = list();
        assert_eq!(list.age_at("Ivanov, Peter", Date::parse("2020.03.13").unwrap()), Some(49));
        assert_eq!(list.age_at("Ivanov, Peter", Date::parse("2020.03.14").unwrap()), Some(50));
        assert_eq!(list.age_at("Ivanov, Peter", Date::parse("2020.12.01").unwrap()), Some(50));
        assert_eq!(list.age_at("Ivanov, Peter", Date::parse("2020").unwrap()), Some(50));
        assert_eq!(list.age_at("Nilsen, Ann", Date::parse("2009.01.01").unwrap()), Some(21));
        assert_eq!(list.age_at("Berg, Kurt", Date::parse("2009.01.01").unwrap()), None);
        assert_eq!(list.age_at("Nobody, Nobody", Date::parse("2009.01.01").unwrap()), None);
    }

    #[test]
    fn peak_falls_back_to_the_current_rating() {
        let list = list();
        assert_eq!(list.peak_for("Chénier, Marie", None), Some(2210));
        assert_eq!(list.peak_for("Ivanov, Peter", None), Some(2455));
        assert_eq!(list.peak_for("Berg, Kurt", Some(1500)), Some(1500));
        assert_eq!(list.peak_for("Nobody, Nobody", None), None);
    }

    #[test]
    fn bad_rows_are_reported_by_line_and_the_rest_loads() {
        let (list, issues) = RatingList::parse(
            "\
name,rating,title,federation,birth
A B,1500,,,
,1500,,,
C D,high,,,
E F,1600,GM,,
G H,1700,GM,not-a-federation,
I J,1800,,FRA,1999.13.01
",
        );
        assert_eq!(list.len(), 2, "A B and E F");
        let lines: Vec<usize> = issues.iter().map(|issue| issue.line).collect();
        assert_eq!(lines, vec![3, 4, 6, 7], "every bad row is named: {issues:?}");
    }

    #[test]
    fn a_file_without_a_name_column_is_refused_early() {
        let (list, issues) = RatingList::parse("rating,title\n1500,GM\n");
        assert!(list.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 1);
    }

    #[test]
    fn column_order_is_free_and_unknown_columns_are_ignored() {
        let (list, issues) = RatingList::parse("born,title,name,rating,notes\n1980,GM,A B,1500,whatever\n");
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(list.player("A B").unwrap().rating, Some(1500));
        assert_eq!(list.player("A B").unwrap().birth, Some(Date::parse("1980").unwrap()));
    }

    #[test]
    fn a_quoted_name_keeps_its_comma_and_its_quote() {
        let (list, issues) = RatingList::parse("name,rating\n\"O'Brien, \"\"Duke\"\"\",2000\n");
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(list.players()[0].name, "O'Brien, \"Duke\"");
    }
}
