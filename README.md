# EAL/3 in Rust

EAL evaluates engineering arguments against declared evidence, assumptions and
objections. EAL/3 source states the argument and its acquisition requests.
Operator configuration selects the collectors and grants their access to external
systems. Assessments retain the source, context, observation times and method
identities needed to explain the result.

The workspace provides parsing, composition, static validation, typed bindings,
authored assessment, evidence planning and optional finite ASPIC+ reasoning.
The runtime adds supervised collection, SQLite persistence, compatible observation
reuse and registered sources. CLI and MCP clients use the same runtime.

See [implementation boundaries](docs/implementation.md),
[runtime configuration](docs/runtime.md), [application commands](docs/applications.md)
and [evidence collectors](docs/collectors.md).

## Build and install

`rust-toolchain.toml` selects Rust 1.90.0 with rustfmt and Clippy.

```sh
cargo build --release --workspace --locked
cargo install --path crates/eal-process --locked
cargo install --path crates/eal-runtime --locked
cargo install --path crates/eal-cli --locked
cargo install --path crates/eal-mcp --locked
cargo install --path crates/eal-coordination --locked
cargo install --path crates/eal-collectors --locked
```

Install the packages into the same prefix so commands can resolve their companion
executables. The runtime requires POSIX ownership, file modes and process
facilities. Windows private-store ACL verification is unimplemented.

| Package | Commands |
| --- | --- |
| `eal-cli` | `eal`, `eal-syntax`, `eal-core`, `eal-runtime`, `eal-aspic-solve` |
| `eal-process` | `eal-process-supervisor` |
| `eal-runtime` | `eal-command-supervisor` compatibility alias and `eal-method-worker` |
| `eal-mcp` | `eal-mcp`, `eal-host` |
| `eal-coordination` | `eal-coordinate`, `eal-mas-server`, `mas-protocol-codec` |
| `eal-collectors` | `eal-collector` |

## Assess an argument

The [API example](examples/api-load-test.eal) declares a synthetic load-test argument.
Development commands can validate and plan without creating a store:

```sh
cargo run --locked -p eal-cli --bin eal-core -- --methods engineering validate examples/api-load-test.eal
cargo run --locked -p eal-cli --bin eal-core -- --methods engineering plan examples/api-load-test.eal --claim performance_criteria_met
```

Use the production CLI with an operator-owned registry for acquisition:

```sh
eal --workspace ./workspace --registry ./workspace/tools.toml \
  collect case.eal --context '{"site":"bench"}' --claim works
```

Pass the returned collection ID to `reason` with the same source and context.
`plan` computes the complete declared support, objection and defence closure.
`collect` executes configured collectors. `reason` assesses supplied durable
observations. Successful measured negatives and acquisition failures remain
distinct.

For repeated use, register the source and selected claim:

```sh
eal --workspace ./workspace --registry ./workspace/tools.toml \
  register case.eal --entry-id demo --claim works --context '{"site":"bench"}'
eal --workspace ./workspace --registry ./workspace/tools.toml \
  assess-known demo --claim works
```

Registered assessment reuses compatible observations at their original age and
collects missing or expired evidence. See [source registration and
assessment](docs/applications.md) and [bounded result packets](docs/packets.md).

## Evidence collectors

The registry supports `command` and `json_file` bindings. Command collectors
receive one bounded JSON request on stdin and return a JSON object containing
`value` with optional time, context, request and details fields. File observations
must preserve their observation time, context and request identity.

`eal-collector` supplies read-only Kubernetes, GitHub and Tempo adapters. Operator
arguments fix the executable or endpoint and allowed scope. Kubernetes and Tempo
use an explicit cluster inventory. The [catalogue deployment
example](examples/catalog-sync-operator/README.md) explains configuration and the
claim boundaries.

The command supervisor enforces deadlines and output limits and owns process-group
cleanup. Collectors execute with their configured host permissions. Select
`inherit_env` explicitly and pin collector dependencies when their byte identity
is required. See [runtime trust boundaries](docs/runtime.md).

## MCP interfaces

`eal-mcp` supports stdio and stateless Streamable HTTP through the Rust MCP SDK.
The [SDK codec configuration](vendor/rmcp/README.md) preserves JSON object keys
and number identity.

Operator exposure offers eleven operations for language discovery, validation,
formatting, planning, collection, reasoning, formal compilation and result
retrieval. Registered exposure offers `eal_sources`, `eal_find_claims`,
`eal_assess_known` and `eal_get_known_packet` for an operator allowlist:

```sh
eal-mcp --workspace ./workspace --registry ./workspace/tools.toml \
  --transport stdio --exposure registered --known-entry demo
```

Registered clients select an entry and claim. They cannot supply arbitrary source
or override the registered context. HTTP checks Host, Origin and bearer policy
before dispatch. See [transport and text-host configuration](docs/text-host.md).

## Reasoning and coordination

The default registry contains `structured/1`, `deductive/1`, `inductive/1`,
`abductive/1`, `causal/1`, `counterfactual/1`, `analogical/1` and `temporal/1`.
The `engineering` profile adds API-load, reachability, RMS and sampled-negative
methods. `aspic` adds `argumentation/aspic/2`; `all` includes every bundled method.

[Finite ASPIC+ reasoning](docs/aspic.md) supports grounded, preferred and stable
semantics with explicit preference and query policies. Exhausted search returns
an incomplete result. Export recomputes the formal result before producing
`aspic-view/3`. The [visualiser adapter](docs/native-visualiser.md) exposes the
bounded theory-to-view process interface.

[Durable coordination](docs/coordination.md) provides assessor and reviewer runs,
role-scoped gateways, immutable first responses, finite budgets and recovery of
uncertain effects. Its JSON mailbox and native MCP mailbox share the task and
delivery contracts.

## Verify

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Tests exercise native language semantics, exact numeric identities, independent
finite solver definitions, process deadlines, evidence correspondence, durable
reuse, transport admission, packet bounds and recovery. They establish the
properties checked on their declared inputs. Live deployment and telemetry
claims require observations from their configured systems.

The native [smoke command](docs/native-smoke.md) checks installed applications,
coordination and downstream graph consumption:

```sh
cargo install --path tools/eal-devtools --locked
eal-smoke applications --prefix /absolute/installed/bin
```

The [context experiment](experiments/eal-context-001/README.md) has a separate
Rust crate and explicit execution commands. Its planning simulations and
synthetic machinery checks are separate from live model-session measurements.

The manually triggered [workflow](.github/workflows/rust.yml) runs formatting,
Clippy, tests and a release build. It uses bounded debug information and caches
Cargo downloads and build output. Cache hits still run verification.

## Regenerate the parser

Normal builds use checked-in generated Rust sources. Regeneration requires Java
and curl. The generator and runtime are pinned through
[tools/antlr4rust.env](tools/antlr4rust.env).

```sh
sh tools/regenerate-parser.sh --check
sh tools/regenerate-parser.sh
```

`EAL.g4` is the EAL/3 grammar. Syntax recognition, lexical lowering, composition,
validation and assessment have separate interfaces and tests.

## Licence

The repository uses GPLv3. Required third-party attribution and licence texts
are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
