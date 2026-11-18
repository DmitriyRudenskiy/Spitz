//! PGN game dates whose month and day may be unknown.
//!
//! Behaviour studied from `src/date.h` of the reference suite; no source was copied (see
//! `NOTICE.md`). Unknown components are stored as zero, which is also what makes them sort
//! before every real value of the same or shorter prefix.

use std::fmt;

/// Largest representable year: the native header record gives a date 20 bits, 11 of them year.
pub const MAX_YEAR: u16 = 2047;

/// A date with optional month and day. A component of `None` is unknown.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateError {
    /// Not a number, a dash separator, or too many components.
    Malformed,
    YearTooLarge,
    MonthOutOfRange,
    DayOutOfRange,
}

impl Date {
    /// A fully known date.
    pub const fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Every component unknown, the value a game with no `Date` tag carries.
    pub const UNKNOWN: Self = Self {
        year: 0,
        month: 0,
        day: 0,
    };

    pub const fn year(&self) -> Option<u16> {
        match self.year {
            0 => None,
            y => Some(y),
        }
    }

    pub const fn month(&self) -> Option<u8> {
        match self.month {
            0 => None,
            m => Some(m),
        }
    }

    pub const fn day(&self) -> Option<u8> {
        match self.day {
            0 => None,
            d => Some(d),
        }
    }

    /// True when at least one component is unknown.
    pub const fn is_partial(&self) -> bool {
        self.year == 0 || self.month == 0 || self.day == 0
    }

    /// The 20-bit form the native header record stores: year in the top 11 bits, then month,
    /// then day. Ordering by this key is the ordering the game list sorts by.
    pub const fn sort_key(&self) -> u32 {
        ((self.year as u32) << 9) | ((self.month as u32) << 5) | self.day as u32
    }

    /// Parse `YYYY.MM.DD` where a component may be `??`, `00`, empty, or absent, and the whole
    /// string may be empty. A trailing separator is tolerated.
    pub fn parse(text: &str) -> Result<Self, DateError> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(Self::UNKNOWN);
        }
        let mut parts = text.split('.');
        let year = component(parts.next(), MAX_YEAR as u32, DateError::YearTooLarge)?;
        let month = component(parts.next(), 12, DateError::MonthOutOfRange)?;
        let day = component(parts.next(), 31, DateError::DayOutOfRange)?;
        if parts.next().is_some_and(|rest| !rest.is_empty()) {
            return Err(DateError::Malformed);
        }
        Ok(Self {
            year: year as u16,
            month: month as u8,
            day: day as u8,
        })
    }

    /// The `[min, max]` span a date can denote, as sort keys. Unknown components widen the
    /// span to the whole of the enclosing component, so no calendar arithmetic is needed:
    /// both bounds use the same convention and only ever meet each other.
    fn span(&self) -> (u32, u32) {
        let low = Date::new(if self.year == 0 { 1 } else { self.year },
                            if self.month == 0 { 1 } else { self.month },
                            if self.day == 0 { 1 } else { self.day })
            .sort_key();
        let high = Date::new(if self.year == 0 { MAX_YEAR } else { self.year },
                             if self.month == 0 { 12 } else { self.month },
                             if self.day == 0 { 31 } else { self.day })
            .sort_key();
        (low, high)
    }
}

/// An inclusive date range whose own bounds may be partial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
    pub start: Date,
    pub end: Date,
}

impl DateRange {
    /// True when the date can fall inside the range, which for unknown components means the
    /// two spans overlap.
    pub fn matches(&self, date: Date) -> bool {
        let (low, high) = date.span();
        let (from, to) = (self.start.span().0, self.end.span().1);
        low <= to && from <= high
    }
}

/// One `YYYY`/`MM`/`DD` field: digits, `??`, `00`, or empty for unknown.
fn component(field: Option<&str>, highest: u32, too_large: DateError) -> Result<u32, DateError> {
    let field = match field {
        None | Some("") => return Ok(0),
        Some(field) => field,
    };
    if field.bytes().all(|c| c == b'?') {
        return Ok(0);
    }
    if !field.bytes().all(|c| c.is_ascii_digit()) {
        return Err(DateError::Malformed);
    }
    let value: u32 = field.parse().map_err(|_| DateError::Malformed)?;
    if value > highest {
        return Err(too_large);
    }
    Ok(value)
}

