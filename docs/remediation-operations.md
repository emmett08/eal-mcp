# Operation contract ownership

`crates/eal-mcp/src/operations/requests.rs` owns the MCP and text-host request contract.
Its `operations!` declaration generates the typed request records, exhaustive operation enum, names, field inventory, exposure, acquisition classification and tool schemas.
`packet_budget!` applies the same rule to the nested packet fields.
`Operations::dispatch` matches the typed request enum exhaustively and retains explicit handlers.
Adding an operation requires its declaration and its handler.
A missing handler fails compilation.

The text host and `Server::call` both use `OperationRequest::decode`.
The server resolves exposure and validates registered selection before acquiring an operation permit or acquisition lease.
Settings and direct server construction share `Exposure::resolve`.
The host also retains its 2 MiB wire limit and strict duplicate-key parsing.
Direct typed callers retain the existing JSON depth, node and byte ceilings.
The general MCP client still validates the remote server's discovered schema before sending a request.

The checked-in `tools.json`, hand-maintained operation field table, host-specific field checks and independent strict-integer dispatcher have been removed.
The public field table is now a generated compatibility view of the declaration.
Schemas are constructed directly from that declaration at runtime, so there is no generated schema file to keep in sync.

Optional nullable fields preserve null as the omitted/default value.
Optional numeric fields, packet budgets, semantics and query mode reject explicit null, as their published MCP schemas require.
Assessment IDs, collection IDs, selected claim IDs and evidence IDs are admitted as nonempty identifiers.
Optional metadata-search filters retain their existing empty-string behaviour.
Attack edges require exactly two string endpoints in both schema discovery and admission.
Integer admission examines exact JSON number spellings before bounded conversion and rejects decimal or exponent forms without conversion through binary64.
Range/default constants are shared with schema construction and handlers.

Context objects are copied directly into typed records.
They are never deserialised through `serde_json::Value` again.
This preserves arbitrary-precision numbers and literal keys such as `$serde_json::private::Number`.
Invalid input diagnostics use application-owned codes and field names rather than echoing supplied values.
Unknown or unexposed tools retain the existing `Unknown tool` diagnostic category.

The independently authored corpus in `tests/operation_contracts.rs` covers every operation.
It checks valid shapes and missing, unknown, wrong-type, null, empty and oversized inputs through the text adapter and shared typed decoder.
It checks strict integers, numeric limits, attack-pair shapes and literal context values.
Its server test exercises exposure and malformed requests through `Server::call` and verifies that rejected requests leave retained state unchanged.
Existing byte-stream and HTTP transport tests cover SDK routing, strict integer spellings, cancellation, permit retention, restricted exposure and small-stack rejection.

Verification commands for this boundary are

```sh
cargo test -p eal-mcp --locked
cargo clippy -p eal-mcp --all-targets --locked -- -D warnings
```

Whole-workspace checks and installed application smoke tests remain release requirements.
