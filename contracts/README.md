# Published schemas and licences

The runtime validates current interfaces through typed Rust contracts and closed
MCP operation schemas. The language guide is maintained with the runtime under
`crates/eal-runtime/src/discovery/`.

| File | Purpose |
| --- | --- |
| `schemas/aspic-view.schema.json` | JSON schema for checked ASPIC graph exports consumed by the visualiser |
| `schemas/coordination-worker-response.schema.json` | JSON schema for coordination worker answers |
| `licenses/` | Required licence texts for attributed materials |

Method input, query and output schemas are maintained beside their implementations
in `crates/eal-core/src/methods/` and `crates/eal-aspic/src/`. MCP input schemas are
maintained in `crates/eal-mcp/src/operations/tools.json`.

The `EAL/2` observation HMAC domain identifies the observation-record algorithm.
The authored source language is EAL/3. Record identities distinguish integers,
floating-point values, negative zero, Unicode and exact source/context bytes.

See [implementation boundaries](../docs/implementation.md) and
[runtime configuration](../docs/runtime.md).
