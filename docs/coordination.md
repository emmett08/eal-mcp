# Coordinate independent work and retain uncertain outcomes

[Documentation](index.md) · [Applications](applications.md) · [Execution tickets](tickets.md)

An engineer may need one worker to assess a registered argument and another to
review what that assessment supports. EAL's coordinator makes the handoff
explicit. It freezes the task, reserves each effect before execution and retains
the first response before admission. If a worker or transport disappears, the
journal distinguishes work that never started from work whose outcome is
unknown. That distinction determines whether continuation is safe.

`eal-coordinate` provides a registered assessor/reviewer workflow and a separate
participant-debate protocol. The first checks responses against persisted EAL
findings. The second routes bounded application payloads over a declared graph.
Use [execution tickets](tickets.md) when the work itself consists of authoring,
integrating, exporting and executing an argument.

## Pin a registered question

Install the companions into the same prefix as described in
[installation](getting-started.md). Production companions resolve beside the current
executable, without a `PATH` fallback. Library clients can select an installation
through explicit `HostPaths` in the `_with_host` constructors.

The run specification is a strict JSON object with `server` and `task`, and only
optional `budgets` and `transport` alongside them:

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

`server` has exactly the five illustrated fields. Nullable values use their
native defaults, including `workspace/.eal/runs.sqlite3` for the database. Paths
resolve when the specification is admitted. Available native method selections
are `builtin`, `engineering`, `aspic` and `all`. Existing runs accept no
command-line server overrides.

The task has exactly `task_id`, `question`, `entry_id`, `claims`, `profile` and
`requirements`, with one to sixteen unique claims. Here `profile` is an existing
coordination JSON field selecting a task-correspondence check. It is separate
from the EAL/4 grammar, which has one source-level semantic contract.

`registered_claims/1` requires exactly `{"context": {...}}`. It checks the selected
claims, exact registered context, source/import-bundle revision and method
registry. Registration alone cannot establish that a claim's prose answers the
question, so that correspondence remains part of the engineer's review.

`api_load/1` also requires complete `workload`, `criteria` and `claim_outputs`
objects. It checks all five API workload fields, the pinned report digest,
freshness, installed API method contract and every direct route's requested
`passes` or `fails` predicate. Its finite criteria are at least 100 requests,
sample p95 at most 200 ms and error rate at most 1%. Changing numbers in the task
does not change those installed method semantics.

Start in an empty run directory:

```sh
eal-coordinate start --run RUN --spec run-spec.json
eal-coordinate status --run RUN
```

`start` pins the task, context, source, method fingerprint and actual pin time.
It acquires no evidence and starts no worker. A new pin needs a new directory,
and a source change invalidates the earlier correspondence. The alternative
`start --demo MANIFEST --task-id ID` selects an existing trusted demo task without
preparing or running it. `--wall-seconds INTEGER` overrides that start-time
budget.

## Make the assessment and review separate calls

Reserve an assessor before launching its independent context:

```sh
eal-coordinate reserve --run RUN --role assessor \
  --execution-mode actual_host_agent --context-out assessor-context.json
eal-coordinate query --run RUN --call-id RESERVED_CALL_ID
eal-coordinate record --run RUN --call-id RESERVED_CALL_ID \
  --response assessor-first-output.json --execution-id ACTUAL_EXECUTION_ID \
  --execution-mode actual_host_agent
```

Launch the external worker only when reservation returns `new: true`. Its context
contains the frozen task, addressed messages, gateway argv and the path to the
exact response schema retained in the run directory. The command
`eal-coordinate --response-schema` prints that schema without creating a run or
database.

The assessor uses its `query` gateway and returns exactly `message` and
`consumed_message_ids`. The message is an `outcome` retaining every selected
assessment reference, or a typed `disposition` retaining a rejection or
incomplete code. After admission, reserve a fresh reviewer with `--role reviewer`.
It retrieves the finding through its own gateway:

