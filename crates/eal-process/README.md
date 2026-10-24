# Bounded process execution

`eal-process` owns the established supervisor protocol, bounded client and independent process-tree cleanup companion.
It depends on neither the EAL language nor runtime storage.

`Supervisor` accepts exact argv, an explicitly supplied environment, cwd, input bytes, a timeout and a combined output byte budget.
`Retention` transfers only a duplicated ownership handle to the companion.
Runtime acquisition leases implement that capability while acquisition and lock policy remain in `eal-runtime`.
The last inherited handle remains alive until supervised descendants have stopped.

Install `eal-process-supervisor` with the application.
The runtime retains `eal-command-supervisor` as a delegating executable compatibility alias until EAL 0.2.
The implementation fails when the companion is missing or incompatible and never falls back to unsupervised execution.
Handwritten code forbids unsafe Rust.

During EAL 0.1, runtime discovery preserves an existing `eal-command-supervisor` path when both executable names are installed.
This keeps explicit coordination companion identity compatible while deployments install the canonical companion.

`StreamLimits` adds separate stdout and stderr budgets through
`Supervisor::run_with_stream_limits` and its cancellation variant.
The existing `CommandRequest` record and combined-output methods remain unchanged.
Each byte must fit both its stream allowance and the combined allowance before capture.
The client checks announced stream lengths before accepting their payload.

Combined-only requests retain the original configuration encoding.
Requests with independent stream limits explicitly carry `protocol_version = 2`.
Their optional `stream_limits` field is mandatory for that request version.
Earlier companions reject those unknown fields before launching a collector.
Response framing remains unchanged.
Applications using version 2 must install the companion from the same release.
