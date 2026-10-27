# Discover a collector contract before choosing its binding

The normal [execution path](getting-started.md) uses a direct tool manifest. An
authoring client can optionally inspect operator-supplied descriptions to find a
prospective measurement before constructing that manifest. This contract lets the
client check the source interface without embedding a provider's implementation.
`eal-collector describe` prints the current optional native catalogue and performs
no acquisition. Its parsed value matches
[`contracts/collectors/native.json`](../contracts/collectors/native.json).

```sh
eal-collector describe > native-collectors.json
```

The catalogue uses `schema = eal.collector-catalogue/1` and
`acquisition_contract = eal.acquisition/1`. The version identifies these wire
semantics independently of the producing package version. A consumer must reject
unknown versions, duplicate IDs, ambiguous JSON and unsupported constraints.
Consumers compile and validate the supplied input and result JSON Schemas before
admitting a catalogue. Typed wire decoding alone establishes their field shape.
The checked schema is
[`collector-catalogue.schema.json`](../contracts/schemas/collector-catalogue.schema.json).
The pure `eal-collector-contracts` crate owns the wire models and typed command
registry serialisation. It depends on no runtime, database, network client or
language parser.

Each descriptor gives a stable collector ID and version, purpose, planning
parameter names, input JSON Schema, result JSON Schema, acquisition predicate,
command template and input constraints. `input_schema.properties` identifies the
parameters projected into the EAL evidence input. Other planning parameters
configure fixed selectors in the proposed command. Input values preserve their
declared JSON types. An `ordered_window` constraint requires an increasing pair
of integer fields with a bounded difference. The collector independently enforces
its current contract before acquisition.

`CollectorDescriptor::required_parameters` returns unconditional top-level
required input fields and command-template parameters. Other declared input
properties can be omitted. Supplied null remains a value subject to the schema.
Conditional schema requirements are checked against the complete prepared input.
Catalogue version 3 retains earlier descriptors and adds API and delivery
contracts. `describe-api` derives a descriptor from a pinned operator source.

The predicate describes measurement availability. For example, a nonempty
cluster inventory permits interpretation of a Kubernetes observation even when
`all_present` is false. Collecting that observation leaves the argument's separate
acceptance and sufficiency requirements intact. `result_schema` describes the
observation's `value` field. Failed acquisition exits unsuccessfully. Earlier or
cached observations retain their original `observed_at` and correspondence.

Command templates use literal argv tokens. `{name}` substitutes a reviewed
planning parameter. `{operator.name}` requires a separately supplied operator
selection. A placeholder occupies one complete token. Partial interpolation,
unknown parameter names and planning-controlled executable tokens are rejected.
Bindings use explicit environment names and process allowances. No shell is
involved. A catalogue describes available tools and grants no execution authority.
The operator approves executable paths, credential names, source scope and the
resulting registry before invoking collection.

Consumers parse bounded strict JSON before deserialising the wire models. The
runtime's strict parser rejects duplicate keys and non-finite numbers, while its
envelope validation checks scoped context and acquisition identity. Importing a
retained observation additionally requires its original timestamp. The public
native `Request` facade delegates common identity checks to `AcquisitionRequest`
and retains provider-specific selector validation. Native providers require
object inputs and cap identity fields at 1,024 bytes. The general acquisition
contract permits any finite JSON input and retains the runtime's aggregate
request budget.

The current native catalogue is an optional set of infrastructure measurements. Generic EAL
`command` and `json_file` bindings also support other domains. A laboratory
collector can describe numeric readings and channel arrays through its own
input schema without adding a provider to the EAL language or rebuilding the
runtime. An authoring client selects only the catalogues approved for its task.

Changes to required fields, placeholder interpretation or constraint semantics
require a new catalogue schema version. A collector's measurement contract uses
its own descriptor version and declared EAL tool version. Compatible documentation
or package changes preserve those versions. Reformatting EAL source still changes
source identity and requires observations bound to the resulting source.

## Authoritative admission and operator classification