```sh
eal-coordinate lookup --run RUN --call-id REVIEWER_CALL_ID --finding-id FINDING_ID
```

The reviewer returns exactly `answer` and `consumed_message_ids`. The answer
contains `task_digest`, `finding_id`, `claims`, `scope`, `conclusion` and
`qualification`. Each claim retains its actual assessment ID, status and time.
Unassessed fields stay `null`. Scope preserves the requested context and task
requirements, and qualification is `conditional_on_registered_argument`.

| Conclusion | Meaning |
| --- | --- |
| `assessed` | Every selected claim has a matching persisted EAL assessment. |
| `out_of_scope` | Source, context, workload or criterion correspondence failed. |
| `unresolved` | Evidence or an effect's complete receipt is unavailable. |

Supported, contested and unsupported are all assessed EAL outcomes. A null
unassessed field cannot become `unsupported`, and unsupported does not establish
the opposite claim. Disposition codes are `source_changed`, `context_mismatch`,
`workload_mismatch`, `criteria_mismatch`, `evidence_unavailable` and
`effect_outcome_unknown`.

The gateway performs real MCP stdio calls. Registered exposure supplies
`eal_sources` and `eal_assess_known` for the pinned entry. A separate operator
session retrieves persisted packets and explanations. Before acquisition and
after retrieval, the gateway checks source, methods, context, time, status and
agreement between the wrapper, packet and explanation. Lookup retrieves and
checks the persisted finding again. Workers cannot override source, context,
assessment time, reuse policy or server configuration.

## Run a command worker

Automated execution takes an operator-selected argv directly:

```sh
eal-coordinate execute --run RUN --execution-mode provider \
  --worker-argv '["/absolute/provider-worker"]' \
  --worker-environment /private/worker-environment.json
```

Each role receives one JSON context on stdin in a fresh supervised process. The
optional environment file supplies its complete environment. Values remain out
of exported execution evidence. `software_test` returns the direct role response.
Live command modes return this receipt envelope:

```json
{"response": {}, "execution": {"execution_id": "external-id", "model": null, "usage": null}}
```

`response` above stands for the complete role-specific object. Optional usage
contains nonnegative integer token counts. External `record` accepts an unwrapped
`actual_host_agent` response, while `provider` requires the envelope. Separate
roles need distinct execution IDs. A command cannot use its local call ID or a
`command:` ID as an external execution identity. Receipts are retained as
`operator_command_assertion`. They record a host assertion, without independently
authenticating a provider or establishing what a model inspected.

`CommandWorker` bounds input context at 4 MiB and combined output at up to 16 MiB.
The remaining run allowance can shorten its configured deadline. JSON admission
also enforces depth 128 and 100,000 nodes. The supervisor holds the coordination
run lease until it has stopped and reaped the process group. This lease differs
from the execution workspace's observation lease and the coordinator's
`lifecycle.lock`, so a gateway can acquire evidence while its worker is running.

Failures distinguish cancellation, timeout, output overflow, nonzero exit,
startup failure, supervisor failure and invalid response. They retain bounded
partial stdout and available execution status. Live stderr is omitted.
Software-test stderr is bounded and redacts explicit environment values. The
parent observes the supervisor PID instead of trusting a worker-supplied value.

The development example `coordination_software_worker` mechanically authors both
role responses through their supplied gateways. It exercises integration under
`software_test`, without supplying evidence of model execution or quality.

## Continue delivery without repeating uncertain effects

`coordination.sqlite` is authoritative. A transaction retains exact first-response
bytes as base64 and SHA-256, execution evidence, the parsed response, admission
diagnostics and an immutable reply outbox. Publication and message acknowledgement
follow commitment. Malformed, rejected and interrupted first attempts remain in
the journal. Later calls cannot replace them.

Delivery is at least once. An identical stable message ID deduplicates, while
changed content under the same ID fails. Continuation reconciles publication,
acknowledgement and transport accounting after lost confirmations:

