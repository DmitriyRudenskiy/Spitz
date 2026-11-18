# Rating list file format

A rating list is a small text file the user supplies — usually an export of a federation or a
rating-site download — giving attributes per player name: current rating, peak rating, title,
federation, birth date. It is read, never written by the app, and it is CSV so a person can fix a
row.

```
# lines starting with a hash and blank lines are ignored; the first row is the header
name,rating,peak,title,federation,birth
"Ivanov, Peter",2410,2455,GM,BUL,1970.03.14
"Nilsen, Ann",1980,,,,1988
"Chénier, Marie",2210,,FM,FRA,1995.07
```

## Columns

The header row names the columns; order is free and unknown columns are ignored.

| Column | Meaning |
|---|---|
| `name` | the player's name, in the form the library uses. Required. |
| `rating` | current rating. |
| `peak` | highest rating ever reached. Defaults to `rating`. |
| `title` | `GM`, `IM`, `FM`, `CM`, a woman's title, or `NM`/`AO` and the like. |
| `federation` | three letters, upper case. |
| `birth` | a date, and it may be partial: `1988`, `1995.07` or `1970.03.14` — the same shape PGN dates take. |

## Quoting and blanks

A field containing a comma is quoted with `"`; a `""` inside a quoted field is one `"`. An empty
field means **unknown**, and stays unknown: it is never read as rating 0 — a literal `0` is read as
unknown too — and a player with no birth date has no age rather than age 0.

## Bad rows

A row with no name, a rating that is not a number, or a birth date that is not a date is reported
with its line number and skipped. One bad row never costs the user the rest of the file.

## Matching

A name is looked up through the same collation the library sorts by, so case and diacritics do not
lose a row, and a name written given-first finds a row written surname-first. The list is an
outside source: a rating taken from it is shown as such, and the stored game is unchanged until the
user chooses to write the value in.
