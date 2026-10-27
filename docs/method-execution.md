# Execute methods within explicit limits

[Documentation](index.md) · [Language](language-v4.md) · [Runtime](runtime.md)

An evidence record gives an argument something to reason from. A reasoning method
then performs a particular calculation under a declared contract. Keeping those
steps separate lets an engineer distinguish a failed acquisition, a completed
negative calculation and a calculation that exhausted its allowance. Each leads
to a different next action.

EAL executes the eight built-in methods in its pure core. Optional engineering,
ASPIC and custom methods can execute in an independently supervised process. The
core checks input and output contracts in both cases. Process supervision adds
wall-clock, memory and lifecycle controls for installed executables. A trusted
callback invoked directly in the core has no such process enforcement merely
because its descriptor declares timeout or memory metadata.

## Bound a native calculation

The host's limits TOML can set `method_work` independently of EAL/4 language
semantics:

```sh
printf '%s\n' '[limits]' 'method_work = 1' > limits.toml
eal-host run case.eal --tools tools.toml --workspace run --claim claim \
  --limits limits.toml
```

This deliberately small example shows the control, rather than a useful allowance
for a real investigation. The default is 1,000,000,000 units per native invocation.
It admits every valid input within the seven standard computational methods'
existing fixed size bounds.

A unit represents one scalar operation, collection visit or lookup in a
conservative structural model. Elementary library and arbitrary-precision
numeric operations each count as one operation, so the model excludes their bit
complexity. The charge is an admission bound, not measured CPU instructions,
elapsed time or a guarantee about performance on another machine. JSON bytes,
nodes, depth and method schemas have their own limits.

Admission computes the charge with saturating arithmetic after JSON and schema
checks. Complete-query validation uses the same allowance, so validation cannot
run a calculation that assessment would refuse for insufficient work. The limits
participate in the compiled programme identity and remain in the host request.

When the charge exceeds the allowance, the method returns `status: incomplete`,
empty `details`, and required and available units in `resource`. Its result
predicate remains unestablished. When an admitted calculation completes and its
result fails the predicate, that negative result is retained. Increasing a work
allowance can resolve exhaustion. It cannot turn the latter result into support.

Let `J` count JSON values, including containers, and let `L = 64(J + 1)`. The
linear term covers input checks, bounded scans, result construction and fixed
scalar bookkeeping. The formulas are implemented in
[`methods/work.rs`](../crates/eal-core/src/methods/work.rs).

| Method | Admission charge | Algorithm covered |
| --- | --- | --- |
| `deductive/1` | `L + 2^a (J + a)(a + 1)` | Up to `2^a` valuations, each visiting the formula tree and constructing or looking up an assignment with up to `a` atoms. Valid inputs have at most 12 atoms and 512 formula nodes. |
| `inductive/1` | `L + 256` | Fixed AS241 polynomial approximation and Wilson interval calculation, with no loop over the reported trial count. |
| `abductive/1` | `L + 16h²` | Ordered-map construction, lookups and posterior scans over `h` hypotheses, including worst-case tree visits. |
| `causal/1` | `L` | A fixed number of mean and variance scans over the acquired groups. Rational arithmetic and square root are elementary operations in this model. |
| `counterfactual/1` | `L + 8(v + 1)²(e + 1)` | Up to `v` readiness rounds and `e` dependency checks per round, with up to `v` linear membership comparisons. The linear term covers two affine solves. |
| `analogical/1` | `L + 16f²` | Ordered-map lookups, missing-feature ordering and comparisons over `f` selected features, plus complete input traversal. |
| `temporal/1` | `L` | A fixed number of scans over already ordered sampled events. |

Here `h` is the candidate count, `v` the structural-variable count, `e` the number
of declared parent links and `f` the selected-feature count. Deductive charging
caps its shift at 12 even for malformed input. That defensive cap does not admit
extra atoms, which still fail the method's ordinary validation.

`structured/1` performs no native calculation. It still requires actual scoped
observations and obeys acquisition and assessment limits. Optional engineering
methods and installed extensions retain their own implementation bounds and
supervisor contracts.

## Select an installed method worker