impl fmt::Display for Date {
    /// Unknown components print as `??`, so a partial date never prints as a real one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let year = match self.year {
            0 => "?".repeat(4),
            y => format!("{y:04}"),
        };
        let month = match self.month {
            0 => "??".to_string(),
            m => format!("{m:02}"),
        };
        let day = match self.day {
            0 => "??".to_string(),
            d => format!("{d:02}"),
        };
        write!(f, "{year}.{month}.{day}")
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            DateError::Malformed => "malformed date",
            DateError::YearTooLarge => "year above the representable maximum",
            DateError::MonthOutOfRange => "month out of range",
            DateError::DayOutOfRange => "day out of range",
        };
        f.write_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str) -> Date {
        Date::parse(text).expect("parses")
    }

    #[test]
    fn parses_every_unknown_form() {
        for text in [
            "1999.??.??",
            "1999.03.??",
            "1999.03",
            "1999",
            "1999..",
            "1999.03.00",
            "1999.00.04",
        ] {
            assert_eq!(parsed(text).year(), Some(1999), "{text}");
        }
        assert_eq!(parsed("1999.??.??").month(), None);
        assert_eq!(parsed("1999.03.??").day(), None);
        assert_eq!(parsed("1999.03.00").day(), None);
        assert_eq!(parsed("1999.00.04").month(), None);
        assert_eq!(parsed("").year(), None);
        assert_eq!(parsed("????.??.??"), Date::UNKNOWN);
        assert_eq!(parsed("1999.3.4"), Date::new(1999, 3, 4));
        assert_eq!(parsed("1999.03.14"), Date::new(1999, 3, 14));
    }

    #[test]
    fn rejects_malformed_dates() {
        assert_eq!(Date::parse("1999-03-14"), Err(DateError::Malformed));
        assert_eq!(Date::parse("abcd"), Err(DateError::Malformed));
        assert_eq!(Date::parse("1999.03.14.01"), Err(DateError::Malformed));
        assert_eq!(Date::parse("3000.01.01"), Err(DateError::YearTooLarge));
        assert_eq!(Date::parse("1999.13.01"), Err(DateError::MonthOutOfRange));
        assert_eq!(Date::parse("1999.12.32"), Err(DateError::DayOutOfRange));
    }

    #[test]
    fn partial_date_matches_only_inside_its_year() {
        let partial = parsed("1999.??.??");
        let inside = DateRange {
            start: parsed("1999.03.01"),
            end: parsed("1999.11.30"),
        };
        let later = DateRange {
            start: parsed("2000.01.01"),
            end: parsed("2000.12.31"),
        };
        assert!(inside.matches(partial));
        assert!(!later.matches(partial));
    }

    #[test]
    fn full_date_outside_a_partial_bound_still_fails() {
        let range = DateRange {
            start: parsed("1999.01.01"),
            end: parsed("1999.??.??"),
        };
        assert!(range.matches(parsed("1999.11.30")));
        assert!(!range.matches(parsed("2000.01.01")));
    }

    #[test]
    fn unknown_components_survive_a_round_trip() {
        let date = parsed("2011.05.??.");
        assert_eq!(date.year(), Some(2011));
        assert_eq!(date.month(), Some(5));
        assert_eq!(date.day(), None);
        let text = date.to_string();
        assert_eq!(text, "2011.05.??");
        assert_eq!(Date::parse(&text), Ok(date));
        assert_eq!(Date::UNKNOWN.to_string(), "????.??.??");
    }

    #[test]
    fn unknown_components_sort_first() {
        let mut dates = [parsed("1999.05.01"), Date::UNKNOWN, parsed("1999"), parsed("1998.12.31")];
        dates.sort_by_key(Date::sort_key);
        assert_eq!(
            dates,
            [Date::UNKNOWN, parsed("1998.12.31"), parsed("1999"), parsed("1999.05.01")]
        );
    }
}