```sh
eal-coordinate resume --run RUN
eal-coordinate cancel --run RUN
eal-coordinate export --run RUN --output retained-evidence.json
```

`resume` never repeats an uncertain worker or assessment effect. An assessment
without a committed complete receipt yields `effect_outcome_unknown`, no
assessment references and unknown counts. Cancellation prevents new effects while
allowing committed delivery to finish. Export is a detached view of retained
state.

Status separates ready roles, `awaiting_worker_result`, `delivery_pending`,
`completed`, `completed_unassessed`, `failed`, `cancelled` and `expired`. Failed,
cancelled or expired runs and rejected records exit 1. Successfully retained
unassessed answers and uncertain reservations return their typed JSON with exit
0. Usage errors exit 2. Other failures emit a native error object on stderr and
exit 1.

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

All budgets are positive integers. Worker time cannot exceed wall time. Gateway
and MCP quotas are reserved before each attempt, including failures, and survive
restart. A successful transport response only counts as a successful tool result
when it contains the required successful structured object. Traces distinguish
`not_called`, `started`, `success` and `interrupted_or_failed`.

Specification, demo and environment files are regular-file reads bounded at
4 MiB. External response files have a 16 MiB hard bound. A response above the
frozen output budget but within that hard bound is retained and rejected. A
larger file fails before commitment. Coordination JSON is bounded at depth 128,
with narrower task-specific shape limits. Externally launched agents also remain
subject to their host's own execution and token controls. Missing usage receipts
do not produce estimated billing.

## Select mailbox transport

The default `{"kind":"json"}` uses a durable local SQLite mailbox. To use the
native persistent MCP mailbox, select
`{"kind":"lab","runtime":"/absolute/installed/bin"}` in the specification or
`start --runtime /absolute/installed/bin`. This installation directory must contain
`eal-mas-server`.

`eal-mas-server --database ABSOLUTE_PATH` exposes mailbox, reservation, completion
and codec operations through the Rust MCP SDK. The pinned nom codec runs in
process. `mas-protocol-codec` also provides bounded one-shot and line-oriented
interfaces. Complete JSON, canonical wire text and complete parsing are admitted
before the mailbox transaction. Domain contracts use `EAL/MAS-coordination/3`,
while the independent transport envelope remains version 1.

The native bridge exposes `mas_create_run`, `mas_receive`, `mas_ack`, `mas_status`,
`mas_encode`, `mas_validate`, `mas_reserve_call`, `mas_finish_call`,
`mas_runtime_info` and `mas_send_envelope`. Encoding, runtime inspection and
rejected admission create no mailbox database. Actor identity remains a trusted
local host assertion.

A lazy SDK connection owns a dedicated Tokio thread and serialises calls. It
checks discovered schemas and invokes each operation once. Local/schema rejection
and ordinary `{ok:false,error:{code,message}}` replies preserve a healthy session.
Timeout, transport failure, malformed envelopes, tool-level failure or `CODEC_IO`
close it. Reopening is explicit and requires reconciliation of durable state.
Deadlines cover admission, first-session startup, discovery and invocation.
Cleanup retains process ownership through a bounded allowance after cancellation.
No daemon or remote endpoint is installed.

## Route participant debates

A debate session applies a declared communication graph to bounded application
payloads. The coordinator owns participant identities, round barriers and routing.
An external host performs the model or deterministic work. EngArg uses this
interface while retaining its prompts, completion records and credentials in its
own process. Accepted frames establish protocol conformance, not evidence that a
payload's conclusions are true.

Start a new journal and exchange one newline-terminated JSON object per line on
stdin/stdout:

```sh
eal-coordinate debate-session --run /absolute/path/to/new-debate
```

A start frame selects participants, peer rounds, topology and a payload budget:

