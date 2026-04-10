#!/bin/sh

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
moon_exe="$repo_root/target/debug/moon"

if [ ! -x "$moon_exe" ]; then
    (
        cd "$repo_root"
        cargo build
    )
fi

if [ ! -x "$moon_exe" ]; then
    printf '%s\n' "moon executable was not found at $moon_exe" >&2
    exit 1
fi

day=1
while [ "$day" -le 30 ]; do
    date=$(printf '2026-05-%02d' "$day")
    printf '\n=== %s ===\n' "$date"
    "$moon_exe" "$@" "$date"
    day=$((day + 1))
done
