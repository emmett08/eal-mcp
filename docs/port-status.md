# Rust port status

The development reference is
[`earl@e2d6f346a37f92483b9f9a13923b98feb053447b`](https://github.com/emmett08/earl/tree/e2d6f346a37f92483b9f9a13923b98feb053447b),
package `3.2.4`, language `EAL/3`. The source repository remains unchanged.
The first PR established the workspace, frozen contracts and syntax front end.
The authored semantic core and development CLI are implemented, along with finite
ASPIC reasoning, checked source compilation and graph export.
Durable runtime services, trusted acquisition, observation reuse and supervised
method execution are implemented. The application stage adds the production
`eal`, `eal-mcp` and `eal-host` entry points, catalogue revisions, registered
assessment, packet/context projections and discovery. Coordination remains the
next implementation stage.

## Scope and component boundaries

The [source inventory](../contracts/source-lock.json) accounts for all 77
handwritten Python modules installed under `src/eal`, totalling 13,647 lines.
Experiments, papers and research reporting scripts are excluded. Relevant
regression tests and executable examples can become Rust tests and examples as
their corresponding implementation is ported.

| Planned component | Responsibility | Stage |
| --- | --- | --- |
| `eal-parser` | Generated recognisers and syntax diagnostics | Implemented |
| `eal-core` | Owned syntax, composition, validation, expressions, methods, authored assessment, planning and formatting | Implemented |
| `eal-aspic` | Finite ASPIC construction, preferences, defeat, extensions, compilation and checked view export | Implemented and differentially checked; live visualiser bridge pending |
| `eal-runtime` | Collection, immutable records, reuse, catalogue, registered assessment, packets, context, method execution and supervision | Implemented within the documented POSIX runtime profile |
| `eal-mcp` | Exclusive operator/registered catalogues, authentication, transports and text host | Implemented with the official SDK and a documented local codec patch |
| `eal-coordination` | Durable task coordination, delivery and effect identities, codec and worker adapters | Pending |
| `eal-cli` | Production and development command-line interfaces | `eal`, `eal-syntax`, `eal-core` and `eal-runtime` implemented; text host is in `eal-mcp` |

The inventory retains the pinned source paths, hashes and planned component
boundaries. Its `implemented_native` labels identify the implemented and tested
native operations with the deltas documented here, rather than complete semantic
equivalence. Python import factories are replaced by explicit native registries
and a versioned subprocess method protocol. Coordination modules retain pending
labels. The table above records component progress.

Pure assessment uses typed values, supplied observations, explicit context and
an explicit timezone-aware instant. Source composition accepts a host-supplied
resolver; the CLI restricts files and trusted imports to its selected workspace.
Storage, acquisition, process execution and transports remain outside the core.
The authored EAL evaluator and opt-in ASPIC compiler retain separate
semantics.

## Implemented authored semantic core

Composition handles modules, inherited context, qualified references, trusted
imports and closed pattern expansion, including guarded recursive lists and
definition-relative template imports. Static passes check reference kinds and
scopes, method contracts, propositions, validity intervals, dependency cycles,
generated declarations and aggregate resource budgets.

Expressions retain arbitrary integers, binary64 arithmetic, Python-compatible
negative modulo, missing values and error propagation. The registry provides
all eight built-in methods and four opt-in engineering methods: API load
criteria, reachability, RMS and sampled negative evidence. Typed proposition
bindings check exact query correspondence, canonical identities and rational
unit conversions. Native implementations report their own backend identities;
custom callback artefact hashes are host declarations whose loaded-code
correspondence must be established by the host. Contract timeout metadata does
not provide process supervision or deadline enforcement in this core.

Authored assessment includes observation correspondence and age checks,
assumption validity, support, scoped objections and defences, conditional scope
transfer and grounded graph labelling. The standalone Dung solver is bounded
independently. Evidence planning follows the complete support, attack and
defence closure and emits declared calls without acquiring observations.

Semantic formatting validates and replays captured imports, then checks
source-independent composed meaning and idempotence. It preserves user payload
keys such as `span`. Direct IR without retained authored syntax rejects imports,
templates and generated declarations when formatting would require recovering
their source provenance. Formatting changes source identity and requires newly
bound observations.

The development `eal-core` commands are `compile SOURCE`, `validate SOURCE`,
`format SOURCE`, `plan SOURCE --claim NAME`, `evaluate SOURCE --now ISO_TIME
[--context JSON|@FILE] [--records FILE]`, `grounded GRAPH` and `describe`.
`--workspace DIR` selects the trusted file boundary; `--limits FILE` reads a
TOML `[limits]` table. `--methods builtin|engineering` selects the eight-method
default or adds the four optional engineering methods. Production application
entry points are described in [applications](applications.md) and
[the text-host policy](text-host.md); `eal-coordinate` remains pending.

JSON traversal defaults to depth 64 and has a hard ceiling of 128. Expressions
and schema instance checks also have hard depth ceilings of 128. The Rust
source-import and block walkers bound native recursion at 128 independently of
the host's iterative application-expansion budget. The Rust
`structural_values` budget counts serializer scalar, compound and object-key
visits through a bounded streaming pass. This is a representation-specific
budget, not Python's dataclass reflection count. Malformed direct expression
trees receive checked errors, including empty `sum()`; the source implementation
can raise an uncaught exception for that invalid direct IR.

Normal tests consume retained references without Python. They include the
101-case parser corpus and separate expression, method, static-validation,
canonical-identity and planning corpora. The 93-case cross-component corpus
covers composed source through assessment, including import failures. Its
[comparison notes](../fixtures/semantics/README.md) identify native method
identity projections and the limited binary64 tolerances. These checks support
the implemented operations on their retained cases, rather than complete port
acceptance. Differential development uses CPython 3.12.14 with Unicode 15.
Expression `sum` retains that reference's 64-bit C-long specialisation profile
across Rust targets, including those whose native C `long` has a different size.

## Implemented finite formal reasoning

The [finite ASPIC module](aspic.md) implements ordinary/axiom premises,
strict/defeasible rules, all three defeat kinds, minimum-rank and numerical or
partial last-link preferences, grounded/preferred/stable extensions and
credulous/sceptical conclusion queries. Cyclic construction admits finite
premise-founded derivations without repeated branch conclusions.

Observation-bound EAL compilation retains source mappings, reviewed policies,
unusable-record identities and independent authored results in private checked
snapshots. Graph export recomputes all formal fields under captured limits and
checks the retained `aspic-view/3` contract. Native callbacks now receive limits
explicitly and propagate typed incomplete results; construction and search
exhaustion provide no partial acceptance result. Process supervision remains
outside this pure module.

Normal tests compare exact pinned Python solver results, source compilation,
typed-method assessment and view exports. Independent subset definitions check
all directed graphs through three nodes, including self-attacks. These checks
support this finite profile rather than unrestricted ASPIC+ semantics or the
external validity of authored strict rules. Explicit inputs additionally enforce
native JSON traversal budgets before the source validator's fixed schema pass.

## Implemented durable runtime

The [runtime](runtime.md) provides immutable private SQLite records, indexed
observation lookup, descriptor-owned acquisition leases, strict operator TOML
registries, command/file acquisition, keyed binding and exact process-environment
identity, ordered bounded collection and explicit observation rebinding. Original
measurement times survive import and reuse. The frozen observation-record/1 HMAC
domain remains `EAL/2`. A separate supervisor owns process-group cleanup and retains
its inherited lease until cleanup, including on caller death. Deadlines cover
configuration and input transfer; combined output is bounded.

The shared service connects planning, collection, exact collection identity,
authored assessment, supervised ASPIC snapshots and durable explanation. The
production `eal` CLI and development `eal-runtime` driver use this service.
The original eight methods remain pure. Optional and custom
runtime methods use schema-checked workers with startup/execution deadlines and
safe POSIX resource limits. The pure ASPIC library also retains its direct,
resource-count-bounded solver API.

Native process tests exercise blocked IO, output limits, cancellation, child
status, caller death and inherited leases. Store tests cover atomic batches,
concurrent initialisation, backfill, strict JSON and replacement/link/privacy
tamper. Retained Python acquisition references cover registry validation, strict
JSON/envelopes and keyed identities. Runtime service tests exercise durable
collection, age, rebinding refusal and supervised formal assessment.

This is a POSIX private-store/process profile. The source's Windows resource-limit
refusal is retained; private Windows ACL validation remains unimplemented and the
native store fails closed there. Process groups do not contain deliberately
escaped sessions. The native store additionally rejects replacement/hardlink
identity changes and bounds stored JSON at 64 MiB/depth 128. Its aggregate store
budget is separate from per-observation semantic limits and the 32 MiB collection
limit. These defensive and transport differences are documented in the runtime
and method protocol descriptions.

## Implemented application layer

The [production CLI and catalogue](applications.md) retain immutable validated
revisions keyed by exact source/import-bundle digest and method-registry
fingerprint, fixed entry metadata, selected claims and indexed history. Registered
assessment computes the full dependency closure, reuses only acquisition-compatible
stored observations and preserves their original age. Rebinding batches are
committed atomically. Packet and model-context adapters retain bounded statuses,
qualifications, omissions and navigation links while excluding raw observations,
process streams and full source; see [their projection contract](packets.md).
Language discovery reports the pinned guide with current native identities.

The official Rust `rmcp` SDK serves stdio and stateless Streamable HTTP. Its
eleven-tool operator catalogue and three-tool registered-source catalogue are
mutually exclusive. Registered exposure validates the configured entry allowlist
at startup and accepts neither arbitrary source nor caller-selected context.
Closed input schemas, strict integer admission, bounded concurrency and a
workspace acquisition lease precede operation effects. Once blocking work starts,
it retains its permit and lease through completion after request cancellation.
HTTP Host, Origin and bearer checks run before SDK dispatch. The one-request
`eal-host` client discovers the live schema, applies a single deadline and
sanitizes remote results. See [host and HTTP policy](text-host.md).

The SDK uses an explicit [local codec patch](../vendor/rmcp/README.md) to preserve
literal JSON object keys and arbitrary-precision values with the workspace's
`serde_json` features. It is a modified official SDK, with its archive, checksum,
license and exact patch application retained. Normal builds use no Python.

Retained references include 23 catalogue scenarios with 68 operations, packet
and model-context projections, language discovery, MCP tool descriptors and
admission failures, and pure-host behavior. Native tests additionally exercise
source/import/method revisions, original observation ages, registered assessment,
real stdio and HTTP sessions, authentication, cancellation and redaction. Tests
normalize only documented backend identities, clock fields and parser-specific
diagnostics. They establish the exercised cases, rather than unrestricted
application equivalence or final package acceptance.

Three application differences are explicit. Internal refresh preserves a trusted
path-generated entry ID such as `nested/source.eal`, fixing the source's erroneous
explicit-ID revalidation; newly supplied developer IDs retain the source regex.
Integral floats such as `1.0` remain rejected for strict integer tool fields,
using a native diagnostic instead of Pydantic's diagnostic. HTTP paths are literal
routes: braces are rejected rather than interpreted as framework route syntax.
Production `grounded` and `export-aspic` avoid service and database construction;
other production commands retain the shared-service path.

## Remaining implementation sequence

1. **Port coordination.** Implement `eal-coordinate` and the production
   coordination modules, replacing Python-specific process bridges. Preserve
   delivery and effect identity, durable reservations, worker/codec contracts
   and refusal to retry calls with an uncertain execution outcome.
2. **Verify visualisation and packaging.** Connect the native solver to the
   visualiser's live bridge and exercise complete browser consumption alongside
   the retained schema and graph checks. Complete installed-package verification
   with Python absent, including coordination, MCP transports and native
   companions, while retaining process-death and reuse regressions.

The selected ANTLR generator is pinned with its runtime and recognisers are
checked in. A normal build uses those Rust sources. Java is needed only for
regeneration. Retained Python fixtures support native tests without importing
Python.

## Visualiser boundary

The consumer is pinned to
[`aspic_visualisation@a38bff4a193634556ef81e64f9b7823a1b1de6ec`](https://github.com/emmett08/aspic_visualisation/tree/a38bff4a193634556ef81e64f9b7823a1b1de6ec).
Its `schema/aspic-view.schema.json` matches the producer's
`docs/aspic-view.schema.json` byte for byte. Static JSON imports can retain this
contract through the checked Rust exporter.

The live editor's `server/solver-plugin.ts` currently spawns Python to run
`server/solve.py`, which imports the EAL solver. A separate visualiser PR must
select a native Rust executable that accepts the current theory request and
emits checked view JSON. File import compatibility and live solving require
separate end-to-end checks. This port changes neither source repository nor
visualiser.

## Completion criteria

Parser tests establish recognition and the tested lexical representation.
The authored semantic core adds differential assessments, independent Dung
checks, numerical and identity fixtures, dependency planning and semantic
formatter checks. The finite ASPIC stage adds complete extension-family checks,
snapshot comparisons and recomputed graph exports. Complete port acceptance
also requires the coordination implementation, visualiser validation and a
Python-free installed-package acceptance test across all production entry points.
Application tests retain real stdio/HTTP round trips, process ownership and
deadline tests, observation reuse and packet/context/catalogue checks. Pending
components remain explicit until their own checks exercise them.
