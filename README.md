# EAL/3 in Rust

This repository ports the production EAL/3 implementation from
[`emmett08/earl`](https://github.com/emmett08/earl) to Rust. The source reference is
[`e2d6f346`](https://github.com/emmett08/earl/tree/e2d6f346a37f92483b9f9a13923b98feb053447b),
package version `3.2.4`. The source repository remains unchanged.

The current implementation provides generated recognition of the unchanged
`EAL.g4`, owned syntax trees, typed unexpanded declarations and expressions,
structural diagnostics and a canonical source formatter. The development command
`eal-syntax` exercises these components without Python or Java.

Reasoning, ASPIC solving/export, trusted import resolution, pattern expansion,
observation storage, collection, MCP, the model host and coordination are the
remaining port stages. See the [implementation sequence](docs/port-status.md)
and the [module inventory](contracts/source-lock.json). The production `eal`,
`eal-mcp`, `eal-host` and `eal-coordinate` entry points will be implemented with
their corresponding services.

## Build and use

Install Rust through rustup. `rust-toolchain.toml` selects Rust `1.90.0`,
including rustfmt and Clippy. Normal builds use the checked-in generated Rust
sources.

```sh
cargo build --workspace --locked
cargo run --locked -p eal-cli --bin eal-syntax -- parse fixtures/parser/sources/api-load-test.eal
cargo run --locked -p eal-cli --bin eal-syntax -- format fixtures/parser/sources/api-load-test.eal
cargo run --locked -p eal-cli --bin eal-syntax -- recognise fixtures/parser/sources/api-load-test.eal
```

The API load-test fixture contains synthetic observations. `parse` emits the
typed lexical AST as JSON. `recognise` emits the concrete tree, tokens and source
spans. `format` emits canonical source after checking its recognition limits and
typed AST preservation. Use `-` in place of a file to read UTF-8
source from standard input. Errors go to standard error and return a failing
exit status. Recognition and structural checks execute no source imports,
collectors or reasoning methods.

Default recognition limits are 1 MiB of UTF-8 source, 100,000 tokens, nesting
depth 64 and 1,000,000 concrete syntax nodes. Configured nesting has a hard
ceiling of 128, including retained unary operators. Recognition and lexical
lowering each run on a temporary worker reserving a 16 MiB stack, so embedded
callers can use ordinary small stacks. Typed expression depth is bounded at
128, including flat operator chains. Limits produce diagnostics. Formatting can
increase whitespace, tokens or nesting. The checked formatter reports a limit
when the emitted source would exceed the selected budgets. Library callers can
use `Program::format_with_limits` to apply their selected budgets explicitly.

## Verify

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The retained Python reference corpus covers all 76 grammar rules in 101 cases.
Rust tests compare complete parse trees, token identities/text/channels and
Unicode scalar/byte positions, reject malformed input after recovery, check
frozen artefact hashes and exercise the CLI with an empty executable search
path. Further tests cover structural checks and formatter round trips.
The [fixture notes](fixtures/parser/README.md) distinguish recognition from
composed Python lowering. These checks establish the implemented front end's
behaviour on the retained corpus. Semantic comparison belongs to each later
port stage.

The GitHub workflow runs on manual dispatch. It checks Rust on Linux and Windows
and checks parser regeneration separately on Linux.

## Regenerate the parser

Regeneration requires Java and curl. The generator/runtime pair is
`antlr4rust 0.6.0`; the generator download is SHA-256 checked using
[`tools/antlr4rust.env`](tools/antlr4rust.env).

```sh
sh tools/regenerate-parser.sh --check
sh tools/regenerate-parser.sh
```

Set `ANTLR4RUST_JAR` to reuse an existing verified generator. Generated files stay
byte-identical to generator output. The generated module contains three unsafe
context casts supplied by the generator. Its narrow lint exception is isolated
from the handwritten adapter, core and CLI.

## Compatibility and licence

[`contracts/public-contracts.json`](contracts/public-contracts.json) freezes the
current methods, MCP catalogues, CLI operations and schemas. The retained
`aspic-view/3` schema matches the visualiser at the pinned consumer revision.
Rust export and the visualiser's native live-solver bridge are later stages.

The repository retains its GPLv3 licence. Imported source attribution and
third-party notices are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
