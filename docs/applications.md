# Production applications and registered sources

The native application layer provides `eal`, `eal-mcp` and `eal-host`. All use
the same reasoning contracts and configured runtime. Coordination, including
`eal-coordinate`, remains the next stage. The source reference remains
`earl@e2d6f346`; the source repository and grammar are unchanged.

## Installation

Build all binaries for development, or install all three packages into the same
prefix:

```sh
cargo build --release --workspace --locked

cargo install --path crates/eal-runtime --locked
cargo install --path crates/eal-cli --locked
cargo install --path crates/eal-mcp --locked
```

The runtime package installs `eal-command-supervisor` and `eal-method-worker`;
the CLI package installs `eal` and the development drivers; the MCP package
installs `eal-mcp` and `eal-host`. Runtime commands resolve their companion
executables beside themselves. The CLI also accepts operator-owned
`--supervisor` and `--method-worker` paths. `eal-host` launches the installed
sibling `eal-mcp` by default. No normal operation invokes Python or Java.

The durable runtime currently requires POSIX ownership, file modes and process
facilities. Windows builds retain an explicit private-store refusal because
Windows ACL validation is unimplemented. See [runtime boundaries](runtime.md)
and [the method-worker protocol](method-worker.md).

## CLI operations

Use `eal --help` for complete arguments. Common configuration is
`--workspace DIR`, `--registry TOML`, `--database SQLITE`, `--limits TOML` and
`--methods builtin|engineering|aspic|all`. Registry, database and executable
locations are trusted operator choices. Source and context files and imports
must stay inside the workspace. Runtime imports resolve from the workspace
root, including when the primary file is in a subdirectory.

| Commands | Result or effect |
| --- | --- |
| `describe`, `validate`, `format`, `plan` | Language/contracts, diagnostics, checked formatting or complete evidence dependency planning |
| `collect`, `reason`, `compile-aspic` | Configured acquisition, durable authored assessment or supervised formal compilation |
| `explain`, `packet` | Stored explanation or a bounded assessment projection |
| `register`, `register-tree`, `sources`, `find` | Source registration, bounded discovery and catalogue lookup |
| `assess-known`, `history`, `model-context` | Registered claim assessment, revision-specific run metadata or checked model messages |
| `grounded`, `export-aspic` | Explicit graph labelling or recomputed checked graph export |

Production `grounded` and `export-aspic` run before service construction: they
create no database and require no runtime companions. `export-aspic` writes only
its explicit workspace output. Other production commands use the shared service
and can create `.eal/runs.sqlite3` and resolve companions even when the selected
operation performs no acquisition. Use the pure `eal-core` development CLI for
storage-free source validation, formatting, planning and supplied-record
assessment.

For an existing trusted `case.eal` declaring claim `works`:

```sh
eal --workspace ./workspace --registry ./workspace/tools.toml \
  register case.eal --entry-id demo --claim works --context '{"site":"bench"}'
eal --workspace ./workspace --registry ./workspace/tools.toml sources
eal --workspace ./workspace --registry ./workspace/tools.toml \
  assess-known demo --claim works --reuse compatible
eal --workspace ./workspace --registry ./workspace/tools.toml history demo
eal --workspace ./workspace --registry ./workspace/tools.toml \
  model-context demo --claim works --question 'What does the evidence establish?'
```

Assessment uses the current UTC time unless `--now ISO_TIME` supplies a
timezone-aware instant. A supported result is conditional on declared scope,
observations, assumptions and inference rationales; it does not prove prose
truth. Invalid validation, incomplete computation and failed observations use
exit status 1. Invalid commands or inputs use status 2. A valid unsupported or
contested claim is a completed assessment, rather than a command error.

## Catalogue and registered assessment

An entry fixes its workspace-relative `.eal` path, label, context and selected
claims. Its default ID is the normalized path; an explicit developer ID must
match `[A-Za-z_][A-Za-z_0-9.-]{0,127}`. Changing fixed metadata requires another
entry ID and an independently registered path. Context is finite JSON with an
8,192-byte canonical envelope; one to 256 claims may be selected. Tree discovery
is sorted and bounded to 512 EAL files, 8,192 scanned entries and depth 16 below
the discovery root, and reports rejected source files individually. It does not descend
through directory symlinks.

Refresh validates the current source and imports before recording a revision.
Revisions are immutable and keyed by entry ID, exact source/import-bundle digest
and method-registry fingerprint. Invalid edits leave the last valid revision
intact. Method changes can create a new revision while the source digest stays
the same. History selects exact source and context identities and returns run
metadata, rather than observation values. It does not turn a historical result
into a fresh observation.

The native adapter intentionally fixes the pinned source's nested default-ID
refresh bug. A generated ID such as `nested/case.eal` remains valid when an
existing entry refreshes; the source mistakenly rechecks it as an explicit
developer ID and rejects `/`. Public registration still applies the explicit-ID
regex to newly supplied IDs. A retained oracle witness records the source
failure and native tests cover registration, refresh and registered assessment.

Registered assessment plans the complete support, objection and defence closure.
The default `compatible` policy selects successful stored readings only when
their request, scoped context, current binding and command-environment identities
match. Each reading retains its original observation time and is checked for
freshness at assessment time. A comparable measured negative can be reused;
failed acquisition cannot. Missing readings are acquired and rebinding batches
are committed atomically. `--reuse fresh` acquires the selected evidence again.
The workspace lease spans selection, acquisition, rechecks and durable commit.
The CLI and embedded API can override context for one assessment without changing
entry metadata; catalogue history filters the entry's registered context.

Packets and model context exclude raw observations, process streams and full
source. They retain bounded statuses, checked scalar outputs, qualifications,
omissions and explanation links. Negative, stale and missing evidence remain
distinct. See [packet and context contracts](packets.md).

## MCP exposure and text host

`eal-mcp` uses the official Rust `rmcp` SDK for stdio and stateless Streamable
HTTP. The [pinned SDK and local JSON codec patch](../vendor/rmcp/README.md) retain
literal object keys and number identity with the evaluator's JSON features.
The modified SDK archive, checksum, license and exact patch application are
recorded; normal builds do not execute its upstream build script.

The default operator exposure publishes eleven tools for description, source
formatting/validation/planning, collection, reasoning, formal compilation,
explanation, packets and grounded graphs. Registered exposure publishes only
`eal_sources`, `eal_find_claims` and `eal_assess_known` for a configured allowlist:

```sh
eal-mcp --workspace ./workspace --registry ./workspace/tools.toml \
  --transport stdio --exposure registered --known-entry demo
```

The catalogues are exclusive. Registered entries are checked at startup; the
registered tools accept neither arbitrary source nor caller-selected context,
time or reuse policy. Public source summaries omit registered context and source
text. Closed schemas, bounded admission and acquisition exclusion run before
effects. Cancellation after blocking work starts retains its permit and lease
until that work completes.

Strict integer fields reject integral floats such as `1.0`, preserving the
source rejection outcome with the native diagnostic
`Tool input requires a strict integer for FIELD`. HTTP paths are literal: braces
are rejected instead of selecting framework route parameters. Host, Origin and
bearer policy is checked before SDK dispatch. For configuration precedence,
authentication, deadlines, HTTP redaction and the one-request `eal-host` client,
see [text host and HTTP policy](text-host.md).

Retained application references cover catalogue metadata/history, packet/context
projections, discovery, MCP descriptors and rejected requests, and host policy.
Native tests add registered reuse, transport and cancellation regressions.
These checks describe the exercised cases; coordination, the visualiser's native
live-solver bridge and complete installed-package acceptance remain outstanding.
