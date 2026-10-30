# Preserve evidence across collection and reassessment

`eal-runtime` supplies the native runtime beneath the production CLI,
catalogues and transports. It keeps IO, clock lookup and process execution out
of `eal-core` and `eal-aspic`. The shared `ReasoningService` implements validation,
formatting, evidence planning, collection, executable-program assessment, stored-collection assessment,
supervised ASPIC compilation, durable explanation, catalogue revision history
and registered assessment. See [applications](applications.md) for production
commands and MCP exposure, and [packets](packets.md) for bounded projections.

## Choose the execution boundary

The [first-run guide](getting-started.md) installs the runtime and its companions.
Use `eal-host run` for an executable package, or the staged `eal` operations in
[applications](applications.md) when collection and assessment need separate
inspection. The development driver `eal-runtime` uses the same bootstrap and
runtime service.

The production `eal` file commands resolve source and context files inside their
workspace. Service imports also resolve from the workspace root, including when
the primary file is in a subdirectory. In contrast, `eal-host run` reads its
explicit package source and context paths before sending their values to MCP.
Those primary files can live outside the run-store workspace. Imported modules
still need to be available under the service workspace.

The pure `eal-core` file-mode driver supplies the source file's origin to its
composition API. Manifest, database and executable locations remain operator
configuration on these entry points.

The runtime resolves `eal-process-supervisor` and `eal-method-worker` beside its
own executable. Operator options `--supervisor` and `--method-worker` select
explicit alternatives before their identities are captured. Install companions
from the same release because their private transport is version-coupled.

