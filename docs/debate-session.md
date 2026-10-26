# External-host debate sessions

`eal-coordinate debate-session` executes a selected communication graph over bounded application payloads. The coordinator validates the configuration and owns participant identities, round barriers and routing. An external host performs the model or deterministic work and returns each complete round. EngArg uses this interface while retaining its provider prompts, completion records and credentials in its own process.

This session is separate from the registered-argument assessor/reviewer workflow. Its payloads are application data. Acceptance establishes protocol conformance and preserves the received data. EAL evidence acquisition, method execution and registered claim assessment retain their existing contracts.

## Start and exchange frames

```sh
eal-coordinate debate-session --run /absolute/path/to/new-debate
```

The run directory must be empty. Standard input and standard output carry one JSON object per line. Every frame requires a terminating newline. The `eal.debate/1` contract bounds a frame, including its newline, to 32 MiB. Participant payloads have their own limit from 1 byte through 1 MiB, a maximum nesting depth of 64 and at most one million values. The coordinator rejects a dispatch that would exceed the frame bound before copying or publishing peer payloads.

Send a start frame such as:

```json
{"kind":"start","spec":{"schema_version":"eal.debate/1","debate":{"participants":4,"rounds":1,"topology":{"kind":"ring"},"adaptation":{"kind":"static"}},"max_payload_bytes":131072}}
```

The coordinator first returns an independent round. Its calls omit `previous` because no earlier payload exists:

```json
{"kind":"dispatch","dispatch":{"round":0,"calls":[{"participant":0,"round":0,"edges":[],"incoming":[],"peers":[]},{"participant":1,"round":0,"edges":[],"incoming":[],"peers":[]},{"participant":2,"round":0,"edges":[],"incoming":[],"peers":[]},{"participant":3,"round":0,"edges":[],"incoming":[],"peers":[]}]}}
```

Execute every dispatched call and send exactly one response for each participant. Response order may differ from participant order. Each response must retain its dispatched participant and round:

```json
{"kind":"results","result":{"round":0,"responses":[{"participant":0,"round":0,"payload":{"observation":"A"}},{"participant":1,"round":0,"payload":{"observation":"B"}},{"participant":2,"round":0,"payload":{"observation":"C"}},{"participant":3,"round":0,"payload":{"observation":"D"}}]}}
```

The next dispatch uses only the complete accepted previous round. In a forward ring, participant 0 receives participant 3's payload and its own previous payload. Each call records the complete edge set and its incoming participant indices. A payload may carry information received in earlier rounds, so the graph constrains immediate inputs rather than imposing a secrecy boundary.

After all peer rounds, the coordinator emits this terminal frame and exits successfully:

```json
{"kind":"completed","rounds":2,"calls":8}
```

`rounds` includes the independent round 0. `calls` counts participant work only. Application synthesis, criticism and adjudication remain outside the session. The host must validate the terminal counts and the child process exit status.

The supported topology kinds are `fully_connected`, `grouped`, `chain`, `ring`, `star`, `tree`, `layered`, `sparse` and `bipartite`. Their typed configuration and scheduling rules come from the shared `eal-coordination-contracts` crate. Static connectivity reuses the same edge set. Scheduled connectivity supplies explicit replacements after the first peer round. Participants remain fixed during the session.

## Failure and retained state

A missing, duplicate or unexpected participant rejects the complete result round. A wrong round, unknown field, oversized payload, malformed frame or input loss also fails the session. Failure emits a `failed` frame where the output channel remains usable and returns a nonzero exit status. Previously accepted rounds remain retained. Calls reserved for an incomplete round remain uncertain.

The coordinator retains the exact input bytes before admission and exact output bytes before publication. Immutable frame files have a byte count and SHA-256 digest in `debate.sqlite`. Effect records bind each participant and round to its dispatch frame and accepted response frame. Accepted payloads also retain an exact encoded JSON SHA-256 digest. The journal records opaque host-submitted data and supplies no independent authentication of provider execution.

A persistent exclusive ownership file allows one new session to claim a run directory. A later invocation against that directory fails. An interrupted or uncertain external call is never automatically retried. The host can inspect retained results before choosing a separately identified new run.

```sh
eal-coordinate debate-status --run /absolute/path/to/new-debate
eal-coordinate debate-export --run /absolute/path/to/new-debate \
  --output /absolute/path/to/debate-export.json
```

Status reports expected, committed and uncertain calls. Export includes the frozen specification and its digest, effect receipts, events and immutable frame references. Inspection bounds frame reads and checks the retained byte counts and digests. Export files are immutable publications. Use another output path after the journal changes.

Registered source validation and evidence execution remain available through the existing [coordination workflow](coordination.md).
