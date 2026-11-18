#!/usr/bin/env bash
# Provenance gate for the change `port-reference-database-internals` (task 1.2).
#
# Two checks:
#   1. every module NOTICE.md marks `implemented` exists on disk;
#   2. no source file of this project carries a copyleft header, and the MIT notice is carried
#      while the module that needs it exists.
#
# Exits non-zero on the first violation set. No dependencies beyond grep/awk.

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

notice=NOTICE.md
fail=0

copyleft='GNU General Public License|GNU Affero General Public License|GNU Lesser General Public License|GNU FREE DOCUMENTATION|This file is part of Scid|free software: you can redistribute'

# --- 1. modules marked implemented must exist -----------------------------------------------
while IFS='|' read -r module status _; do
    module=$(echo "$module" | tr -d ' `')
    status=$(echo "$status" | tr -d ' ')
    [ -f "$module" ] && continue          # already there
    case "$status" in
        implemented) echo "NOTICE.md: implemented module missing: $module" >&2; fail=1 ;;
        planned) ;;
        *) [ -n "$status" ] && { echo "NOTICE.md: unknown status '$status' for $module" >&2; fail=1; } ;;
    esac
done < <(awk -F'|' '/^\| `/{print $2 "|" $3}' "$notice")

# --- 2. no copyleft header in our own sources ------------------------------------------------
sources=$(find . -path ./.git -prune -o \
    \( -name '*.rs' -o -name '*.toml' -o -name '*.ts' -o -name '*.tsx' \) -print 2>/dev/null)
if [ -n "$sources" ]; then
    # shellcheck disable=SC2086
    hits=$(grep -lE "$copyleft" $sources 2>/dev/null)
    if [ -n "$hits" ]; then
        echo "copyleft header in project sources (see D1 in the change design):" >&2
        echo "$hits" >&2
        fail=1
    fi
fi

# --- 3. the MIT notice travels with the module that needs it ---------------------------------
lexer=chess-core/src/pgn/lexer.rs
if [ -f "$lexer" ] && ! grep -q "Permission is hereby granted, free of charge" "$notice"; then
    echo "$lexer exists but its MIT notice is not carried in $notice" >&2
    fail=1
fi

[ "$fail" -eq 0 ] && echo "provenance check passed"
exit "$fail"
