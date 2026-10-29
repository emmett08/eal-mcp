# Change EAL without losing its execution boundaries

A change to acquisition, assessment or a transport boundary can alter the meaning
of an existing result. Begin with [implementation boundaries](implementation.md)
and the contract owned by the component you intend to change. Keep observations,
source identity, numerical domains and incomplete results explicit in both code
and tests.

The repository uses Rust 1.90.0 and edition 2024. `rust-toolchain.toml` selects the
compiler, rustfmt and Clippy. Normal builds use checked-in generated parser code.
Handwritten crates forbid unsafe Rust. The parser adapter isolates the generated
module's existing context casts.

## Build and check the workspace

Run from the repository root:

```sh
cargo build --release --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Tests exercise native semantics, exact numeric identity, independent finite
solver definitions, process deadlines, evidence correspondence, durable reuse,
transport admission, packet bounds and recovery. They establish the properties
checked on their declared inputs. Live deployment or model-quality conclusions
require separately identified observations from those systems.

The [manually triggered workflow](../.github/workflows/rust.yml) runs workspace
verification, installed checks and the separate context experiment. Cargo caches
reduce repeated compilation while verification still runs after a cache hit.
Use the installed checks below when a change crosses an executable or transport
boundary.

## Regenerate the parser when the grammar changes

The EAL/4 grammar is [EAL.g4](../grammar/EAL.g4). Regeneration requires Java and
curl. The generator and runtime are pinned in
[antlr4rust.env](../tools/antlr4rust.env).

```sh
sh tools/regenerate-parser.sh --check
sh tools/regenerate-parser.sh
```

Recognition, lexical lowering, composition, validation and assessment have
separate interfaces and tests. Preserve the generated files when a change affects
only runtime behaviour. A grammar change requires regenerated recognisers and
checks of the language's intended meaning.

## Installed acceptance

`eal-smoke` consolidates the application, coordination and static consumer checks.
Build it with the workspace, then point it at one directory containing the installed
native commands and companions.

```sh
cargo build --workspace --locked
cargo build --locked -p eal-coordination --example coordination_software_worker

target/debug/eal-smoke applications --prefix target/debug
target/debug/eal-smoke coordination --prefix target/debug \
  --worker target/debug/examples/coordination_software_worker

target/debug/eal-smoke consumer \
  --consumer /absolute/path/to/aspic_visualisation/src/domain/aspic.ts \
  --binary target/debug/eal-core --node /absolute/path/to/node
```

Application and coordination checks accept `--workspace DIR`. The directory must
be new or empty. Their synthetic inputs and `smoke-result.json` are retained there,
including on failure. Omit the option to create a uniquely named temporary
workspace. Reports state the original observation time and installed prefix.

### What the installed checks establish

Application acceptance checks all eleven groups across
CLI validation and discovery, durable collection and reuse, model-context
projection, SDK stdio, authenticated HTTP and restricted historical packet
retrieval. HTTP controls check missing and incorrect bearer credentials, foreign
Origin and foreign Host. The bearer check rejects an entire credential-bearing
semantic response and preserves clean results unchanged. Installed operation
contract checks compare every advertised operation through text and SDK calls.
The installed `eal-tool-run` companion must reject an incomplete invocation with
empty stdout. This admission check requires no Docker image. Isolated package
execution has separate tool-runtime tests.

Coordination acceptance executes four complete software-worker runs, two through
the JSON mailbox and two through the native persistent MCP mailbox. It checks
exact retained response bytes and SHA-256, call-bound execution identities,
distinct supervisors, selected source and context, original assessment times,
collection accounting, message consumption, delivery acknowledgements and replay
without repeated calls or executions. An injected SQLite receipt-commit failure
checks that recovery retains an unknown effect after an assessment has persisted.
The subsequent manual assessor disposition retains its exact bytes. A reviewer
command completes the run with null unassessed fields and unchanged acquisition
accounting.

The consumer check pins both the TypeScript importer and its adjacent schema,
then calls the complete `parseAspicView` function using the companion checkout's
Ajv dependency. Run `npm ci` in that checkout first. Node 24 strips TypeScript;
the check relocates only its two import declarations and adds the JSON import
attribute required by Node. All importer function bodies remain unchanged.
Generated native cases include unresolved predicates and unavailable dependencies.
Seven forged graphs and three invalid metadata controls must be rejected.
Rust owns fixture selection, native command execution, independent producer-schema
checks, byte limits and result reporting. The manually triggered Rust workflow
also checks out the pinned companion and runs this import check. See
`eal-smoke --help` for cached views and explicit supervisor options.

### Ownership during verification

Every selected native command receives an explicit environment with `PATH` empty.
Command checks reuse `eal-process-supervisor`, with a 60-second monotonic deadline
and a 16 MiB combined output allowance. The HTTP server guard owns its process
group, independently drains each stream with a 4 MiB ceiling, and bounds the
complete server lifetime to 180 seconds. Startup has a 20-second deadline. Normal
shutdown sends SIGINT and allows five seconds before group cleanup. The HTTP
client bypasses proxies and enforces a five-second request deadline and a 4 MiB
response ceiling. Process ownership is retained through cleanup on failure.

These are deterministic software acceptance checks against synthetic observations.
They exercise local command, transport, persistence and recovery mechanisms.
External provider identity and model-quality studies retain separate evidence
requirements.

Smoke file admission runs before executable discovery or process execution.
Cached view JSON has a 64 MiB allowance; the pinned consumer source and schema
each have a 4 MiB allowance. These files must be regular and reject leaf aliases. Parent
aliases resolve to a selected canonical parent. Their loaders share the
runtime's bounded, non-blocking regular-file reader on POSIX hosts. The tests
cover FIFOs, ordinary aliases, exact allowances and sparse oversized files.
These checks bound allocation and special-file admission rather than every
possible filesystem stall.


## Keep research measurements separate

The [context experiment](context-experiment.md) is a separate
Rust crate outside production builds and published binaries. Follow its explicit
manifest and execution commands when changing experiment code. Planning
simulations, synthetic rehearsals and live model sessions retain their own
measurement labels and interpretation limits.

Return to the [documentation path](index.md).
