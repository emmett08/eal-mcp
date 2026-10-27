# Production applications and registered sources

The native application layer provides `eal`, `eal-mcp` and `eal-host`. All use
the same reasoning contracts and configured runtime. The
[coordination layer](coordination.md) adds `eal-coordinate`, and the
[visualiser adapter](aspic.md#visualiser-integration) provides the explicit theory-to-view
process.

Install the applications and their companions using the
[first-run guide](getting-started.md). This chapter follows the next operational
decision: whether to run an explicit package, expose a configured host or retain a
registered claim for later reassessment. Each uses the same runtime service.

## CLI operations

Use `eal --help` for complete arguments. Common configuration is
`--workspace DIR`, `--registry TOML`, `--database SQLITE`, `--limits TOML` and
`--methods builtin|engineering|aspic|all`. Registry, database and executable
locations are trusted operator choices. These `eal` file commands require source
and context files inside the workspace. Runtime imports resolve from the workspace
root, including when the primary file is in a subdirectory. The package form of
`eal-host run` reads its explicitly selected primary source and context files
separately, as explained in [runtime ownership](runtime.md#choose-the-execution-boundary).

| Commands | Result or effect |
| --- | --- |
| `describe`, `validate`, `format`, `plan` | Language/contracts, diagnostics, checked formatting or complete evidence dependency planning |
| `collect`, `reason`, `compile-aspic` | Configured acquisition, durable authored assessment or supervised formal compilation |
| `explain`, `packet` | Stored explanation or a bounded assessment projection |
| `register`, `register-tree`, `sources`, `find` | Source registration, bounded discovery and catalogue lookup |
| `assess-known`, `history`, `model-context` | Registered claim assessment, revision-specific run metadata or checked model messages |
| `grounded`, `export-aspic` | Explicit graph labelling or recomputed checked graph export |

Production `grounded` and `export-aspic` run before service construction: they
create no database and require no runtime companions. `export-aspic` writes only
its explicit workspace output. Other production commands use the shared service
and can create `.eal/runs.sqlite3` and resolve companions even when the selected
operation performs no acquisition. Use the pure `eal-core` development CLI for
storage-free source validation, formatting, planning and supplied-record
assessment.

For an existing trusted `case.eal` declaring claim `works`:

```sh
eal --workspace ./workspace --registry ./workspace/tools.toml \
  register case.eal --entry-id demo --claim works --context '{"site":"bench"}'
eal --workspace ./workspace --registry ./workspace/tools.toml sources
eal --workspace ./workspace --registry ./workspace/tools.toml \
  assess-known demo --claim works --reuse compatible
eal --workspace ./workspace --registry ./workspace/tools.toml history demo
eal --workspace ./workspace --registry ./workspace/tools.toml \
  model-context demo --claim works --question 'What does the evidence establish?'
```

Assessment uses the current UTC time unless `--now ISO_TIME` supplies a
timezone-aware instant. A supported result is conditional on declared scope,
observations, assumptions and inference rationales. It does not prove prose
truth. Invalid validation, incomplete computation and failed observations use
exit status 1. Invalid commands or inputs use status 2. A valid unsupported or
contested claim is a completed assessment, rather than a command error.

## Catalogue and registered assessment

An entry fixes its workspace-relative `.eal` path, label, context and selected
claims. Its default ID is the normalized path. An explicit developer ID must
match `[A-Za-z_][A-Za-z_0-9.-]{0,127}`. Changing fixed metadata requires another
entry ID and an independently registered path. Context is finite JSON with an
8,192-byte canonical envelope. One to 256 claims may be selected. Tree discovery
is sorted and bounded to 512 EAL files, 8,192 scanned entries and depth 16 below
the discovery root, and reports rejected source files individually. It does not descend
through directory symlinks.

Refresh validates the current source and imports before recording a revision.
Revisions are immutable and keyed by entry ID, exact source/import-bundle digest
and method-registry fingerprint. Invalid edits leave the last valid revision
intact. Method changes can create a new revision while the source digest stays
the same. History selects exact source and context identities and returns run
metadata, rather than observation values. It does not turn a historical result
into a fresh observation.

Registered assessment plans the complete support, objection and defence closure.
The default `compatible` policy selects successful stored readings only when
their request, scoped context, current binding and command-environment identities
match. Each reading retains its original observation time and is checked for
freshness at assessment time. A comparable measured negative can be reused.
Failed acquisition cannot. Missing readings are acquired and rebinding batches
are committed atomically. `--reuse fresh` acquires the selected evidence again.
The workspace lease spans selection, acquisition, rechecks and durable commit.
The CLI and embedded API can override context for one assessment without changing
entry metadata. Catalogue history filters the entry's registered context.

Packets and model context exclude raw observations, process streams and full
source. They retain bounded statuses, checked scalar outputs, qualifications,
omissions and explanation links. Negative, stale and missing evidence remain
distinct. See [packet and context contracts](packets.md).

## MCP exposure and text host

`eal-mcp` uses the official Rust `rmcp` SDK for stdio and stateless Streamable
HTTP. The [pinned SDK and local JSON codec patch](../vendor/rmcp/README.md) retain
literal object keys and number identity with the evaluator's JSON features.
The modified SDK archive, checksum, license and exact patch application are
recorded. Normal builds do not execute its upstream build script.

The default operator exposure publishes thirteen tools for description, source
formatting/validation/planning, interface preparation, collection, execution,
reasoning, formal compilation, explanation, packets and grounded graphs. Registered exposure publishes
`eal_sources`, `eal_find_claims`, `eal_assess_known` and `eal_get_known_packet`
for a configured allowlist:

```sh
eal-mcp --workspace ./workspace --registry ./workspace/tools.toml \
  --transport stdio --exposure registered --known-entry demo
```

The catalogues are exclusive. Registered entries are checked at startup. The
registered tools accept neither arbitrary source nor caller-selected context,
time or reuse policy. Public source summaries omit registered context and source
text. Closed schemas, bounded admission and acquisition exclusion run before
effects. Cancellation after blocking work starts retains its permit and lease
until that work completes.

`eal_get_known_packet` reads an existing assessment using `entry_id`,
`assessment_id` and `claim`. It checks the exact registered revision and context
and returns a bounded projection without collecting again. Assessment and packet
tools accept optional byte and statement budgets through `packet_limits`. See
[retrieval and presentation budgets](packets.md#choose-a-presentation-budget)
for admission limits, historical semantics and model input guidance. Omitting `packet_limits` uses the configured defaults.

Strict integer fields reject integral floats such as `1.0` with the diagnostic
`Tool input requires a strict integer for FIELD`. HTTP paths are literal: braces
are rejected instead of selecting framework route parameters. Host, Origin and
bearer policy is checked before SDK dispatch. For configuration precedence,
authentication, deadlines, credential-safe semantic responses and the one-request `eal-host` client,
see [text host and HTTP policy](#text-host-and-http-policy).

Native tests exercise registered reuse, MCP transport admission, cancellation and
packet retrieval. [Coordination](coordination.md) adds durable role-scoped task execution.

Server TOML configuration has a 1 MiB file allowance. An operator-selected
configuration alias resolves once to its canonical regular-file target before
bounded reading. The `eal-syntax` source-file path uses the same regular-file
boundary with its existing 1 MiB source allowance. Its explicitly selected
source aliases retain canonical-target semantics. Standard input remains a
separate bounded stream. FIFO file arguments are rejected without waiting for
a writer. Filesystem identity checks do not establish immutable contents during
an in-place concurrent write.

## Text host and HTTP policy

The text host lets a shell or another application call the same MCP operations
without implementing an MCP session. It reads one UTF-8 JSON object from standard
input, discovers the server's actual schema and checks the arguments before
invoking the operation. Its 2 MiB request allowance bounds that admission step.
Duplicate keys, nonfinite numbers, unknown operations or fields and incorrect
field types fail before connection.

```sh
printf '%s\n' '{"operation":"grounded","arguments":["a","b"],"attacks":[["a","b"]]}' |
  eal-host --workspace ./workspace
```

The text adapter and server derive their request records, field inventories and
tool schemas from one operation declaration. A nullable field admits explicit
null. Integer budgets require integer spelling, so decimal and exponent tokens
are rejected before numerical conversion. Registered-source selection and request
admission complete before an acquisition lease is taken. See
[interface contracts](interface-contracts.md) for the owning decoder.

By default, the host launches its installed sibling `eal-mcp` with the resolved
settings. It removes `EAL_MCP_*` from the child environment so the server cannot
apply those settings a second time. The [installation guide](getting-started.md)
keeps the host, server and required runtime companions together.

For an existing Streamable HTTP server, supply `--url
http://127.0.0.1:8000/mcp`. Alternatively, `--transport http` derives the endpoint
from the configured host, port and path. A wildcard bind address requires an
explicit URL because it identifies a listener rather than a client destination.
HTTP mode creates no local reasoning service or database. The client negotiates
MCP `2025-11-25`. An expired session or interactive follow-up leaves the operation
unresolved instead of triggering an automatic replay.

### Bound the whole call

`--timeout` defaults to 120 seconds and covers connection, initialisation,
discovery, schema validation, invocation and session close. Timeout cancels the
SDK connection and allows bounded cleanup. A configured value must be positive,
finite and representable by the platform clock. Each operation produces one JSON
response. `--help` prints usage.

| Exit status | Interpretation |
| --- | --- |
| `0` | The operation succeeded. Inspect the assessment's status separately. |
| `1` | The MCP operation returned a tool error. |
| `2` | Input, configuration, connection, deadline or credential inspection failed. |

Registered clients retrieve retained results with `get_known_packet`, supplying
`entry_id`, `assessment_id` and `claim`. That operation, `assess_known` and `packet`
also accept bounded integer `packet_limits.max_bytes` and
`packet_limits.max_statement_bytes`. Admission checks these presentation limits
before connecting. [Packet retrieval](packets.md#retrieve-a-registered-result-again)
explains what can be recovered from a historical result.

### Resolve settings once

Settings use this order: CLI arguments, `EAL_MCP_*` environment variables, TOML,
then defaults. A supplied list replaces the lower-precedence list. Paths in TOML
resolve beside that file, while other relative paths use the current directory.
The `--methods` sets are `builtin`, `engineering`, `aspic` and `all`.

Server and credential admission happens before constructing the runtime service.
Malformed server TOML reports `EAL_SERVER_CONFIGURATION_TOML` with a numeric line
and byte-column location. Limits files use `EAL_LIMITS_TOML`, and the tool manifest
uses `EAL_REGISTRY_SYNTAX`. These diagnostics omit parser excerpts and supplied
token values, including failures that occur before credentials have been decoded.
Server TOML has a 1 MiB allowance.

### Make remote access an operator choice

A nonloopback bind address, allowed host or allowed origin requires
`--token-env NAME`. Its value must be a nonempty printable ASCII bearer token
without whitespace. Configuration retains the variable name. The verifier retains
its SHA-256 digest and compares digests in constant time. A stdio configuration
leaves an unused HTTP token unread.

Host admission accepts `127.0.0.1`, `localhost`, `::1`, the configured bind host and
explicitly allowed hosts. Origin admission accepts explicit allowed Origins, the
request's own HTTP origin and loopback Origins when the Host is loopback. Foreign
hosts or origins, duplicate admission headers and missing or invalid bearer
credentials are rejected before SDK dispatch.

HTTP clients also require a token environment variable beyond loopback. The URL
must omit user information, a nonempty query and fragments. An explicitly chosen
plain HTTP URL can carry bearer authentication. Select HTTPS when the deployment
requires transport encryption. Ordinary clients use the HTTP library's proxy
configuration. An embedder can supply a configured `reqwest::Client`.

### Preserve a result or reject it in full

Before printing an authenticated HTTP response, the host inspects JSON keys,
values and repeatedly encoded JSON strings for the configured credential. A match
rejects the entire semantic response with status 2. This preserves the meaning of
accepted results, including their assessment states, evidence and identifiers.
Numeric and Boolean values are inspected in canonical textual form, so a short
credential can collide with ordinary data and cause rejection.

Inspection is bounded to 128 nested levels, 100,000 JSON nodes and 16 MiB.
Strings that decode as JSON receive the same recursive inspection. Other prose
keeps its exact text. Exhausted inspection limits fail the response. The guarantee
covers the known credential within admitted encodings, with arbitrary transformed
encodings outside that check.

A local diagnostic envelope may redact its display-only `error` text while
retaining the literal `is_error` and `error` keys. A remote structured tool error
is a semantic result and receives the same immutable-result check as success.
The SDK preserves recognised content-block fields and exact structured-result
JSON. Unknown content-block extensions may be omitted during SDK decoding.

The patched SDK bounds JSON responses and HTTP error bodies at 16 MiB alongside
its SSE event bound. Native tests exercise the installed host through actual SDK
stdio and HTTP sessions, including admission, authentication and immutable output.

Continue with [packets](packets.md) to share a retained result, or return to the
[documentation path](index.md).
