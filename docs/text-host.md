# Text host and HTTP policy

`eal-host` reads one UTF-8 JSON object from standard input and sends one operation through the official Rust MCP SDK. The request is limited to 2 MiB. Duplicate object keys, nonfinite numbers, unknown operations and fields, and incorrect field types are rejected before connection. The host discovers the server's actual tool schema and validates the arguments before invoking that tool.

```sh
printf '%s\n' '{"operation":"grounded","arguments":["a","b"],"attacks":[["a","b"]]}' |
  eal-host --workspace ./workspace
```

The default transport launches the installed sibling `eal-mcp` executable with explicit resolved settings. It removes every `EAL_MCP_*` variable from the child environment, so the child does not reload a configuration file or apply those variables again. Install `eal-mcp`, `eal-host`, `eal-command-supervisor`, and `eal-method-worker` together. No Python executable is involved.

Registered clients can use operation `get_known_packet` with `entry_id`,
`assessment_id` and `claim` to recover a stored packet. Operations
`assess_known`, `get_known_packet` and `packet` also accept `packet_limits`
with bounded integer `max_bytes` and `max_statement_bytes` fields. These are
presentation limits, validated before connecting. See [packet retrieval](packets.md#registered-retrieval-and-presentation-budgets).

`--url http://127.0.0.1:8000/mcp` connects to an existing Streamable HTTP server. Configuring `--transport http` derives the endpoint from the configured host, port, and path; a wildcard bind address requires an explicit `--url`. HTTP mode creates no local reasoning service or database. The host negotiates MCP `2025-11-25` and does not replay tool requests after an expired session or request interactive follow-up calls.

`--timeout` defaults to 120 seconds. One deadline covers connection, initialization, discovery, schema validation, invocation, and session close. Timeout cancels the SDK connection and allows bounded cleanup. A configured timeout must be positive, finite, and representable by the platform clock. Exit status is 0 for success, 1 for a tool error, and 2 for invalid input, configuration, connection, deadline, or redaction failure. Each outcome produces one JSON response; `--help` prints usage instead.

Server settings use CLI arguments before `EAL_MCP_*` environment variables, then TOML, then defaults. Paths from TOML are relative to that file; other relative paths use the current directory. Lists replace the lower-precedence list. The native `--methods` profiles are `builtin`, `engineering`, `aspic`, and `all`; Python registry-factory imports are replaced by these explicit native selections.

HTTP server configuration and credential checks run before runtime service construction. Nonloopback bind addresses, allowed hosts, or allowed origins require `--token-env NAME`. That variable must contain a nonempty printable ASCII bearer token without whitespace. Settings retain its name; the verifier retains its SHA-256 digest and compares digests in constant time. Stdio configuration does not read an unused HTTP token.

HTTP admission retains the source framework's default hosts `127.0.0.1`, `localhost`, and `::1`, plus the configured bind host and explicit allowed hosts. It accepts explicit allowed Origins, the request's own HTTP origin, and loopback Origins for loopback Host headers. Foreign Host, foreign Origin, and missing or invalid bearer credentials are rejected before SDK operations. Duplicate admission headers are rejected.

HTTP clients require a token environment variable beyond loopback. URL user information and nonempty queries or fragments are forbidden. As in the source, an explicit plain HTTP URL can carry bearer authentication; use HTTPS when deployment traffic needs transport encryption. Ordinary clients retain the HTTP library's proxy configuration; embedders can provide a configured `reqwest::Client`.

Before printing an HTTP result or error, the host sanitizes remote JSON keys, values, and repeatedly encoded JSON strings. Fixed host and recognized MCP content wrapper fields remain stable. Unsafe encoded JSON is replaced, and key collisions cause an error instead of discarding a value. Native sanitization is bounded to 128 nested levels, 100,000 JSON nodes, and 16 MiB; a failure prints a sanitized error. Public metadata may contain the same characters as an opaque token, as in the source wrapper contract.

The patched SDK bounds JSON responses and HTTP error bodies at 16 MiB, alongside the client's SSE event bound. Its typed MCP content blocks retain recognized fields; unknown content-block extensions can be omitted during SDK decoding. Structured result JSON retains arbitrary keys and exact numbers.

`tools/capture-host-reference.py` regenerates the retained pure-host fixtures using the pinned source revision and framework versions. Normal tests consume those fixtures and exercise the built native host binary against an actual SDK HTTP server.
