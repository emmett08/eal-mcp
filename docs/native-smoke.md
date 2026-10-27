# Native installed acceptance

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

## Preservation contract

Application acceptance checks all eleven groups across
CLI validation and discovery, durable collection and reuse, model-context
projection, SDK stdio, authenticated HTTP and restricted historical packet
retrieval. HTTP controls check missing and incorrect bearer credentials, foreign
Origin and foreign Host. The bearer check rejects an entire credential-bearing
semantic response and preserves clean results unchanged. Installed operation
contract checks compare every advertised operation through text and SDK calls.
The installed `eal-tool-run` companion must reject an incomplete invocation with
empty stdout. This admission check requires no Docker image; isolated package
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

The consumer check retains the exact pinned TypeScript graph-validator bodies,
validates native exports against the published schema and count policies, and
requires all seven forged-graph controls to fail. Node executes the supplied
consumer implementation. Rust owns fixture selection, native command execution,
schema checking, byte limits and result reporting. See `eal-smoke --help` for
cached views and explicit supervisor options.

## Process ownership

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
Cached view JSON has a 64 MiB allowance and the pinned consumer source has a
4 MiB allowance. Both require regular files and reject leaf aliases. Parent
aliases resolve to a selected canonical parent. Their loaders share the
runtime's bounded, non-blocking regular-file reader on POSIX hosts. The tests
cover FIFOs, ordinary aliases, exact allowances and sparse oversized files.
These checks bound allocation and special-file admission rather than every
possible filesystem stall.
