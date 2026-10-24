# EAL-CONTEXT-001 native diagnostics

Version 0.1.0 defines three executable machinery checks in the isolated Rust
experiment package. The commands implement planning calculations, scripted history
policies and native packet comparison. Their outputs identify the implementation,
selected inputs and measurement scope. Model-session performance remains unmeasured.

## History policies

The history representation retains whole tool exchanges. Every packet block
contains a complete ordered set of function calls and corresponding results.
Assessment references contain the entry, claim, assessment, source, context,
method-registry and original assessment-time identities. A reference cannot change
those fields while retaining its key.

| Condition | Packet retention | Explicit restoration |
|---|---|---|
| A | Keep packet exchanges until the common request cap requires whole-turn removal. | No restoration cache. |
| B | After four completed turns, replace the exchange with its reference metadata. | Replay the first exposed exchange from the host cache. |
| C | After four completed turns, replace the exchange with its reference metadata. | Request a stored assessment at a selected presentation budget. |

Condition C admits the byte/statement budget ladder `4096/1024`, `16384/1024` and
`65536/4096`. Each request consumes its level even when retrieval fails. The ladder
resets at the next user turn. Restoration accepts only exact previously exposed
references. Conditions A and B expose the default `16384/1024` budget.

The common request cap covers protected instructions, tools, reference index and
current messages. Capacity pressure removes complete completed turns. The current
turn and protected material remain present. A current request that exceeds the
cap after all eligible removals produces an explicit context-overflow failure.

`eal-context rehearse` executes 40-, 80- and 120-turn traces under all three
conditions. Packets and answers are synthetic fixtures. The request cap is 32,768
serialised UTF-8 JSON bytes. The report records packet bytes, request occupancy,
lookups, eviction events and source hashes. Byte counts describe the native JSON
serialisation and establish no provider-token measurement.

## Usage and deadlines

The usage adaptor preserves raw provider-adapted readings and arbitrary
nonnegative integer precision. Cached input is a subset of total input. Reasoning
is added once when the supplied output count excludes it. Missing required
readings leave the total null. The implementation supplies no provider endpoint.

A turn budget preserves its original monotonic deadline and admits at most six
model/tool cycles by default. Nonfinite or reversed clock readings are rejected.
Deadline-restricted latency assigns the deadline to incomplete observations and
caps completed observations at that deadline. Tests exercise these arithmetic and
failure-accounting rules with synthetic readings.

## Native packet comparison

`eal-context probe` accepts two distinct clean checkouts with explicit 40-digit
revision identities. It checks those identities and records every executable's
SHA-256 digest. Optional `--build` compiles the CLI, MCP server and runtime
companions into each selected checkout's target directory.

The diagnostic uses simple, byte-pressure and argument-count-pressure EAL
fixtures. Both runtimes receive identical sources, observation values, context
and assessment clocks. The default packet comparison normalises generated
assessment, collection and observation IDs while retaining their repeated
identity relations. Every other packet field remains in the comparison.

Candidate retrieval uses actual `eal-host` to MCP stdio to `eal-mcp` requests.
The report measures the raw packet JSON carried by MCP. It checks presentation
budgets, omissions, original assessment identity and time, record-count stability,
registered entry admission and private-observation exclusion. A source edit and
new assessment exercise retrieval of the preceding stored revision.

Each native child receives an empty executable search path. The process runner
bounds a request to 1 MiB, combined stdout/stderr to 8 MiB and execution to
240 seconds. The trusted local run owns its child process groups. Temporary
private workspaces are removed after execution. A report is produced only after
all selected diagnostic assertions pass.

## Statistical planning model

`eal-context planning allocation` evaluates 38 parameter cells.
`eal-context planning sensitivity` evaluates five further cells. Every cell
defaults to 6,000 simulated studies. The reports record the seed, xoshiro256**
generator, scenario parameters, component tests and Monte Carlo uncertainty.

Each scenario supplies a paired treatment/baseline token cost and three paired
quality differences. Baseline and treatment costs use the declared lognormal
planning family. Quality differences use bounded uniform margins generated from
the selected latent correlation structure. The sample unit is an independent
scenario mean. The simulation assumes complete telemetry and accurate scores.

The token contrast is treatment total minus 0.90 times baseline total. The
quality contrasts compare each paired mean with a degradation boundary of
0.02. One-sided paired Student tests use significance level 0.05. Joint success
requires all four components to pass. The token estimand is the ratio of
arithmetic mean costs. Log-transformed cost ratios supply a different estimand.

Reports retain joint success, component pass rates, two-sided coverage and Wilson
Monte Carlo intervals. Boundary and harmful-effect cells challenge the decision
rule. Analytic allocation and repeat-allocation calculations expose the assumed
quality variance and repetition cost. The numerical profile records binary64
arithmetic, the normal approximation and Student quantile algorithm.

The 10% saving threshold and two-percentage-point quality margins are explicit
planning parameters. The calculations describe their selected distributions.
Empirical token savings, real workload variance, model answer quality and
retention remain unknown. Executed findings and reproduction commands are in
[results](results.md) and the [README](README.md).
