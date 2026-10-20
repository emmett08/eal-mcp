# Durable acquisition and supervised methods

`eal-runtime` supplies the native runtime beneath the later production CLI,
catalogues and transports. It keeps IO, clock lookup and process execution out
of `eal-core` and `eal-aspic`. The shared `ReasoningService` implements validation,
formatting, evidence planning, collection, stored-collection assessment,
supervised ASPIC compilation and durable explanation. Packet, catalogue,
model-context and MCP operations remain pending.

## Build and use

Build the workspace so the command and its two companions are available together:

```sh
cargo build --release --workspace --locked
```

For an installation, install both packages into the same prefix:

```sh
cargo install --path crates/eal-runtime --locked
cargo install --path crates/eal-cli --locked
```

The development driver finds `eal-command-supervisor` and `eal-method-worker`
beside its own executable. Operator options `--supervisor` and `--method-worker`
can supply other executable paths, resolved before their identities are captured.
The companions are version-coupled to this
runtime; their private transport is separate from the method extension protocol.

```text
eal-runtime [--workspace DIR] [--registry FILE] [--database FILE]
            [--limits FILE] [--methods builtin|engineering|aspic|all]
            [--workers 1..32] COMMAND

validate SOURCE
format SOURCE
plan SOURCE --claim NAME
collect SOURCE --context JSON|@FILE [--claim NAME | --evidence NAME ...]
reason SOURCE --context JSON|@FILE [--collection ID] [--now ISO_TIME]
compile-aspic SOURCE --context JSON|@FILE --collection ID --goal NAME
explain ASSESSMENT_ID [--claim NAME]
```

`compile-aspic` also accepts `--now`, `--semantics`, `--query-mode` and
`--preference` with the same choices as `eal-core`. Assessment uses the current
UTC time unless the operator supplies an explicit timezone-aware instant.
Source and context files and trusted `.eal` imports must resolve within the
workspace. Registry, database and executable locations are operator configuration.
Runtime service imports resolve from the workspace root; placing the primary
source in a subdirectory does not change that base. The pure `eal-core` file-mode
driver additionally supplies the source file's origin to its composition API.
`validate` gives status 1 for invalid source; collection gives status 1 when an
observation fails. Command/input errors, including source parse errors in other
operations, give status 2. Incomplete computation supplies no accepted or negative
conclusion.

## Trusted collectors

A TOML registry binds declared tool names and versions to operator-owned code
or existing observations. Source and evidence payloads cannot select executables.

```toml
[tools.runner]
kind = "command"
version = "1"
argv = ["/absolute/path/to/collector", "--json"]
timeout_seconds = 30
max_output_bytes = 1048576
inherit_env = ["PATH", "KUBECONFIG"]
parallel_safe = false

[tools.imported_report]
kind = "json_file"
version = "1"
path = "observations/report.json"
```

Omitting `inherit_env` inherits the complete current environment; an empty list
inherits none. The optional `env` table overlays the selected environment.
`pinned_files` accepts up to 32 distinct `{path, sha256}` entries for command
dependencies. Relative files stay within the workspace. Absolute pinned paths
are explicit operator grants. Regular-file checks, no-follow opens and before/
after digest checks reject detected substitution. Unpinned libraries, network
services and physical measurements remain outside these identity checks.

Collectors read one JSON request on standard input and emit one strict JSON
object containing `value` and optional `observed_at`, `context`, `request` and
`details`. The `request` correspondence object contains `tool`, `tool_version`,
`input` and scoped `context`. File imports require all three of `observed_at`,
`context` and `request`, preserving original time and acquisition identity.
Duplicate keys, nonfinite numbers, invalid Unicode and excessive JSON depth fail.

The complete selected collection and its byte allowances are checked before the
first collector runs. Consecutive `parallel_safe = true` bindings may overlap,
up to the operator's worker limit; a serial binding is a barrier. Returned records
keep requested order. Successful and failed observations are durable; raw output
is replaced by SHA-256 digests and byte counts. A successful measured negative
value remains distinct from a failed collector.
Collector envelopes obey the binding's output byte cap; the semantic
`record_bytes` allowance is applied later when assessing each durable observation.

