# Text host and HTTP policy

`eal-host` reads one UTF-8 JSON object from standard input and sends one operation through the official Rust MCP SDK. The request is limited to 2 MiB. Duplicate object keys, nonfinite numbers, unknown operations and fields, and incorrect field types are rejected before connection. The host discovers the server's actual tool schema and validates the arguments before invoking that tool.

```sh
printf '%s\n' '{"operation":"grounded","arguments":["a","b"],"attacks":[["a","b"]]}' |
  eal-host --workspace ./workspace
```

The text adapter and MCP server share one typed operation decoder.
The operation declaration generates both tool schemas and required/optional field inventories.
Explicit null is accepted only by fields advertised as nullable.
Integer budgets reject decimal and exponent spellings before numerical conversion.
Registered source selection and request admission precede acquisition leases.
See [operation contract ownership](remediation-operations.md).

The default transport launches the installed sibling `eal-mcp` executable with explicit resolved settings. It removes every `EAL_MCP_*` variable from the child environment, so the child does not reload a configuration file or apply those variables again. Install `eal-mcp`, `eal-host`, `eal-process-supervisor`, and `eal-method-worker` together. Add `eal-tool-run` for bindings to portable verified tool packages.

Registered clients can use operation `get_known_packet` with `entry_id`,
`assessment_id` and `claim` to recover a stored packet. Operations
`assess_known`, `get_known_packet` and `packet` also accept `packet_limits`
with bounded integer `max_bytes` and `max_statement_bytes` fields. These are
presentation limits, validated before connecting. See [packet retrieval](packets.md#registered-retrieval-and-presentation-budgets).

`--url http://127.0.0.1:8000/mcp` connects to an existing Streamable HTTP server. Configuring `--transport http` derives the endpoint from the configured host, port, and path; a wildcard bind address requires an explicit `--url`. HTTP mode creates no local reasoning service or database. The host negotiates MCP `2025-11-25` and does not replay tool requests after an expired session or request interactive follow-up calls.

`--timeout` defaults to 120 seconds. One deadline covers connection, initialization, discovery, schema validation, invocation, and session close. Timeout cancels the SDK connection and allows bounded cleanup. A configured timeout must be positive, finite, and representable by the platform clock. Exit status is 0 for success, 1 for a tool error, and 2 for invalid input, configuration, connection, deadline, or credential-inspection failure. Each outcome produces one JSON response; `--help` prints usage instead.

Server settings use CLI arguments before `EAL_MCP_*` environment variables, then TOML, then defaults. Paths from TOML are relative to that file; other relative paths use the current directory. Lists replace the lower-precedence list. The `--methods` profiles are `builtin`, `engineering`, `aspic`, and `all`.

HTTP server configuration and credential checks run before runtime service construction. Nonloopback bind addresses, allowed hosts, or allowed origins require `--token-env NAME`. That variable must contain a nonempty printable ASCII bearer token without whitespace. Settings retain its name; the verifier retains its SHA-256 digest and compares digests in constant time. Stdio configuration does not read an unused HTTP token.

HTTP admission accepts the default hosts `127.0.0.1`, `localhost`, and `::1`, plus the configured bind host and explicit allowed hosts. It accepts explicit allowed Origins, the request's own HTTP origin, and loopback Origins for loopback Host headers. Foreign Host, foreign Origin, and missing or invalid bearer credentials are rejected before SDK operations. Duplicate admission headers are rejected.

HTTP clients require a token environment variable beyond loopback. URL user information and nonempty queries or fragments are forbidden. An explicit plain HTTP URL can carry bearer authentication; use HTTPS when deployment traffic needs transport encryption. Ordinary clients retain the HTTP library's proxy configuration; embedders can provide a configured `reqwest::Client`.

Before printing an authenticated HTTP response, the host inspects decoded JSON
keys and values and repeatedly encoded JSON strings for the configured bearer
credential. A detected credential rejects the entire semantic response with exit
status 2. Assessment states, evidence, identifiers and protocol fields retain their
exact values on success. Numeric and Boolean values are inspected in their
canonical textual form, so short credential collisions fail safely. Inspection
is bounded to 128 nested levels, 100,000 JSON nodes and 16 MiB. Strings that decode
as JSON undergo the same recursive inspection. Other prose
retains its exact text, including prose beginning with JSON punctuation.
Exhausted inspection allowances fail safely. These checks protect known values
within admitted encodings rather than arbitrary transformed encodings.

A separate local diagnostic envelope may redact its explicitly display-only
`error` text. Its fixed `is_error` and `error` keys remain public protocol literals.
Remote structured tool errors remain semantic results and undergo the same
immutable-result check as successful results. Recognised MCP content blocks
retain their SDK field shape.

The patched SDK bounds JSON responses and HTTP error bodies at 16 MiB, alongside the client's SSE event bound. Its typed MCP content blocks retain recognized fields; unknown content-block extensions can be omitted during SDK decoding. Structured result JSON retains arbitrary keys and exact numbers.

Native tests exercise the host binary through actual SDK stdio and HTTP sessions, including request validation, authentication and immutable semantic output.

Malformed server TOML reports `EAL_SERVER_CONFIGURATION_TOML` with a numeric
line and byte-column location. Runtime and CLI limits TOML reports
`EAL_LIMITS_TOML` using the same safe diagnostic helper. These admission paths
exclude parser source excerpts and supplied token values before credentials or
other settings have been decoded. Registry syntax uses `EAL_REGISTRY_SYNTAX`.
