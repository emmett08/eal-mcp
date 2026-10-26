# Durable registered-task coordination

`eal-coordinate` runs a single-host assessor/reviewer process over registered
EAL arguments. An operator freezes the entry, selected claims, correspondence
profile, server configuration and finite budgets. The assessor requests a
checked finding and authors a handoff; a separate reviewer retrieves that
finding and authors an answer. Admission checks those authored responses rather
than supplying their conclusions.

Observation collection, compatible reuse, original measurement ages and EAL
claim statuses remain the responsibility of the registered application service.
The gateway uses actual MCP stdio: registered exposure supplies discovery and
`eal_assess_known`; operator exposure retrieves persisted packets and explanations.
Workers receive role-specific gateway arguments and cannot override source,
context, assessment time, reuse policy or server configuration.

## Installation and trusted configuration

Install `eal-process`, `eal-runtime`, `eal-cli`, `eal-mcp` and `eal-coordination` into the same
prefix; see the [build instructions](../README.md#build-and-use). Production
companions are resolved beside the current executable, without a `PATH`
fallback. Library callers can supply explicit `HostPaths` to the `_with_host`
constructors for a selected installation.

The run specification is a strict JSON object containing `server` and `task`,
with optional `budgets` and `transport` only. For example:

```json
{
  "server": {
    "workspace": "/absolute/workspace",
    "registry": "/absolute/workspace/tools.toml",
    "database": null,
    "methods": "builtin",
    "limits": null
  },
  "task": {
    "task_id": "readiness-review",
    "question": "What do the selected registered checks support?",
    "entry_id": "readiness",
    "claims": ["ready", "challenged"],
    "profile": "registered_claims/1",
    "requirements": {"context": {"cluster": "staging"}}
  }
}
```

The server has exactly those five fields. Optional values can be `null`; the
default database is `workspace/.eal/runs.sqlite3`. Paths resolve when the
specification is admitted. Native method selections are `builtin`, `engineering`,
`aspic` and `all`. Existing runs accept no
command-line server overrides.

Shipped pure method workers use an empty environment consistently across CLI,
MCP and worker gateway processes. Custom `WorkerCommand` environments and
collector environment policies remain explicit and retain their exact identities.
Catalogue refresh records a new immutable revision when its method identity
changes. Existing assessments retain their original fingerprint. A task pin
requires the current registered method identity.

A task has exactly `task_id`, `question`, `entry_id`, `claims`, `profile` and
`requirements`. It selects one to sixteen unique claims. Registration and task
selection do not establish that their prose answers the operator's question.

`registered_claims/1` requires exactly `{"context": {...}}` and checks the selected
claims, exact registered context, source/import-bundle revision and method
registry. `api_load/1` additionally requires complete `workload`, `criteria` and
`claim_outputs` objects. It checks all five API workload fields and the pinned
report digest, source freshness, installed API method contract and every direct
claim route's requested `passes` or `fails` predicate. Its installed finite
criteria remain at least 100 requests, sample p95 at most 200 ms and error rate
at most 1%. Changing task numbers cannot change those method semantics.

`start` pins the task, registered context, source digest, method-registry
fingerprint and actual pin time. It requires an empty run directory. Starting
does not acquire evidence or execute a worker. A new task pin requires a new
run directory; a source change invalidates earlier correspondence.

## Commands and worker responses

```sh
eal-coordinate start --run RUN --spec run-spec.json
eal-coordinate status --run RUN
eal-coordinate resume --run RUN
eal-coordinate cancel --run RUN
eal-coordinate export --run RUN --output retained-evidence.json
```

`start --demo MANIFEST --task-id ID` selects one exact task from an existing
trusted demo manifest instead of `--spec`. It does not prepare or run a demo.
`--wall-seconds INTEGER` overrides that one budget when starting.

Reserve an external assessor before launching its independent context:

```sh
eal-coordinate reserve --run RUN --role assessor \
  --execution-mode actual_host_agent --context-out assessor-context.json
eal-coordinate query --run RUN --call-id RESERVED_CALL_ID
eal-coordinate record --run RUN --call-id RESERVED_CALL_ID \
  --response assessor-first-output.json --execution-id ACTUAL_EXECUTION_ID \
  --execution-mode actual_host_agent
```

Launch a worker only when reservation returns `new: true`. Its context contains
the pinned task, addressed messages, native gateway argv and a path to the
byte-exact published response schema retained privately in the run directory.
The additive `eal-coordinate --response-schema` prints that embedded schema
without creating a run or database.

The assessor uses its `query` gateway and returns exactly `message` and
`consumed_message_ids`. Its message is an `outcome` retaining every selected
assessment reference, or a typed `disposition` retaining the finding's rejection
or incomplete code. After the accepted handoff, reserve a fresh reviewer with
`--role reviewer`. It uses:

```sh
eal-coordinate lookup --run RUN --call-id REVIEWER_CALL_ID --finding-id FINDING_ID
```

The reviewer returns exactly `answer` and `consumed_message_ids`. The answer
contains `task_digest`, `finding_id`, `claims`, `scope`, `conclusion` and
`qualification`. Every claim retains its actual assessment ID, status and time;
unassessed fields remain `null`. Scope retains the complete requested context
and profile requirements. Qualification is `conditional_on_registered_argument`.

| Conclusion | Retained meaning |
| --- | --- |
| `assessed` | Every selected claim has a matching persisted EAL assessment. |
| `out_of_scope` | Source, context, workload or criterion correspondence failed. |
| `unresolved` | Evidence or an effect's complete receipt is unavailable. |

Supported, contested and unsupported are all EAL outcomes. An unassessed `null`
cannot be replaced by `unsupported`; unsupported does not prove the opposite
claim. The closed disposition codes are `source_changed`, `context_mismatch`,
`workload_mismatch`, `criteria_mismatch`, `evidence_unavailable` and
`effect_outcome_unknown`.

Automated execution uses an operator-selected argv without an implicit shell:

```sh
eal-coordinate execute --run RUN --execution-mode provider \
  --worker-argv '["/absolute/provider-worker"]' \
  --worker-environment /private/worker-environment.json
```

Each role receives one context object on stdin in a fresh supervised process.
`software_test` returns the direct role response. Live command modes require
`{"response": {...}, "execution": {"execution_id": "external-id", "model": null,
"usage": null}}`. Usage, when supplied, contains nonnegative integer token
counts only. External `record` permits a direct `actual_host_agent` response,
while `provider` requires its matching receipt. Separate roles require distinct
execution identities. Receipt values are operator assertions, not authenticated
provider evidence or proof that a model inspected its messages.

The native development example `coordination_software_worker` mechanically
authors both role responses through their supplied gateway argv. Supply its
absolute executable path to `execute --execution-mode software_test`.
It exercises native integration and supplies no evidence of model execution or
model quality.

The optional environment file supplies the complete explicit worker environment.
Values are excluded from exported execution evidence. Live stderr is omitted;
software stderr is bounded and redacts supplied values. Interruption preserves
bounded partial stdout and its execution category; command exceptions do not
expose private argv or environment text.

## Durable effects, delivery and limits

`coordination.sqlite` is authoritative. Worker and assessment effects are
reserved before execution. One transaction retains exact first-response bytes
as base64 plus SHA-256, execution evidence, parsed response, admission diagnostics
and immutable reply outbox. Publication and incoming-message ACK follow the
commit. Malformed, rejected and interrupted first attempts remain retained;
later calls cannot replace or repair them. Exports are detached views.

Delivery is at least once. Identical stable message IDs deduplicate; changed
content under one ID fails. `resume` reconciles committed publication, ACK and
transport accounting after lost confirmations. It never repeats an uncertain
worker or assessment effect. An assessment without a committed complete receipt
produces `effect_outcome_unknown`, no assessment references and unknown counts.
Cancellation prevents new effects while allowing committed delivery to finish.

Status distinguishes ready roles, `awaiting_worker_result`, `delivery_pending`,
`completed`, `completed_unassessed`, `failed`, `cancelled` and `expired`.
Failed, cancelled or expired results and rejected records exit with status 1.
Successful unassessed answers and uncertain reservations retain their typed JSON
status with exit 0. Usage errors exit 2. Other failures emit a native error
object on stderr and exit 1. Successful exports return exit 0.

| Frozen budget | Default | Maximum |
| --- | ---: | ---: |
| `max_worker_calls` | 2 | 2 |
| `max_gateway_calls_per_role` | 4 | 64 |
| `max_mcp_calls_per_role` | 64 | 256 |
| `max_messages` | 8 | 100 |
| `max_bytes` | 65536 | 1048576 |
| `max_worker_output_bytes` | 65536 | 1048576 |
| `wall_seconds` | 900 | 3600 |
| `worker_seconds` | 180 | Selected `wall_seconds` |

All budgets are positive integers; `worker_seconds` cannot exceed the selected
wall budget. Gateway and MCP quotas are reserved before attempts, including
failures, and survive restart. Commands bound combined stdout/stderr, input
transfer and execution time, and retain process ownership through cleanup.
Externally launched host agents remain subject to their host's runtime and token
controls. Missing usage receipts do not produce estimated billing.

Native CLI specification, demo and environment files are regular-file reads
bounded at 4 MiB. External response files have a 16 MiB hard bound: bytes above
the frozen run output budget but within that bound are retained and rejected;
files above the hard bound fail before record commitment. Coordination JSON has
a hard depth ceiling of 128. Task JSON has narrower structural bounds.

## Native mailbox transport and codec

The default `{"kind":"json"}` uses a durable local SQLite mailbox. Select
`{"kind":"lab","runtime":"/absolute/installed/bin"}` in the run specification,
or `start --runtime /absolute/installed/bin`, for a persistent native MCP mailbox.
That directory must contain `eal-mas-server`. It is an executable installation directory.

`eal-mas-server --database ABSOLUTE_PATH` exposes the frozen mailbox, reservation,
completion and codec operations over the official Rust MCP SDK. The pinned nom
codec runs in process; `mas-protocol-codec` also exposes its bounded one-shot and
line-oriented control interfaces. Combined message admission validates complete JSON, canonical wire
text and complete parsing before the mailbox transaction. Domain contracts keep
`EAL/MAS-coordination/3`; the independent transport envelope remains version 1.

Persistent sessions start lazily and serialise requests. Deadline or terminal
transport failure closes the session without retrying an uncertain operation;
reopen explicitly and reconcile durable state. Native process ownership reaps
owned children on close and parent death. No daemon or remote endpoint is
installed. JSON and MCP mailbox transports preserve the same task and effect
rules; transport syntax alone establishes no model-quality advantage.

## Checks and trust boundary

Native tests exercise both correspondence profiles, actual MCP retrieval, manual
and command workers, assessment identities and times, quotas, cancellation and
recovery. Installed acceptance uses `eal-smoke coordination` with a fresh
synthetic observation and explicit native worker executable.

```sh
cargo build --workspace --locked
cargo build --locked -p eal-coordination --example coordination_software_worker
target/debug/eal-smoke coordination \
  --prefix /absolute/installed/bin \
  --worker /absolute/checkout/target/debug/examples/coordination_software_worker
```

The acceptance command executes two runs with the JSON mailbox and two with the
native persistent MCP mailbox. It checks scoped answers, assessment IDs and
times, exact response bytes and hashes, distinct execution receipts, compatible
observation reuse and replay without new calls or executions. A separate injected
receipt-commit failure checks recovery after an application assessment has
persisted. An exact manual assessor disposition and a reviewer-only command then
complete with `unresolved` and null unassessed fields.

All native children run with `PATH` empty. The command retains exports and
`smoke-result.json` in a fresh workspace. Supply `--workspace DIRECTORY` to
select its location. See [native installed acceptance](native-smoke.md) for
process, stream and deadline bounds.

This is a trusted single-host process on a local filesystem. Role restrictions
do not isolate shared filesystem permissions or authenticate measurements.
Content identities detect inconsistent retained views under that trust model;
they do not defend against a shared writer who can alter all databases. General
model accuracy, latency, cost and production adequacy require separate evidence.

For coordinator-owned graph routing over external-host application payloads, use the separate [debate-session protocol](debate-session.md). Its frozen session and wire-frame journal preserve participant identities and round barriers while the registered assessor/reviewer workflow retains its existing finding checks.