## Persistence and reuse

The default database is `.eal/runs.sqlite3`. A store instance anchors the parent,
database and 32-byte random key identities. It checks ownership, private mode,
regular type, link count and no-follow policy around fresh SQLite connections.
The persistent initialisation lock serialises schema/key creation; atomic batch
insertion preserves immutable IDs. Key installation fsyncs the file and directory.
Replacement of a live instance's database or key fails. The private-store policy
currently requires POSIX; Windows ACL verification is not implemented.

The source's `EAL/2` HMAC domain remains frozen for observation-record/1.
Binding and exact effective collector environment identities use the private
store key. Runtime metadata records keyed environment identity without copying
the environment or key bytes. Trusted collector output is retained as its declared
observed value.
Stored payloads have a separate fixed 64 MiB defensive envelope with depth 128;
collection admission retains the source's 32 MiB aggregate limit and independent
per-observation budgets.

`ObservationRebinder` selects acquisition-compatible successful records from the
indexed store and verifies the exact stored original before preparing a detached
record. It requires current binding and, for commands, execution-environment
identity. It retains original `collected_at`, `observed_at` and origin identity;
rebinding never refreshes measurement age. Freshness and evidence predicates are
assessed later at the explicit assessment instant. The registered-source policy
that chooses and atomically commits reused collections belongs to the catalogue
stage.

The retained runtime references contain 119 acquisition witnesses and 38
collection-identity/reuse cases from the unmodified pinned Python source. Malformed
JSON preserves failure categories while native parser wording can differ.
Stored-original comparison deliberately uses typed canonical equality, refusing
metadata substitutions such as integer `1` for float `1.0` that Python object
equality can accept. Registry TOML also has a native 1 MiB admission ceiling.

## Process ownership and method extensions

Each command has a separate supervisor. Its absolute monotonic deadline starts
before configuration transfer. Nonblocking IO bounds combined stdout/stderr and
handles a collector that never reads stdin. Collector status is reported
independently of supervisor status. Cancellation or parent-pipe EOF triggers
process-group cleanup before the supervisor releases its inherited acquisition
lease. Linux uses subreaper adoption to reap orphaned group members. Other Unix
hosts rely on their orphan reaper while the supervisor retains its lease until
the group disappears; only Linux lifecycle behaviour is verified here.
Dropping another copy of a lease closes that descriptor without explicitly
unlocking the shared open-file description. A process group provides cleanup for
cooperating descendants; this adapter is not a sandbox for code that creates a
new session or otherwise escapes the group.

The original eight built-in assessors remain pure in-process functions. Runtime
registries bind optional engineering, ASPIC and custom methods to supervised
workers, with startup and execution deadlines, combined-output bounds and POSIX
CPU/address-space limits. Native workers charge CPU allowance from current process
usage. An external command also charges its launcher/startup CPU within the bounded
startup allowance. The immutable executor identity includes the worker executable
hash, operator configuration and protocol/implementation identity. Hash checks
detect changes; they do not attest the dynamic loader or unpinned dependencies.

The library's `WorkerCommand` and `MethodExecutor::register_command` replace
Python's `package.module:function` factories. They register an explicit trusted
argv, environment, working directory and typed method contract. The bundled
worker first applies resource limits, then execs that command. External code must
implement `eal-method-worker/1`, emit the exact ready frame, read one strict request
frame, and emit one strict result frame. No source-level code loading or implicit
module search is provided. See [the protocol](method-worker.md).

The pure libraries still permit direct native callbacks for embedding and direct
finite solvers. Their metadata makes no process-deadline claim. The runtime's
ASPIC snapshot adaptor supplies a supervised executor while preserving checked
provenance, input/output contracts and independent authored results.
