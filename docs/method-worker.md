# Native method workers

The runtime executes installed engineering, ASPIC and custom methods through
an independently supervised process. The eight built-in methods stay
in the pure core. Direct core native callbacks remain trusted in-process calls;
their declared timeout and memory metadata does not enforce process limits.

An operator constructs `WorkerCommand` from an absolute executable path, explicit
argument vector, complete environment map and absolute working directory.
`MethodExecutor::supervise_registry` installs bundled engineering and ASPIC
callbacks in a detached registry. Already bound host executors remain unchanged;
unknown unbound methods require `register_command` and an explicit command.
Source and observation payloads cannot select executables, environment, imports
or shell expressions. The runtime never constructs a shell command. An operator
may explicitly register a command interpreter as trusted code.

The bundled `eal-method-worker` launches custom implementations through a versioned
process protocol. Each implementation supplies an explicit executable.
The process boundary is a resource and lifecycle boundary, not a filesystem or
network sandbox for untrusted code.

## Transport

One process handles one request. Standard input and output use UTF-8 JSON objects
on single newline-terminated lines. Duplicate decoded keys, non-finite numbers,
extra object members, additional response frames and trailing output are rejected.
Arbitrary integers and literal JSON object keys retain their normal semantics.
Standard error shares the supervisor's bounded output budget and is not used as
a response channel.

The worker must emit and flush this exact canonical readiness line before reading
standard input:

```json
{"protocol":"eal-method-worker/1","ready":true}
```

The supervisor allows at most five seconds for readiness, then starts the
contract's independent execution deadline and delivers one request. The first
frame selects the protocol version for the following response. A request has
exactly these fields:

```json
{"protocol":"eal-method-worker/1","method":"engineering/rms/1","contract":{},"payload":{"origin":0,"samples":[3,4]},"limits":{}}
```

Here `contract` is the complete registered method descriptor, rather than the
illustrative empty object. `limits` is the captured `ExecutionLimits` object;
omitted budget fields use the core defaults. The bundled worker requires its
local descriptor to match the requested descriptor exactly. Protocol requests
are capped at 20 MiB, one million JSON nodes and a hard nesting depth of 128.
The payload separately obeys the contract's input schema and input byte cap.

A successful response contains exactly `ok` and `output`:

```json
{"ok":true,"output":{"rms":3.5355339059327373,"sample_size":2}}
```

A failed response contains exactly `ok`, `incomplete` and `error`:

```json
{"ok":false,"incomplete":true,"error":"Construction budget exceeded"}
```

The error string contains at most 512 UTF-8 bytes. `incomplete=true` reports an
unfinished bounded computation, never a negative query result. A completed
negative query remains a successful typed output. The parent checks the output
schema and the complete canonical success envelope against `max_output_bytes`;
readiness and the final newline have a separate fixed transport allowance.

## Resource and identity policy

Before readiness, the bundled worker applies POSIX address-space and cumulative
CPU limits with safe `rustix` APIs and disables core dumps. Native CPU allowance
is rounded up from current process CPU time plus the declared execution timeout.
For a custom executable, the bundled worker applies these limits before replacing
itself with the exact registered argument vector. Its cumulative CPU allowance
also includes the five-second startup window; startup and execution wall-clock
deadlines remain separate. Limits are inherited across that replacement.

The independent supervisor bounds combined standard output/error and owns process
group termination and reaping on timeout, output overflow, cancellation or parent
death. Timeouts and explicit worker budget exhaustion return `incomplete` with no
partial details. Malformed replies, schema violations and excessive output return
an invalid method failure. CPU/memory-related termination signals are reported as
unfinished execution without claiming that a specific resource caused them.

Registries retain the pure method's declared identity and add a distinct executor
identity. The executor fingerprint binds protocol source, worker/runtime source,
explicit command configuration and SHA-256 snapshots of worker, supervisor and
custom executable files. Those files are hashed again before each invocation;
changes reject execution. This records checked file snapshots, not an atomic
attestation of the machine code loaded by the operating system. Executable and
directory ownership remain the operator's responsibility. No function-pointer
process-specific address enters a fingerprint.
