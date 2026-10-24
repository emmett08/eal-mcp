# EngArg and EAL remediation

The [cross-repository ledger](https://github.com/emmett08/engarg/blob/main/docs/REMEDIATION.md) maps all 21 review findings to their implementation owners and regression sources.
The paired pull requests record combined acceptance and exact tested commit identities.
The original programme started at `7bd9948ed242b1173b8331a5c4cee4a158a42722`.
The boundary follow-up starts at the merged programme revision `b48a48d`.
These base identities provide no acceptance evidence for later changes.

## Boundary follow-up

Adversarial checks after the original programme found the following gaps.
The scope below identifies the implemented corrections and their regression sources.
Release acceptance remains bound to the tested revisions recorded on the paired pull requests.

| Boundary | Correction and owner | Regression source |
| --- | --- | --- |
| Known credentials inside encoded JSON | The pure contracts package owns one bounded immutable inspector. Host and Tempo consumers inspect decoded strings, keys and scalar spellings. Duplicate-key ambiguity and exhausted inspection budgets fail safely | Contracts `sensitive_json` tests, MCP `tests/host.rs`, native `tests/collectors.rs` and paired EngArg host tests |
| Credentials in generated collector metadata | Tempo retains one captured credential set through final observation-envelope inspection. Successful semantic values are never replaced | Native CLI tests include generated metadata, query fields and request/context collisions |
| Omitted evidence qualifications | Packet projection rejects unknown issues. Positive packet-wide availability omissions keep unavailable evidence unresolved | Runtime `tests/packets.rs` and `tests/model_context.rs` |
| Reusable retained-artifact bases | The adapter canonicalises a relative-artifact base once and preserves independent absolute grants. Traversal, escape and leaf-alias checks remain active | Adapter `tests/admission.rs` |
| Native configuration before effects | Typed admission rejects empty scoped attribute components and non-UTF-8 operator paths. Executable conversion preserves the exact admitted path | Collector `tests/configuration.rs` and installed CLI cases in `tests/collectors.rs` |

Sensitive JSON inspection was extracted into the narrow contract package because three consumers require the same policy.
Tempo credential lifetime is a private record separate from its semantic value.
Configuration and artifact-base checks remain with their existing admission owners.
The follow-up changes no public observation or packet schema and preserves the manual workflow trigger.

EAL's pure collector contract owns request, envelope, identifier and direct binding invariants.
The narrow `eal-process` package owns bounded process execution.
Optional adapters own reusable acquisition and trace mechanics.
The runtime retains evidence correspondence, persistence, reuse and lease policy.
The language core retains its existing pure boundary.
MCP request admission and schemas share [one operation declaration](remediation-operations.md).

The [collector contract](collector-contracts.md), [native collector guide](collectors.md) and [runtime guide](runtime.md) describe the current boundaries and compatibility windows.
The runtime's installed supervisor alias remains during migration to EAL 0.2.
Explicit native version 1 descriptors remain supported until collectors 1.0.
Historical observations preserve their recorded age.
The paired EngArg v1/v2 readers preserve source and identity while current execution safeguards remain mandatory.

## EAL field ownership inventory

Wire strings remain strings where they name an extensible declaration, version, source or opaque stored record.
A finite internal enum is used where variants select behaviour.
This inventory covers the runtime and its transport boundary in addition to EngArg's separate domain inventory.

| Field family | Owner and representation | Admission or correspondence check |
| --- | --- | --- |
| Source declaration names and references | Core AST records and composed `semantic::Program` retain authored strings. `SupportKind`, `TargetKind` and declaration variants are typed | Core validation checks identifier grammar, duplicates, referenced declaration kinds, dependencies and imported namespaces before evaluation |
| Language, catalogue, request, export and packet schema versions | Versioned wire strings and package-owned constants. Native `ContractVersion` is an enum | The applicable loader dispatches supported versions. Current model-context admission rejects unknown packet versions. Historical export interpretation belongs to the paired frozen EngArg readers |
| Collector, executable-tool and environment-variable names | Contract `CollectorId`, `ToolId` and `EnvironmentName` provide checked names. Request wire fields retain their string encoding | Catalogue/profile and binding admission enforce the applicable identifier and environment-name rules. Native adapters add their declared selector limits |
| Tool versions and reasoning-method IDs | Tool versions remain contract-selected text. `MethodRegistry` owns versioned method identifiers and immutable `MethodContract` records | Tool binding version must match the source. Method registration validates versioned identity, schemas and implementation metadata. Unknown methods fail static admission |
| Binding kind, provider, scope and native version | `ToolKind`, `ProviderConfig`, `Provider`, `ResourceScope` and `ContractVersion` select finite variants | Direct binding invariants and `Config::new` validate combinations before acquisition. CLI conversion delegates to those same boundaries |
| Source, context, input, request, data and registry fingerprints | Canonical SHA-256 strings identify their particular serialised content. Private binding and environment identities use keyed digests | Runtime identity, acquisition, evidence evaluation and reuse compare the relevant fields. Packet projection validates digest shape. A digest establishes declared correspondence rather than evidential sufficiency |
| Catalogue `entry_id` and selected claim IDs | Catalogue entries are opaque bounded keys. Explicit entry names and default relative-source paths have deliberately different admission | Registration validates ownership and selected claims. MCP `Exposure::resolve`, registered selection and exact revision/context checks limit access |
| Collection, assessment, observation and run IDs | `RunStore` owns opaque record keys and expected record kinds. Generated IDs use UUIDs. Public wire representations remain text | Store lookups bind SQL parameters and expected kinds. `CollectionIdentityValidator` checks source/context. Registered retrieval checks source, claim, context and method revision. Packet IDs have bounded encodings |
| MCP operations, exposure and transport | `Operation`, typed request records, `Exposure`, `ClientTarget` and the SDK `ProtocolVersion` own finite dispatch choices | One decoder checks shape, nullability, exact integers and budgets. Exposure and registered selection precede permits and acquisition. URL/authentication checks precede connection or service construction |
| Observation acquisition state and failure categories | The collection owner emits `ok` or `error`. `AcquisitionFailureCategory`, `ExecutionError` and `ProcessError` own execution failures | Only `ok` observations enter successful reuse. Other or malformed states cannot establish available evidence. Supervisor framing checks typed status, stream lengths and return-code bounds |
| Method-result state | Core `MethodResultStatus` owns supported, unsupported, not-run and incomplete variants | Evaluation constructs these states. Packet projection and model-context admission use this same enum and reject unknown states |
| Claim, assumption, argument, environment and objection results | Core evaluation owns the authored-assessment state vocabulary. Wire results retain compatible strings | Packet/model-context projection admits the state set for each result family. A supported claim, available observation and successful acquisition retain their separate meanings |
| Completeness and availability qualifications | Evaluation `complete`, packet `summary_complete`, availability issues and model-context requirement outcomes describe distinct properties | Incomplete packet/3 requires global `complete=false` and a false `assessment_complete` qualifier on each selected claim. Projection omissions independently determine `summary_complete`. Unknown issue/state values fail bounded projection |
| Store record `kind`, evidence `kind`, labels, queries, rationales and prose | Deliberately extensible text owned by the store, language or selected method | Expected-kind lookups and method/evidence contracts constrain use. These fields do not select an unconstrained executable or authorise a transition |
| Diagnostic code, message and source location | Core `Diagnostic` retains compatible code strings and typed spans. Registry/operation boundaries emit application-owned codes. Process and acquisition failures have typed categories | Boundary handlers preserve category and location while excluding credential-bearing registry excerpts and unsafe semantic responses. Human-readable messages are presentation text |
| Coordination messages and effect state | The pinned MAS codec owns admitted message records. The effect ledger owns `EffectStatus` and constrained stored transitions | Codec admission checks message identity/shape. Ledger uniqueness and database state constraints prevent reserved or unknown effects from being silently reopened |

The inventory identified method-state drift between evaluation and packet projection.
A bounded method could return `incomplete` while the packet path accepted only completed results.
The shared method-state enum and incomplete-only packet/3 path address that mismatch.
Complete assessments retain packet/2 output shape.
Packet/3 preserves the incomplete qualifier even when the byte budget selects the navigation fallback.
The required per-claim qualifier also causes the reviewed older model-context consumer's closed claim-field admission to refuse these new packets, including fallback packets whose argument detail is absent.
Current consumers check packet version and qualifier consistency explicitly.
The regression starts with actual bounded native-method exhaustion and checks the assessment, packet, model context, malformed states and fallback.
This review found no additional finite-state input that currently bypasses its admission owner.
That judgement is scoped to the inspected boundaries and does not assert that every remaining string field requires an enum.

The follow-up packet audit found that omitted availability qualifications could
turn a stale negative observation into a definite unmet predicate in model context.
Packet projection now rejects unknown or non-string availability issues.
Model context retains `unresolved` for unavailable evidence whenever the
packet-wide availability-issue omission count is positive.
Available evidence and complete pure predicate failures retain `met` and
`not_met` respectively. This changes no packet fields or versions.
Regression cases exercise the public packet-builder-to-model-context path and
the conservative treatment of other unavailable entries in the same packet.

Release acceptance requires Rust 1.90 formatting, workspace Clippy with warnings denied, workspace tests and debug/release binary builds.
Installed application and coordination smoke checks use the actual companions.
The application checks exercise all 15 operations through both text and MCP entry points.
The shared registry corpus also runs through actual runtime admission and the paired EngArg exporter.
EAL workflow execution remains manual.
