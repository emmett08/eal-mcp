# Implementation and operating boundaries

EAL/3 is implemented in Rust. The workspace uses Rust 1.90.0 and edition 2024.
The production applications share one runtime service and explicit execution
budgets. Language evaluation, external acquisition and durable coordination
have separate owners.

| Component | Implemented responsibility | Operating boundary |
| --- | --- | --- |
| `eal-parser` | Generated recognition, tokens, syntax trees and diagnostics | Bounded source, token count, nesting and syntax nodes |
| `eal-core` | Composition, validation, typed expressions, authored assessment, planning and formatting | Supplied observations, explicit context and assessment time |
| `eal-aspic` | Finite argument construction, preferences, defeats, extensions, queries and checked graph export | Explicit finite budgets with typed incomplete results |
| `eal-process` | Bounded subprocess protocol, client and independent companion | Exact argv, explicit environment and retained ownership capability |
| `eal-adapters` | Optional transport, retained artifact and specialised source/trace acquisition | Operator scope and shared collector contracts |
| `eal-runtime` | Acquisition, private storage, reuse, catalogues, packets and supervised methods | Trusted operator configuration and POSIX process/filesystem facilities |
| `eal-mcp` | Operator and registered catalogues, stdio, Streamable HTTP and the text host | Closed schemas, admission limits and configured authentication |
| `eal-coordination` | Task pinning, role gateways, first responses, effect reservations, delivery and mailbox serving | Trusted workers and one authoritative local ledger |
| `eal-cli` | Production commands, development drivers and the visualiser adapter | Workspace file boundaries and explicitly selected outputs |
| `eal-collectors` | Kubernetes, GitHub and Tempo observations | Fixed operator selectors, finite inventory, deadlines and byte allowances |
| `eal-devtools` | Native capture and installed acceptance commands | Explicitly selected executables, fixtures and output destinations |
| `eal-context-experiment` | Context-history, rehearsal, comparison and allocation calculations | Separate experiment crate and explicit synthetic or measured scope |

## Language and authored assessment

Composition handles qualified modules, trusted imports, inherited context and
closed pattern expansion. Static checks cover reference kinds, method
contracts, typed propositions, validity intervals, dependency cycles and
aggregate resource budgets. JSON and expression traversal have hard ceilings.
The [runtime guide](runtime.md) and command help describe configurable limits.

Values preserve arbitrary integers, binary64 arithmetic and rational unit
conversions at their respective boundaries. Missing observations and expression
errors have explicit representations. Typed bindings check acquisition
correspondence, canonical identities and unit conversions.

The default registry contains eight reasoning methods. The engineering profile
adds API-load criteria, reachability, RMS and sampled negative evidence.
The ASPIC profile adds finite argumentation. Custom native callbacks carry
host-declared implementation identities. Runtime method workers add process
deadlines and resource enforcement.

Authored assessment checks observation age, assumptions, support, objections,
defences and scoped transfer. Planning follows the complete declared evidence
closure. Formatting validates composed meaning and idempotence. Changed source
bytes produce a new source identity and require matching observation bindings.

## Finite formal reasoning

The [ASPIC implementation](aspic.md) supports ordinary and axiom premises,
strict and defeasible rules, three defeat kinds, ranked and partial
preferences, grounded/preferred/stable extensions and credulous/sceptical
queries. EAL compilation retains source mappings, observation provenance and
the independently evaluated authored result.

Graph export recomputes formal results and validates the `aspic-view/3`
schema. Exhausted construction or extension budgets yield an incomplete result.
The `eal-aspic-solve` process provides a bounded JSON interface for a separately
configured visualiser. Consumer and browser behaviour need their own checks.

## Acquisition and evidence identity

The runtime registry binds names and versions to command collectors or
JSON-file observations. Source input supplies evidence parameters through a
strict JSON request. Operator configuration selects executable arguments,
environment inheritance, optional file pins, deadlines and output bounds.
Collector results preserve request identity, scoped context and measurement time.

The [collector guide](collectors.md) describes read-only Kubernetes and GitHub
commands and per-cluster Tempo TraceQL searches. An identified inventory bounds
the production cluster universe. Resource presence, deployment health, revision
identity and trace queryability require their declared individual predicates.
An empty bounded trace search retains unresolved absence.

Private SQLite records preserve immutable IDs and original observation ages.
Compatible reuse requires matching request, scope, binding and execution
environment identities. Acquisition failures remain distinct from measured
negative values. Registered assessment recollects missing or expired evidence.

Supervisors bound combined process output and retain acquisition leases until
owned process-group cleanup completes. Trusted code can still exercise its
host privileges. Escaped sessions and external effects require independent
ownership controls. File hashes identify selected dependencies within their
declared coverage.

## Applications, transport and coordination

The [application layer](applications.md) provides immutable source revisions,
selected claims, registered assessment and revision-bound history. Bounded
packets preserve statuses, qualifications, omissions and retrieval identities.
Raw observations remain in operator-accessible durable records.

The MCP service exposes eleven operator tools or four registered-source tools.
Registered callers select configured claims under fixed entry context.
Host, Origin and bearer policy precede HTTP dispatch. Cancellation retains
blocking-operation ownership until the admitted work completes. The SDK codec
preserves literal JSON keys and number identities.

[Coordination](coordination.md) binds tasks to source, context and method
identities with finite budgets. Its ledger reserves effects before execution,
retains exact first-response bytes and reconciles delivery without repeating an
uncertain external effect. Worker receipts preserve their asserted provenance.
Unknown assessment outcomes retain explicit unresolved fields.

## Verification and external validation

Workspace checks are formatting, warnings-denied Clippy, unit/integration tests
and release builds. Native capture records selected checks and backend identity.
Installed smoke commands exercise application transport, reuse, coordination
recovery and supplied visualiser contracts. Tests use bounded local fixtures.

The experiment crate checks context-history machinery and prospective study
calculations separately from production verification. Synthetic rehearsals and
assumed-nuisance simulations retain those labels. Live production-cluster
coverage, Tempo availability, provider receipts and model-quality effects
require identified measurements from their actual operating environments.

The `eal-runtime` development command uses the same bootstrap as `eal` and MCP.
Bundled method workers receive an empty environment on every entry point.
Unrelated ambient variables therefore neither reach pure algorithms nor alter
their execution identity. The development command retains its bounded,
workspace-relative limits input and its existing command syntax.
