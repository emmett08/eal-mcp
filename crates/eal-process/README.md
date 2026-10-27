# Bounded process execution

`eal-process` owns the established supervisor protocol, bounded client and independent process-tree cleanup companion.
It depends on neither the EAL language nor runtime storage.

`Supervisor` accepts exact argv, an explicitly supplied environment, cwd, input bytes, a timeout and a combined output byte budget.
`Retention` transfers only a duplicated ownership handle to the companion.
Runtime acquisition leases implement that capability while acquisition and lock policy remain in `eal-runtime`.
The last inherited handle remains alive until supervised descendants have stopped.

Install `eal-process-supervisor` with the application.
The implementation fails when the companion is missing or incompatible and never falls back to unsupervised execution.
Handwritten code forbids unsafe Rust.

Discovery selects only the canonical companion name from the installed package.

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

## Duplex protocol sessions

Protocol version 3 forwards a bounded bidirectional byte stream for MCP and
other interactive protocols. The same companion owns the child process group,
Linux subreaper, signal handling and inherited acquisition descriptor. It checks
an independent session deadline and cumulative input, stdout and stderr bounds.
The protocol client owns one socket. Closing it, cancelling the session or losing
the host process makes the companion stop descendants before releasing the lease.
The SDK supplies MCP protocol interpretation; the supervisor interprets no JSON-RPC.

`Supervisor::duplex` returns the raw stream and an explicit RAII process owner.
The owner requests cleanup without killing the companion prematurely. If cleanup
exceeds the short notification allowance, the companion and its lease remain alive
and a reaper retains exit ownership. Unsupported or absent companions fail without
launching an unsupervised server.
