# Collector authoring contracts

An authoring client can select prospective measurements from an operator-supplied
catalogue without knowing the provider's CLI or embedding its execution code.
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

The current native catalogue is an optional infrastructure profile. Generic EAL
`command` and `json_file` bindings also support other domains. A laboratory
collector can describe numeric readings and channel arrays through its own
input schema without adding a provider to the EAL language or rebuilding the
runtime. An authoring client selects only the catalogues approved for its task.

Changes to required fields, placeholder interpretation or constraint semantics
require a new catalogue schema version. A collector's measurement contract uses
its own descriptor version and declared EAL tool version. Compatible documentation
or package changes preserve those versions. Reformatting EAL source still changes
source identity and requires observations bound to the resulting source.
