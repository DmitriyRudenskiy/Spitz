# Spelling rule file format

A rule file is a small text file the user supplies when a source spells names one way and the rest
of the world spells them another. It is applied when *matching* a name — completing it, correcting
it, finding its rating list row — and never when storing it, so the header text a user typed stays
exactly as typed.

```
# one rule per line: kind, the text to find, the text to put in its place
prefix  spsa  spas
infix   sz    ss
suffix  ski   sky
```

## The kinds

| Kind | Finds | Replaces |
|---|---|---|
| `prefix` | the start of the name | once |
| `infix` | any occurrence inside the name | every occurrence |
| `suffix` | the end of the name | once |

Rules are applied in the order they appear, so a file can undo its own earlier rule.

## Case and accents

A name is folded before rules run — lower case, accents removed — and so is every rule, which is
why `Spsa` and `spsa` are the same rule and `Chen` finds `Chén`.

## Bad lines

A line that is not three fields, a kind that is not one of the three, or a rule that finds nothing
is reported with its line number and skipped. The rest of the file loads.