Assessment uses current UTC time unless `--now` selects a timezone-aware instant.
Invalid validation and failed observations have distinct diagnostics from invalid
commands. Incomplete computation supplies no accepted or negative conclusion.
The [application guide](applications.md#cli-operations) gives command exit status
and result interpretation.

## Bind a collector to an evidence request

A TOML manifest binds declared tool names and versions to operator-owned code
or retained observations. The evidence declaration supplies the measurement
parameters. The manifest owns the executable and its permissions. All five
transports share the [direct binding contract](direct-tools.md).

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

Omitting `inherit_env`, or supplying an empty list, inherits no process
environment variables. The optional `env` table overlays the selected environment.
`pinned_files` accepts up to 32 distinct `{path, sha256}` entries for command
dependencies. Relative binding resources resolve beside the manifest and stay within that package. Absolute pinned paths
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
up to the operator's worker limit. A serial binding is a barrier. Returned records
keep requested order. Successful and failed observations are durable. Raw output
is replaced by SHA-256 digests and byte counts. A successful measured negative
value remains distinct from a failed collector.
Collector envelopes obey the binding's output byte cap. The semantic
`record_bytes` allowance is applied later when assessing each durable observation.

## Persistence and reuse

The default database is `.eal/runs.sqlite3`. A store instance anchors the parent,
database and 32-byte random key identities. It checks ownership, private mode,
regular type, link count and no-follow policy around fresh SQLite connections.
Database and sidecar validation uses filesystem metadata without opening extra
descriptors. Existing database construction follows the same rule, preserving
SQLite's process-level POSIX locks during concurrent reads and writes. Only
creation of a previously absent database uses a separate file descriptor.
The persistent initialisation lock serialises schema/key creation. Atomic batch
insertion preserves immutable IDs. Key installation fsyncs the file and directory.
Replacement of a live instance's database or key fails. The private-store policy
currently requires POSIX.  Windows ACL verification is not implemented.

The observation-record/1 HMAC domain is `EAL/2`, independent of the source language version.
Binding and exact effective collector environment identities use the private
store key. Runtime metadata records keyed environment identity without copying
the environment or key bytes. Trusted collector output is retained as its declared
observed value.
Stored payloads have a separate fixed 64 MiB defensive envelope with depth 128.
Collection admission enforces a 32 MiB aggregate limit and independent
per-observation budgets.

`ObservationRebinder` selects acquisition-compatible successful records from the
indexed store and verifies the exact stored original before preparing a detached
record. It requires current binding and, for commands, execution-environment
identity. It retains original `collected_at`, `observed_at` and origin identity.
Rebinding never refreshes measurement age. Freshness and evidence predicates are
assessed later at the assessment instant. The registered assessment host plans
the complete claim closure, chooses compatible observations individually,
collects missing observations and atomically stores its rebinding batch. See
[registered assessment boundaries](applications.md).

Stored-original comparison uses typed canonical equality. Integer `1` and float
`1.0` have distinct identities. Registry TOML has a 1 MiB admission ceiling.

## Keep ownership until execution stops

Each command has a separate supervisor. Its absolute monotonic deadline starts
before configuration transfer. Nonblocking IO bounds combined stdout/stderr and
handles a collector that never reads stdin. Collector status is reported
independently of supervisor status. Cancellation or parent-pipe EOF triggers
process-group cleanup before the supervisor releases its inherited acquisition
lease. Linux uses subreaper adoption to reap orphaned group members. Other Unix
hosts rely on their orphan reaper while the supervisor retains its lease until
the group disappears. Only Linux lifecycle behaviour is verified here.
Dropping another copy of a lease closes that descriptor without explicitly
unlocking the shared open-file description. A process group provides cleanup for
cooperating descendants. This adapter is not a sandbox for code that creates a
new session or otherwise escapes the group.

The ten built-in assessors are pure in-process functions. Runtime
registries bind optional engineering, ASPIC and custom methods to supervised
workers, with startup and execution deadlines, combined-output bounds and POSIX
CPU/address-space limits. Native workers charge CPU allowance from current process
usage. An external command also charges its launcher/startup CPU within the bounded
startup allowance. The immutable executor identity includes the worker executable
hash, operator configuration and protocol/implementation identity. Hash checks
detect changes. They do not attest the dynamic loader or unpinned dependencies.

The library's `WorkerCommand` and `MethodExecutor::register_command` register an explicit trusted
argv, environment, working directory and typed method contract. The bundled
worker first applies resource limits, then execs that command. External code must
implement `eal-method-worker/1`, emit the exact ready frame, read one strict request
frame, and emit one strict result frame. No source-level code loading or implicit
module search is provided. See [method execution](method-execution.md).

The pure libraries still permit direct native callbacks for embedding and direct
finite solvers. Their metadata makes no process-deadline claim. The runtime's
ASPIC snapshot adaptor supplies a supervised executor while preserving checked
provenance, input/output contracts and independent authored results.


### Process protocol for embedded clients

The `eal-process` crate owns the supervisor protocol, bounded client and cleanup
companion. It depends on neither the language nor runtime storage. `Supervisor`
accepts exact argv, an explicit environment, working directory, input bytes,
deadline and combined output allowance. `Retention` transfers a duplicated
ownership handle. The runtime supplies acquisition leases through that narrow
capability while retaining acquisition and lock policy itself.

Install the canonical `eal-process-supervisor` companion. A missing or incompatible
companion fails before launch. The package exposes no unsupervised fallback.

`StreamLimits` adds separate stdout and stderr budgets through
`Supervisor::run_with_stream_limits` and its cancellation variant.
`CommandRequest` also supports a combined-output allowance without separate stream limits.
Each byte must fit both its stream allowance and the combined allowance before capture.
The client checks announced stream lengths before accepting their payload.

Combined-only requests use protocol version 1.
Requests with independent stream limits explicitly carry `protocol_version = 2`.
Their optional `stream_limits` field is mandatory for that request version.
Earlier companions reject those unknown fields before launching a collector.
Response framing remains unchanged.
Applications using version 2 must install the companion from the same release.

### Duplex protocol sessions

Protocol version 3 forwards a bounded bidirectional byte stream for MCP and
other interactive protocols. The same companion owns the child process group,
Linux subreaper, signal handling and inherited acquisition descriptor. It checks
an independent session deadline and cumulative input, stdout and stderr bounds.
The protocol client owns one socket. Closing it, cancelling the session or losing
the host process makes the companion stop descendants before releasing the lease.
The SDK supplies MCP protocol interpretation. The supervisor interprets no JSON-RPC.

`Supervisor::duplex` returns the raw stream and an explicit RAII process owner.
The owner requests cleanup without killing the companion prematurely. If cleanup
exceeds the short notification allowance, the companion and its lease remain alive
and a reaper retains exit ownership. Unsupported or absent companions fail without
launching an unsupervised server.


## Select optional acquisition adapters

`eal-adapters` owns bounded HTTP transport, command construction, pinned artifact
import, retained-review correspondence and specialised trace inspection. It
depends on the pure collector contract and process supervisor. The language core
has no dependency on the adapter package, so a source's meaning remains separate
from the deployed way of acquiring its measurements.

Command adapters use `EAL_COMMAND_SUPERVISOR` when it names an absolute companion
path. Otherwise they resolve the installed companion beside the application or
on an absolute `PATH` directory. The supported companion names are
`eal-process-supervisor` and the packaged EngArg facade
`engarg-process-supervisor`. Integration harnesses in a Cargo `deps` directory
also check the parent build directory. Cancellation retains supervisor ownership
until descendant cleanup.

The named `profiles` and `scoped` modules describe optional existing Kubernetes
and Grafana integrations. They declare their supported input names, predicates,
credential-variable names, child environment, versions and allowances. Choose a
general [direct binding](direct-tools.md) for other domains. These presets grant
no authority beyond the operator's selected operation.

Retained artifact import resolves a relative base directory once to its canonical
target. Relative artifacts stay beneath that directory and reject parent traversal
and leaf symbolic links. Absolute pinned paths are independent operator grants.
Their admission does not require an otherwise unused relative base to exist.

Trace inspection selects matching attributes on the same span.
`inspect_trace_with_keys` accepts operator-selected attribute keys, while the
DPE rollout selector supplies a specialised preset. A match establishes the
specified span relationship. Provenance, response completeness, current deployment
state and fleet coverage require their own observations.

Continue with [packets](packets.md) to share a result, or return to the
[documentation path](index.md).
