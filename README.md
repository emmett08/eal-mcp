# EAL/3 in Rust

This repository ports the production EAL/3 implementation from
[`emmett08/earl`](https://github.com/emmett08/earl) to Rust. The source reference is
[`e2d6f346`](https://github.com/emmett08/earl/tree/e2d6f346a37f92483b9f9a13923b98feb053447b),
package version `3.2.4`. The source repository remains unchanged.

The implementation now includes generated recognition of the unchanged `EAL.g4`
and an authored semantic core: composed scopes, trusted imports, inherited
defaults, closed patterns, static validation, typed propositions and bindings,
expressions, reasoning methods and assessment of supplied observations. It also
provides evidence planning, standalone Dung grounded labelling and checked
semantic formatting. The development commands `eal-syntax` and `eal-core` run
without Python or Java.

The opt-in [finite ASPIC implementation](docs/aspic.md) provides explicit theory
solving, observation-bound EAL compilation and recomputed `aspic-view/3` export.
The [native runtime](docs/runtime.md) adds private SQLite storage, trusted collector
acquisition, observation reuse and supervised optional methods. The
[application layer](docs/applications.md) provides the production `eal` CLI,
immutable source catalogues, registered assessment, bounded packets and model
context. `eal-mcp` serves the operator or registered catalogue over stdio or
Streamable HTTP; [`eal-host`](docs/text-host.md) sends one checked request through
the official Rust MCP SDK. The SDK has a documented
[local JSON-preservation patch](vendor/rmcp/README.md).
The [coordination layer](docs/coordination.md) adds durable assessor/reviewer
runs, immutable first responses, role-scoped MCP gateways and native mailbox
and codec executables. The
[native visualiser adapter](docs/native-visualiser.md) adds `eal-aspic-solve` for
the companion editor's bounded theory-to-view process protocol.
See the [implementation status](docs/port-status.md) and frozen
[source inventory](contracts/source-lock.json). Companion browser integration and
installed acceptance checks are described in the native visualiser guide.

The separate [EAL long-session experiment](experiments/eal-context-001/README.md)
contains the proposed evaluation, reproducible planning simulations and machinery
diagnostics. Its [execution report](experiments/eal-context-001/results.md)
distinguishes those measurements from the unrun model-session pilot.

## Build and use

Install Rust through rustup. `rust-toolchain.toml` selects Rust `1.90.0`,
including rustfmt and Clippy. Normal builds use the checked-in generated Rust
sources.

```sh
cargo build --workspace --locked
cargo run --locked -p eal-cli --bin eal-syntax -- parse fixtures/parser/sources/api-load-test.eal
cargo run --locked -p eal-cli --bin eal-syntax -- format fixtures/parser/sources/api-load-test.eal
cargo run --locked -p eal-cli --bin eal-syntax -- recognise fixtures/parser/sources/api-load-test.eal
cargo run --locked -p eal-cli --bin eal-core -- --methods engineering validate fixtures/parser/sources/api-load-test.eal
cargo run --locked -p eal-cli --bin eal-core -- describe
```

Install all four application packages into the same prefix so the production
commands can find their companion executables:

```sh
cargo install --path crates/eal-runtime --locked
cargo install --path crates/eal-cli --locked
cargo install --path crates/eal-mcp --locked
cargo install --path crates/eal-coordination --locked
```

This installs `eal`, `eal-mcp`, `eal-host`, `eal-aspic-solve`, the development commands,
`eal-command-supervisor`, `eal-method-worker`, `eal-coordinate`, `eal-mas-server`
and `mas-protocol-codec`. The durable runtime currently
requires the documented POSIX private-store profile. See
[application commands and catalogue boundaries](docs/applications.md),
[HTTP and text-host policy](docs/text-host.md), and
[packet and model-context projections](docs/packets.md). Coordination retains
the ten lifecycle/gateway commands, trusted task profiles and finite run budgets.
Its default transport is a durable JSON mailbox; selecting `--runtime` uses an
installed native `eal-mas-server` through persistent MCP.

The API load-test fixture contains synthetic observations. In `eal-syntax`,
`parse` emits the typed lexical AST, `recognise` emits the concrete tree and
tokens, and `format` checks lexical preservation. The semantic development CLI
accepts these commands:

```text
eal-core [--workspace DIR] [--limits FILE] [--methods builtin|engineering|aspic|all] COMMAND

compile SOURCE
validate SOURCE
format SOURCE
plan SOURCE --claim NAME
evaluate SOURCE --now ISO_TIME [--context JSON|@FILE] [--records FILE]
grounded GRAPH
aspic THEORY_JSON
compile-aspic SOURCE --claim NAME --now ISO_TIME [--context JSON|@FILE] [--records FILE]
  [--semantics grounded|preferred|stable] [--query-mode credulous|sceptical]
  [--preference minimum_rank|last_link_rank|last_link_partial]
export-aspic RESULT_JSON
describe
```

`compile` emits composed semantic entities; `validate` emits ordered static
diagnostics. `evaluate` assesses the supplied records at the explicit,
timezone-aware instant. Context defaults to `{}` and can be supplied inline or
from a JSON file prefixed with `@`; records default to `{}`. It uses no clock,
collector or persistent store. `plan` follows alternative derivations, premises,
backing, assumption validation, objections and defences to produce the complete
evidence dependency closure and declared calls. It does not execute those calls.
`grounded` reads a JSON object with `arguments` and `attacks`, for example
`{"arguments":["a","b"],"attacks":[["a","b"]]}`. `describe` emits method,
typed-binding and budget contracts.

Use `-` for source from standard input. The workspace defaults to the current
directory; all file inputs and trusted `.eal` imports must resolve within it.
`--limits FILE` reads a TOML `[limits]` table, with unspecified fields retaining
their defaults. For example:

```toml
[limits]
source_bytes = 1048576
declarations = 4096
json_depth = 64
```

The default registry contains `structured/1`, `deductive/1`, `inductive/1`,
`abductive/1`, `causal/1`, `counterfactual/1`, `analogical/1` and `temporal/1`.
`--methods engineering` adds `engineering/api-load-criteria/1`,
`engineering/reachability/1`, `engineering/rms/1` and
`engineering/sampled-negative/1`. Profile `aspic` installs the eight defaults
plus `argumentation/aspic/2`; `all` installs all thirteen methods.

`aspic` accepts exactly `{"theory": ...}` and emits its theory and formal result.
`compile-aspic` translates a checked observation snapshot, retaining both the
formal and authored claim results. `export-aspic` recomputes the supplied formal
result before emitting the graph. See [ASPIC semantics and examples](docs/aspic.md)
for preference, extension, query and provenance contracts. An exhausted formal
search returns `incomplete` with empty details and exit status 1.

Expressions preserve arbitrary integers, binary64 arithmetic, negative modulo,
missing values and expression errors. Typed bindings use exact rational unit
conversions and canonical request/observation identities. Native methods report
Rust implementation identities. A custom native callback's artefact hash is a
host declaration; the core checks its shape, while the host must establish its
correspondence to the loaded code. The runtime supplies supervised callbacks with startup and execution deadlines;
direct core callbacks remain host-owned in-process implementations.

Semantic `format` validates, replays captured imports, and checks composed
semantic preservation and idempotence. Formatting changes source identity, so
observations must be bound to the new source. Library callers supplying direct
IR without retained authored syntax can format plain declarations; imports,
templates and generated declarations require that authored syntax. Validation
failure returns exit status 1; input or command errors return status 2.

Default `eal-syntax` recognition limits are 1 MiB of UTF-8 source, 100,000
tokens, nesting depth 64 and 1,000,000 concrete syntax nodes. Configured nesting
has a hard ceiling of 128, including retained unary operators. Semantic
composition selects nesting from `expansion_depth`, which defaults to 128.
Recognition and lexical
lowering each run on a temporary worker reserving a 16 MiB stack, so embedded
callers can use ordinary small stacks. Typed expression depth is bounded at
128, including flat operator chains. JSON depth defaults to 64 with a hard
ceiling of 128; schema instance checks also cap traversal at 128. Semantic
source-import and block traversal cap native recursion at 128; iterative pattern
applications retain the host's selected expansion-depth budget. Semantic
budgets additionally bound declarations, expansion, references and graph edges.
`structural_values` counts scalar, compound and object-key visits in the Rust
IR serializer, rather than Python dataclass reflection values; its boundary is
representation-specific. Limits produce diagnostics. Formatting can
increase whitespace, tokens or nesting. The checked formatter reports a limit
when the emitted source would exceed the selected budgets. Library callers can
use `eal_core::Program::format_with_limits` for lexical formatting with explicit
budgets; semantic formatting uses the compiled programme's limits.

The production `eal` command connects validation, planning, collection, durable
assessment, supervised ASPIC compilation, catalogues and explanation. The
development `eal-runtime` driver remains available. Build all workspace binaries
before using runtime operations; the supervisor and method worker are separate
companion executables. Production `grounded` and `export-aspic` run before service
construction and create no database. Other production commands use the shared
service; the pure `eal-core` development commands remain available when storage
is unnecessary. See [runtime configuration and boundaries](docs/runtime.md).

## Verify

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Development and test builds optimise `sha2` for repeated executable identity
checks. Hashing complete native companions, including their debug symbols, must
fit the gateway deadlines on CPUs without SHA instructions.

The retained parser reference covers all 76 grammar rules in 101 cases.
Rust tests compare complete parse trees, token identities/text/channels and
Unicode scalar/byte positions, reject malformed input after recovery, check
frozen artefact hashes and exercise the CLI with an empty executable search
path. Further tests cover structural checks and formatter round trips.
The [fixture notes](fixtures/parser/README.md) distinguish recognition from
composed Python lowering. These checks establish the implemented front end's
behaviour on the retained corpus. Native tests also compare the retained
expression, method, validation, canonical-identity, planning and
[cross-component semantic](fixtures/semantics/README.md) corpora. The latter
contains 93 source-to-assessment and import-failure cases. Fixtures state any
numerical tolerance or native backend identity projection; source, query and
observation identities remain exact. The differential reference profile is
CPython 3.12.14 with Unicode 15; `sum` uses its 64-bit C-long profile consistently
across Rust targets. Python is used only to capture development references,
never by normal Rust tests.

Application references retain catalogue revisions and history, packet/context
projections, discovery, MCP descriptors and rejected calls, and host admission
and redaction. Native integration tests exercise registered reuse, both MCP
transports, HTTP policy and cancellation. Coordination checks add task/profile
correspondence, exact first-response retention, durable reservations, delivery
recovery and bounded command execution. These fixtures and checks establish
their recorded cases; the consumer's live bridge and complete package acceptance
remain separate work.

The GitHub workflow runs on manual dispatch. It runs Rust checks and parser
regeneration in separate jobs on `ubuntu-latest`.
The Rust job retains file/line backtraces with `line-tables-only` dev/test debug
information and disables incremental compilation to reduce cached build output.
Debug assertions, overflow checks and release-profile settings remain unchanged.
Cargo downloads and build outputs are cached by runner environment, native
compiler, pinned Rust toolchain and build configuration. Each revision has its
own cache key; compatible earlier revisions provide a fallback. Cache hits do
not skip formatting, Clippy, any workspace test or the release build.

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
ported source methods, MCP catalogues, CLI operations and schemas. The native
[packet retrieval and budget extensions](docs/packets.md#registered-retrieval-and-presentation-budgets)
add one registered tool and optional presentation limits while preserving
existing default response data. The retained
`aspic-view/3` schema matches the visualiser at the pinned consumer revision.
Rust export checks that schema and recomputes the complete formal result.
The [native live bridge](docs/native-visualiser.md) retains that graph contract
and delegates solving to the installed Rust adapter.

The repository retains its GPLv3 licence. Imported source attribution and
third-party notices are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
