# Coordination adapters

The coordination layer uses separate adapters for independent worker commands, EAL assessment, and mailbox delivery. The coordinator reserves each effect before invoking an adapter and retains returned bytes before admitting a response. A lost response leaves an unknown outcome; opening another connection does not authorize automatic execution again.

## Worker commands

`CommandWorker` starts one trusted argv for each reserved call. It supplies the configured environment exactly, writes one JSON context on stdin, and retains exact stdout bytes. The context limit is 4 MiB. Combined stdout and stderr are capped while streaming, up to 16 MiB. The remaining run budget shortens the configured worker deadline. Native JSON ingestion additionally enforces 128 levels and 100,000 nodes.

A worker's coordination-run lease directory differs from its execution workspace. The supervisor inherits that run lease and releases it after stopping and reaping the command's process group. The coordinator's `lifecycle.lock` is separate. An EAL gateway can therefore acquire the workspace's observation lease without competing with its worker's run lease.

`software_test` accepts one strict JSON response object. `provider` and `actual_host_agent` command modes require a closed response/execution envelope with an external execution ID and optional model and usage data. A command cannot replace that external ID with its local call ID or a `command:` ID. The receipt is recorded as `operator_command_assertion`; it does not independently authenticate a provider or establish that a model ran. Manual recording retains the source's separate rule allowing an unwrapped `actual_host_agent` response.

Failures distinguish cancellation, timeout, output limit, nonzero exit, startup failure, supervisor failure, and invalid response. They retain bounded partial stdout and the actual collector status when available. The supervisor PID is observed by the parent, rather than supplied in a worker response. Live-command stderr is omitted; software-test stderr is bounded and redacts explicit environment values, including credentials that coincide with the redaction marker.

## EAL gateway

The gateway launches a registered MCP session exposing only the pinned entry. Acquisition uses `eal_sources` followed by `eal_assess_known` with entry and claim IDs. Workers cannot inject source, context, time, or reuse overrides. A separate operator session retrieves the persisted packet and explanation.

The gateway checks the current task before acquisition and after retrieval. It verifies registered discovery, source and method identities, context, assessment time and status, and agreement among the registered wrapper, persisted packet, and persisted explanation. Lookup verifies the complete finding identity and retrieves the persisted content again.

A durable budget gate runs before each MCP call. Traces distinguish `not_called`, `started`, `success`, and `interrupted_or_failed`. A successful transport response is insufficient unless the tool returns a successful structured object.

## Persistent mailbox MCP

`PersistentMcp` owns one lazy official-SDK connection on a dedicated Tokio owner thread. Calls are serialized, discovered schemas are checked before invocation, and tool requests use `call_tool_once`. It never reconnects or repeats an uncertain request. Local/schema rejection preserves a healthy session; transport failure or an operation timeout closes it terminally. Closing before the first call starts no process. Cleanup has a bounded allowance after deadline cancellation.

The native Lab runtime directory contains `eal-mas-server`. It replaces the Python virtual-environment launcher and separate codec subprocess. The server uses the retained Rust wire grammar in process; diagnostics report `codec_mode: in_process`, rather than claiming a codec process or Python SDK isolation. The bridge exposes exactly `mas_create_run`, `mas_receive`, `mas_ack`, `mas_status`, `mas_encode`, `mas_validate`, `mas_reserve_call`, `mas_finish_call`, `mas_runtime_info`, and `mas_send_envelope`.

Ordinary `{ok:false,error:{code,message}}` rejection preserves the session. A malformed envelope, tool-level failure, or `CODEC_IO` closes it. Actor identity remains a trusted local host assertion. Codec admission precedes routing and store creation; encoding, runtime inspection, and rejected admission do not create a mailbox database.

Absolute deadlines cover adapter admission, first-session startup, discovery, and invocation. The source separately budgeted startup and calls in its persistent wrapper; the native API makes the total bound explicit. Both connection types retain the shared MCP client policy, including disabled expired-session replay.

## Shipped method environment

Bootstrap configures the bundled native reasoning-method worker with an empty environment. These executables implement closed pure algorithms and need no inherited variables. The selected empty environment is retained in executor identity, making native method fingerprints consistent across CLI, MCP, and commands that deliberately supply a minimal private environment. Custom `WorkerCommand` environments and collector bindings retain their explicit operator configuration.

This changes fingerprints from earlier supervised native registries that included inherited environment variables. Re-registering a catalogue source creates a revision with the new method identity. Existing assessments retain their original identities; they are not silently relabelled.