`Catalogue::validate` checks the published wire format and compiles local-only
schemas. `validate_for(AuthoringProfile::RuntimeExportable)` adds the named
`eal.runtime-exportable/1` profile. That profile permits up to 64 distinct inherited
environment names and one scalar comparison as the acquisition predicate. The
published catalogue format retains its 256-name prospective allowance. A valid
wire catalogue can therefore require a profile that this authoring client does
not support. Dotted and hyphenated collector identifiers remain valid.

`RegistryDocument::from_toml` is the authoritative executable registry loader.
Both runtime loading and producer-side checks call its direct invariants.
`CommandRegistry::to_toml` admits executable bindings. `PreparedCommandRegistry`
serialises prospective configuration containing explicit unresolved selections.
The runtime rejects unresolved `/REPLACE_*`, `REPLACE_*` and `{operator.*}` tokens
before execution. Resolving these requires operator choices rather than model
inference. Missing or incompatible supervisor companions remain errors.

Malformed registry diagnostics report `EAL_REGISTRY_SYNTAX` with numeric line and
byte-column locations. They exclude source excerpts, token values and parser
Display text. Other registry admission errors use stable application-owned codes.
The independent `registry-corpus.json` shipped with the contract crate exercises
accepted and rejected bindings through both shared and actual runtime admission.

Sensitivity declarations use the separate operator-owned
`eal.operator-environment/1` document. Its `forward` field lists permitted names.
Its optional `public` field explicitly identifies ordinary configuration among
those names. Omitted names remain sensitive. Duplicate names, unforwarded public
names and credential-like public names are rejected. Bearer credentials remain
sensitive independently of this document. Catalogues and model-generated plans
cannot supply these classifications. Values never belong in the declaration.

## Numeric and schema interpretation

Strict JSON admission has two explicit numeric interpretations behind one duplicate,
Unicode and resource-checking parser. `strict_json` selects the runtime's
established binary64 interpretation for decimal and exponent tokens. Integers
remain arbitrary precision. `strict_json_exact` preserves the arbitrary-precision
`serde_json` number representation for authoring, persisted export identities and
semantic host responses. That includes decimal precision, exponent spelling and
signed-zero behaviour admitted by that representation. Generic optional adapters
select the exact interpretation. Runtime acquisition retains its binary64 boundary.
Both reject non-JSON `NaN` and infinity tokens. An exponent outside binary64's
range is rejected by the binary64 interpretation and retained by the exact interpretation as a
finite symbolic JSON number. Byte, nesting and node allowances apply to both.

The catalogue schema permits omitted `constraints`, matching their established
empty default. Version and predicate controls and whitespace-only purposes are
rejected by both the schema and the owner. Free Unicode string lengths use JSON
Schema's scalar-value `maxLength` meaning. Input byte ceilings remain separate
operational budgets. ASCII identifier and environment-name limits retain their
existing meaning.

The shipped `catalogue-corpus.json` records independent wire, structural-schema
and runtime-exportable-profile outcomes. Schema-expressible cases must agree.
Cross-field collector-ID uniqueness and template-parameter membership remain
semantic checks in the shared owner. Reserved legacy IDs, the 64-name execution
allowance and scalar predicates belong to the named runtime-exportable profile.
The corpus labels those distinctions explicitly. A structural JSON Schema check
alone therefore cannot replace shared semantic/profile admission. Downstream
compatibility checks can run the same cases without reproducing those rules.

## Support-contract diagnostics

The derived `SupportContract` JSON schema fixes `schema` to
`eal.support-contract/1`. `acceptable_methods` requires 1–128 distinct protocol
identities in `name/version` form, each at most 256 ASCII characters. A method
identity names the protocol required by the obligation. Its appearance in a
model response does not qualify a provider or establish that the protocol was
performed.

`SupportContract::diagnostics()` returns independently detected defects with a
relative JSON pointer, stable code and actionable message. A wrong schema
identity and an empty method list appear together. The existing `validate()` API
joins those diagnostics into its error result. Structural admission remains a
separate check from source availability and empirical adequacy.


Acquisition dependencies and pure calculations in current executable packages are
[EAL/4 source](language-v4.md). Authoring metadata is not an executable plan for
the runtime. Pin the authoring client's EAL dependency and installed producer to
compatible revisions, then run their producer/consumer checks before deployment.

Return to [collectors](collectors.md) for implementing a measurement or the
[documentation path](index.md) for the complete workflow.
