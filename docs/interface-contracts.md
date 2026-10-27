# Keep identities and admission at their owning boundaries

[Documentation](index.md) · [Implementation](implementation.md)

A claim can pass through source compilation, acquisition, assessment, transport
and presentation. Each step changes its representation while retaining the
identities and qualifications needed by the next step. The engineering question
at an interface is therefore precise: which component is entitled to admit the
record, and which facts may a consumer infer from its acceptance?

The language core owns pure meaning. The collector contract owns request and
observation-envelope invariants. Optional adapters translate operator-selected
sources, while the runtime owns correspondence, persistence, reuse and leases.
The process companion owns bounded execution. The MCP layer owns transport
admission and exposure. Keeping these owners distinct lets a new adapter reuse
existing evidence rules without reimplementing the language.

## Define a request once

[`operations/requests.rs`](../crates/eal-mcp/src/operations/requests.rs) declares
the MCP and text-host request contract. Its `operations!` declaration generates
typed records, the exhaustive operation enum, names, field inventory, exposure,
acquisition classification and schemas. `packet_budget!` provides the equivalent
ownership for nested packet options. `Operations::dispatch` matches the request
enum exhaustively. Adding an operation requires its declaration and handler,
with a missing handler rejected by compilation.

The text host and `Server::call` both use `OperationRequest::decode`. Exposure and
registered selection are checked before operation permits or acquisition leases
are taken. Settings and direct construction share `Exposure::resolve`. The text
host also enforces its 2 MiB wire limit and strict duplicate-key parsing. Direct
typed callers retain their JSON depth, node and byte ceilings. The general MCP
client checks the remote server's discovered schema before sending a request.

Schemas are constructed from the declaration at runtime, and the public field
table is a generated view of it. This keeps discovery, local admission and handler
selection dependent on one definition.

Nullable optional fields preserve null as an omitted/default value. Optional
numeric fields, packet budgets, semantics and query mode reject explicit null
where their schema requires a value. Assessment, collection, selected-claim and
evidence identifiers must be nonempty. Optional metadata-search filters keep
their allowed empty-string meaning. Attack edges contain exactly two string
endpoints in both discovery and admission.

Integer admission examines the original JSON number spelling before bounded
conversion. Decimal and exponent forms are rejected without passing through
binary64. Range and default constants are shared by schemas and handlers. Context
objects are copied directly into typed records, preserving arbitrary integers
and literal keys such as `$serde_json::private::Number`.

Diagnostics use application-owned codes and field names instead of echoing
supplied values. Unknown and unexposed tools retain the `Unknown tool` category.
The same principle applies to credentials: shared bounded inspection examines
decoded JSON strings, keys and scalar spellings. Duplicate ambiguity or exhausted
inspection fails admission. Collectors retain the credential set through final
envelope inspection, including generated metadata, without rewriting a
successful semantic value.

## Choose a representation that reflects the field's job

Finite internal choices use typed variants. Extensible names and opaque record
keys remain strings with admission at their owning boundary. A digest records
correspondence to particular bytes. It does not establish that those bytes are
sufficient evidence for a claim.

| Field family | Owner and representation | Required check |
| --- | --- | --- |
| Source declarations and references | Core AST and composed programme retain authored strings, with typed declaration and support kinds. | Identifier grammar, uniqueness, reference kinds, dependencies and imported namespaces are validated before evaluation. |
| Language and wire versions | Versioned strings and package-owned constants, with native `ContractVersion` variants. | Loaders admit supported versions. Explicit revision produces a current package. |
| Collector, tool and environment names | Checked `CollectorId`, `ToolId` and `EnvironmentName` values. | Binding admission checks identifiers, environment names and adapter-specific selectors. |
| Tool versions and method identifiers | Contract-selected text and immutable `MethodContract` records. | Tool versions match source declarations. Methods validate identity, schemas and implementation metadata before use. |
| Binding, provider and scope choices | `ToolKind`, `ProviderConfig`, `Provider`, `ResourceScope` and `ContractVersion`. | Direct binding and `Config::new` checks run before acquisition. |
| Source, context, input and data fingerprints | Canonical SHA-256 identities, with keyed private binding and environment identities. | Acquisition, evaluation and reuse compare the identities relevant to the operation. |
| Registered entries and selected claims | Bounded catalogue keys and declared claim names. | Registration, exposure, selection and exact revision/context checks restrict retrieval. |
| Collection, assessment and observation IDs | Opaque `RunStore` keys and expected record kinds. | Store lookup, collection identity and registered retrieval bind source, context, claim and method revision. |
| Operations, exposure and transport | Typed requests, `Operation`, `Exposure`, `ClientTarget` and SDK protocol versions. | Shape, nullability, exact numbers, budgets, URL and authentication checks precede effects. |
| Acquisition state and failure category | Collection `ok`/`error` and typed execution failures. | Only successful corresponding observations are reusable. Supervisor framing validates status, streams and return codes. |
| Method state | Core `MethodResultStatus`. | Evaluation constructs states, and packet/context consumers reject unknown or contradictory states. |
| Claim, assumption, argument and objection state | Core authored-assessment vocabularies. | Each projection admits the vocabulary for that result family. Successful collection and supported claim retain different meanings. |
| Completeness and availability | Assessment `complete`, packet `summary_complete`, availability issues and context outcomes. | Incomplete calculations and omitted presentation details remain separately qualified. |
| Labels, prose, queries and rationale | Extensible text owned by source, store or method. | Expected record kinds and method contracts constrain use. Text cannot independently select an executable or authorise a transition. |
| Diagnostics | Typed source spans and application-owned codes/categories. | Errors preserve category and location while excluding credentials and unsafe source excerpts. |
| Coordination messages and effects | MAS codec records and constrained ledger transitions. | Admission, uniqueness and state checks prevent uncertain effects from being silently reopened. |

## Preserve qualifications during projection

A successful acquisition can carry a measured negative. An absent or stale
observation can leave the same predicate unresolved. A method can also stop
before producing a result. The [packet contract](packets.md) carries those
separate states into smaller model contexts.

Complete assessments use packet/2. An incomplete assessment uses packet/3 with
both global and per-claim completeness qualifications. The small navigation
fallback retains these fields even when it drops method details. Consumers check
version and qualifier consistency before using a packet.

Availability omissions also matter. If a packet reports that it omitted any
availability issues, its unavailable entries remain unresolved because the
consumer cannot recover which qualification was lost. Available evidence and
complete predicate failures retain their respective met and not-met outcomes.
This conservative projection avoids turning a missing measurement into a
negative one.

## Verify the boundary that changed

The independent operation corpus covers every operation through the text adapter
and shared decoder. It checks valid shapes and missing, unknown, wrong-type,
null, empty and oversized fields, strict numbers, attack pairs and literal
context values. Server cases verify that malformed or unexposed requests leave
retained state unchanged. Transport tests add byte-stream framing, HTTP routing,
cancellation and permit retention.

```sh
cargo test -p eal-mcp --locked
cargo clippy -p eal-mcp --all-targets --locked -- -D warnings
```

These focused checks supplement the workspace and installed application checks in
[development](development.md). They establish admission behaviour on the tested
cases. Evidence about external systems still comes from the configured sources.
