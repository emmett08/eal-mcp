#!/bin/sh
# Regenerate the checked-in recognisers from the authoritative EAL/3 grammar.
set -eu

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_root"
. ./tools/antlr4rust.env

check=false
case "${1:-}" in
    --check) check=true; shift ;;
    "") ;;
    *) echo "usage: tools/regenerate-parser.sh [--check]" >&2; exit 2 ;;
esac
[ "$#" -eq 0 ] || { echo "unexpected arguments" >&2; exit 2; }

cache_dir=${XDG_CACHE_HOME:-$HOME/.cache}/eal-mcp
jar=${ANTLR4RUST_JAR:-$cache_dir/antlr4rust-$ANTLR4RUST_VERSION.jar}
temporary_dir=$(mktemp -d)
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM

if [ ! -f "$jar" ]; then
    mkdir -p "$cache_dir"
    curl --fail --location --silent --show-error --retry 3 \
        "$ANTLR4RUST_URL" -o "$temporary_dir/generator.jar"
    printf '%s  %s\n' "$ANTLR4RUST_SHA256" "$temporary_dir/generator.jar" | sha256sum --check --status
    cp "$temporary_dir/generator.jar" "$jar"
fi
printf '%s  %s\n' "$ANTLR4RUST_SHA256" "$jar" | sha256sum --check --status

output_dir=$temporary_dir/generated
mkdir -p "$output_dir"
# antlr4rust 0.6.0 emits EALListener references even with -no-listener.
java -jar "$jar" -Dlanguage=Rust -visitor -Xexact-output-dir \
    -o "$output_dir" grammar/EAL.g4

generated_dir=crates/eal-parser/src/generated
if "$check"; then
    diff -rq "$generated_dir" "$output_dir"
    echo "Generated parser matches the pinned grammar and generator."
else
    mkdir -p "$generated_dir"
    cp "$output_dir"/* "$generated_dir"/
    echo "Regenerated parser with antlr4rust $ANTLR4RUST_VERSION."
fi