```json
{"kind":"start","spec":{"schema_version":"eal.debate/1","debate":{"participants":4,"rounds":1,"topology":{"kind":"ring"},"adaptation":{"kind":"static"}},"max_payload_bytes":131072}}
```

The first dispatch is independent round 0, with no previous payload:

```json
{"kind":"dispatch","dispatch":{"round":0,"calls":[{"participant":0,"round":0,"edges":[],"incoming":[],"peers":[]},{"participant":1,"round":0,"edges":[],"incoming":[],"peers":[]},{"participant":2,"round":0,"edges":[],"incoming":[],"peers":[]},{"participant":3,"round":0,"edges":[],"incoming":[],"peers":[]}]}}
```

Execute every call and return exactly one response for each dispatched participant
and round. Response order may differ from participant order:

```json
{"kind":"results","result":{"round":0,"responses":[{"participant":0,"round":0,"payload":{"observation":"A"}},{"participant":1,"round":0,"payload":{"observation":"B"}},{"participant":2,"round":0,"payload":{"observation":"C"}},{"participant":3,"round":0,"payload":{"observation":"D"}}]}}
```

The next dispatch uses only the complete accepted previous round. In a forward
ring, participant 0 receives participant 3's payload and its own previous payload.
Each call records the full edge set and incoming indices. Earlier information may
travel inside a later payload, so immediate-input routing is not a secrecy
boundary. After the peer round in this example, completion is:

```json
{"kind":"completed","rounds":2,"calls":8}
```

`rounds` includes independent round 0. `calls` counts participant work. Synthesis,
criticism and adjudication remain with the application. The host must validate
terminal counts and child exit status.

Supported topology kinds are `fully_connected`, `grouped`, `chain`, `ring`,
`star`, `tree`, `layered`, `sparse` and `bipartite`, defined by the shared
`eal-coordination-contracts` crate. Static connectivity reuses edges. Scheduled
connectivity replaces edges after the first peer round. Participants remain fixed.

The `eal.debate/1` frame bound is 32 MiB including newline. Participant budgets
range from one byte through 1 MiB, with depth 64 and at most one million values.
A dispatch exceeding the frame bound is rejected before copying or publishing
peer data. Missing, duplicate or unexpected participants, wrong rounds, unknown
fields, oversized payloads, malformed frames and input loss fail the session.
Accepted earlier rounds remain retained, while calls reserved for the incomplete
round remain uncertain.

Exact incoming frames are retained before admission and outgoing frames before
publication. Immutable files have byte counts and SHA-256 digests in
`debate.sqlite`. Effects bind participant/round pairs to dispatch and response
frames. Accepted payloads retain their own encoded JSON digest. The journal
records host-submitted data without authenticating provider execution.

Only one new session may claim a run directory. A later session fails, and an
uncertain external call is never automatically repeated. Inspect retained state
before choosing a separately identified new run:

```sh
eal-coordinate debate-status --run /absolute/path/to/new-debate
eal-coordinate debate-export --run /absolute/path/to/new-debate \
  --output /absolute/path/to/debate-export.json
```

Status separates expected, committed and uncertain calls. Export includes the
frozen specification, effects, events and immutable frame references. Inspection
rechecks bounded reads, byte counts and digests. Export files are immutable, so a
changed journal requires a new output path.

## Check the installed workflow

The [installed acceptance guide](development.md#installed-acceptance) runs
`eal-smoke coordination` against actual companions, a fresh synthetic observation
and an explicit software worker. It checks both mailboxes, scoped responses,
assessment identities and times, exact response bytes, reuse and continuation
without additional effects. An injected receipt-commit failure verifies the
unresolved outcome after an assessment has persisted.

The coordinator is a trusted single-host process on a local filesystem. Role
restrictions do not isolate shared filesystem permissions or authenticate
measurements. Content identities detect inconsistent retained views under that
trust model. A writer able to replace every database can replace the evidence
of correspondence too. Model quality, cost and production adequacy require
separate observations from the intended operating environment.
