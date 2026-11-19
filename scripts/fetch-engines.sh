#!/usr/bin/env bash
# Fetch the engines the real-engine compliance suite drives, and print them as a
# SPITZ_ENGINES list.
#
# Spitz ships no engine (ADR-0001) and its test fixtures are generated rather than vendored
# (ADR-0009), so the binaries live upstream and come down on demand. The default CI job
# therefore still opens no sockets and fails on no mirror; this script belongs to the
# nightly `engines` workflow, where a download failure is worth reading about.
#
# Three independent codebases, because an engine that agrees with itself twice proves
# nothing: Stockfish (the engine everyone is measured against), Arasan (a long-lived
# independent implementation with its own dialect), Viridithas (a Rust engine whose output
# order and banner lines differ from both).
#
# Usage: SPITZ_ENGINES=$(scripts/fetch-engines.sh [directory]) cargo test ...
set -euo pipefail

dir="${1:-$(pwd)/.engines}"
mkdir -p "$dir"

case "$(uname -s)" in
  Linux)  platform=linux ;;
  Darwin) platform=macos ;;
  *) echo "fetch-engines: unsupported platform $(uname -s)" >&2; exit 1 ;;
esac

# name|url — one archive or binary per engine, for this platform.
if [ "$platform" = linux ]; then
  engines=(
    "stockfish|https://github.com/official-stockfish/Stockfish/releases/download/sf_19/stockfish-linux-x86-64-universal.tar.gz"
    "arasan|https://github.com/jdart1/arasan-chess/releases/download/v26.1/arasan-linux-binaries-26.1.tar.gz"
    "viridithas|https://github.com/cosmobobak/viridithas/releases/download/v20.0.0/viridithas-20-linux-x86-64-v3"
  )
else
  # Arasan's macOS builds are shipped as a disk image, which is more work than a nightly
  # suite is worth; Linux is where all three run.
  engines=(
    "stockfish|https://github.com/official-stockfish/Stockfish/releases/download/sf_19/stockfish-macos-universal.tar.gz"
    "viridithas|https://github.com/cosmobobak/viridithas/releases/download/v20.0.0/viridithas-20-macos-aarch64"
  )
fi

found=()
# An engine that never answers must not hang the fetch. `timeout` is GNU; a Mac without
# coreutils trusts the engine instead.
guard=""
command -v timeout >/dev/null 2>&1 && guard="timeout 60"

for entry in "${engines[@]}"; do
  name="${entry%%|*}"
  url="${entry#*|}"
  here="$dir/$name"
  mkdir -p "$here"

  if [ -z "$(find "$here" -type f -perm -u+x -print -quit)" ]; then
    echo "fetch-engines: $name <- $url" >&2
    curl -fsSL --retry 3 -o "$here/$(basename "$url")" "$url"
    case "$url" in
      *.tar.gz) tar xzf "$here/$(basename "$url")" -C "$here" ;;
      *.zip)    unzip -q "$here/$(basename "$url")" -d "$here" ;;
      *)        chmod +x "$here/$(basename "$url")" ;;
    esac
  fi

  # The executable inside: not a network file, not a README, and not a second build for
  # another CPU. Arasan ships several builds and Stockfish ships its source next to the
  # binary, so the first candidate that answers `uci` is the engine.
  binary=""
  while IFS= read -r candidate; do
    if printf 'uci\nquit\n' | $guard "$candidate" 2>/dev/null | grep -q "^uciok\$"; then
      binary="$candidate"
      break
    fi
  done < <(find "$here" -type f -perm -u+x \( -name "*$name*" -o -name "*uci*" \) -print)
  if [ -z "$binary" ]; then
    echo "fetch-engines: $name did not answer a handshake; skipping it" >&2
    continue
  fi
  found+=("$binary")
done

(IFS=:; echo "${found[*]:-}")
