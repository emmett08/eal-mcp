# Published schemas and licences

The runtime validates current interfaces through typed Rust contracts and closed
MCP operation schemas. The language guide is maintained with the runtime under
`crates/eal-runtime/src/discovery/`.

| File | Purpose |
| --- | --- |
| `schemas/aspic-view.schema.json` | JSON schema for checked ASPIC graph exports consumed by the visualiser |
| `schemas/coordination-worker-response.schema.json` | JSON schema for coordination worker answers |
| `schemas/collector-catalogue.schema.json` | Versioned collector descriptions for authoring clients |
| `schemas/acquisition-request.schema.json` | Collector stdin request fields and identity |
| `schemas/observation-envelope.schema.json` | Collector stdout observation envelope |
| `collectors/native.json` | Current optional Kubernetes, GitHub and Tempo authoring catalogue |
| `licenses/` | Required licence texts for attributed materials |

Method input, query and output schemas are maintained beside their implementations
in `crates/eal-core/src/methods/` and `crates/eal-aspic/src/`. MCP input schemas are
maintained in `crates/eal-mcp/src/operations/tools.json`.

The `EAL/2` observation HMAC domain identifies the observation-record algorithm.
The authored source language is EAL/4. Record identities distinguish integers,
floating-point values, negative zero, Unicode and exact source/context bytes.

See [implementation boundaries](../docs/implementation.md) and
[runtime configuration](../docs/runtime.md).

Collector wire models live in the pure `eal-collector-contracts` crate. The
native executable publishes the checked catalogue through `eal-collector describe`.
See [collector authoring contracts](../docs/collector-contracts.md) for versioning,
operator authority and consumer requirements.