The operator constructs `WorkerCommand` with an absolute executable path, an
explicit argument vector, the complete environment and an absolute working
directory. `MethodExecutor::supervise_registry` installs bundled engineering and
ASPIC callbacks in a detached registry. Already bound host executors remain
unchanged. An unknown unbound method needs `register_command` and an explicit
command.

Source and observation data cannot choose an executable, environment, import or
shell expression. The runtime passes the admitted argument vector directly.
An operator can explicitly select a command interpreter as trusted code, with
all the host permissions that choice entails. Supervision provides resource and
lifecycle control. Filesystem and network isolation need their own host controls.

Bundled pure workers receive an empty environment consistently across CLI, MCP
and coordinator gateways. Custom worker environments remain explicit operator
configuration. The chosen environment participates in executor identity, so a
change produces a different fingerprint. Existing assessments retain the identity
under which they ran.

## Implement the worker protocol

One process handles one request. Stdin and stdout carry single-line UTF-8 JSON
objects, each terminated by a newline. Admission rejects duplicate decoded keys,
non-finite numbers, extra fields, additional response frames and trailing output.
Arbitrary integers and literal JSON keys retain their normal semantics. Stderr
shares the supervisor's bounded output allowance and is not a response channel.

Before reading stdin, the worker emits and flushes this exact readiness line:

```json
{"protocol":"eal-method-worker/1","ready":true}
```

The supervisor allows up to five seconds for readiness. It then starts the
contract's execution deadline and delivers one request with exactly these fields:

```json
{"protocol":"eal-method-worker/1","method":"engineering/rms/1","contract":{},"payload":{"origin":0,"samples":[3,4]},"limits":{}}
```

The empty `contract` above is a placeholder for the complete registered method
descriptor. `limits` contains the captured `ExecutionLimits`, with omitted fields
using core defaults. The bundled worker requires the requested descriptor to
match its local descriptor exactly. Protocol requests are capped at 20 MiB,
one million JSON nodes and depth 128. The payload also obeys the method's input
schema and input byte cap.

A successful calculation returns exactly `ok` and `output`:

```json
{"ok":true,"output":{"rms":3.5355339059327373,"sample_size":2}}
```

A failure returns exactly `ok`, `incomplete` and `error`:

```json
{"ok":false,"incomplete":true,"error":"Construction budget exceeded"}
```

The error is at most 512 UTF-8 bytes. `incomplete=true` means unfinished bounded
computation. A completed negative query uses a successful typed output instead.
The parent checks the output schema and the complete canonical success envelope
against `max_output_bytes`. Readiness and the final newline have a separate fixed
transport allowance.

## Preserve process ownership and identity

Before readiness, the bundled worker applies POSIX address-space and cumulative
CPU limits through safe `rustix` APIs and disables core dumps. Native CPU allowance
rounds up current process CPU time plus the declared execution timeout. For a
custom executable, the worker applies limits before replacing itself with the
registered argv. That allowance includes the five-second startup window, while
startup and execution retain separate wall-clock deadlines.

The independent supervisor bounds combined output and owns process-group
termination and reaping after timeout, overflow, cancellation or parent death.
Timeouts and explicit budget exhaustion return an incomplete result without
partial details. Malformed replies, schema violations and excessive output
return an invalid method failure. CPU- or memory-related termination signals
indicate unfinished execution without claiming which resource caused it.

The registry keeps the method's declared identity and adds an executor identity.
The executor fingerprint binds protocol and runtime source, command configuration,
and SHA-256 snapshots of the worker, supervisor and custom executable. Files are
hashed again before invocation, and changed bytes reject execution. These checks
identify selected file snapshots. They do not attest atomically to the machine
code loaded by the operating system. Executable and directory ownership remain
operator responsibilities.

The [development checks](development.md) exercise both completed and exhausted
calculations. [`method_work.rs`](../crates/eal-core/tests/method_work.rs) checks
every standard computational method at its default allowance, below its required
charge and at the exact charge. A deductive countermodel separately verifies that
a completed negative remains distinct from exhaustion. The resulting assessment
qualifications continue into [packets and model context](packets.md).
