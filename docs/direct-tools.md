# Direct tool manifests

`tools.toml` binds each EAL tool name and version to one concrete operation. It
requires no capability catalogue, registration directory, qualification record,
ranked candidate list or semantic execution-plan file. Tool discovery checks the
selected MCP interface; it does not establish source completeness or the validity
of an inference.

The same `ToolRegistry` map implements this operator-owned manifest for the CLI,
MCP service and embedded runtime. `[context]` supplies the package's default
JSON-compatible environment context. An explicitly supplied execution context
replaces that default.

```toml
[context]
site = "laboratory"

[tools.snapshot]
kind = "mcp_stdio"
version = "1"
argv = ["python3", "tools/source.py"]
inherit_env = ["PATH"]
timeout_seconds = 10
max_output_bytes = 1048576

[tools.snapshot.direct]
tool = "read_snapshot"
response_format = "structured"
value_pointer = "/value"
observed_at_pointer = "/observed_at"

[tools.calculate]
kind = "command"
version = "1"
argv = ["python3", "tools/calculate.py"]
inherit_env = ["PATH"]

[tools.saved]
kind = "json_file"
version = "1"
path = "observations/saved.json"
```

Relative commands containing a slash, retained files, schema files and dependency
pins resolve against the manifest's directory. Commands and MCP servers run with
that directory as their working directory, independently of the run store.
Relative paths cannot escape the package through `..` or symbolic links. Absolute
operator-selected command and schema paths remain explicit deployment choices.
Bare executable names use the operator-selected `PATH` where supplied.

Commands inherit no process environment by default. Supply `inherit_env` names
and, when needed, explicit `env` settings. Credentials belong in externally
supplied environment variables; the manifest names them rather than storing their
values. Existing manifests that relied on implicit environment inheritance must
name the required variables explicitly.

## Transport contracts

| `kind` | Configuration | Input | Output |
|---|---|---|---|
| `command` | `argv`, optional environment and `pinned_files` | Bounded existing EAL request envelope on stdin | Observation envelope on stdout |
| `json_file` | `path` | Declared request is checked against retained envelope | Original envelope; `value`, `observed_at`, `context` and `request` required |
| `mcp_stdio` | `argv`, environment, `direct.tool` | Resolved evidence `input` object as MCP tool arguments | Explicit structured content or JSON text mapping |
| `mcp_http` | `direct.url`, `direct.tool`, optional `direct.token_env` | Same MCP input contract | Same MCP output contract |
| `http_get` | `direct.url`, optional `direct.token_env` and `direct.headers_env` | Object keys become query parameters; scalar strings, booleans or numbers only | JSON response mapped by configured pointers |

HTTP URLs must use HTTPS, except loopback HTTP for local services. Embedded
credentials, query strings, fragments, redirects and proxy discovery are rejected
at this boundary. HTTP GET never substitutes an input-provided endpoint. Remote
MCP connections use the shared client's existing bearer-authentication policy.

MCP uses the official SDK client, with one deadline covering initialization,
bounded interface discovery and invocation. It calls exactly the configured
operation and never retries an uncertain invocation. Interactive or deferred
responses are rejected. Discovery and response sizes are bounded by the operation
allowance; discovery also has finite tool and page limits. HTTP GET uses the
existing origin-bound JSON adapter.

Command acquisition uses EAL's process supervisor and acquisition lease, including
descendant cleanup and host-death handling. MCP stdio retains the shared SDK
client's existing lifecycle: it owns the immediate server process and requests
its termination when the transport is dropped or its deadline expires. It does
not inherit the command supervisor's descendant or abrupt host-death guarantees,
and does not pass an acquisition lease descriptor to that server. Operators who
need those stronger MCP server lifetime guarantees must supply an independently
supervised server deployment.

## Response mappings and schemas

`direct.value_pointer` addresses the normalised observation value, with `""`
selecting the whole decoded result. For MCP, `response_format = "structured"`
requires `structuredContent`; `response_format = "json_text"` selects the text
block at `text_block` (default zero) and decodes strict JSON. Duplicate JSON keys,
missing pointers, malformed clocks, oversized results and tool errors do not
become usable negative observations. A successful result containing `false`
remains a usable measured negative.

`observed_at_pointer` preserves the original timestamp from the response. Omit it
only when collection time is the intended measurement time. Retained-file import
always requires the original clock and never uses its reread time.

MCP's discovered input schema is checked before invocation, and any advertised
output schema is checked on a successful response. To pin discovery identities,
supply `input_schema_sha256` and optionally `output_schema_sha256`; when input is
pinned and output is omitted, the discovered tool must have no output schema.
These digests use EAL's canonical schema identity, not source-file bytes.

Local input and normalised output schemas can also be supplied as byte-pinned
JSON Schema files:

```toml
[tools.snapshot.direct.input_schema]
path = "schemas/snapshot-input.json"
sha256 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

[tools.snapshot.direct.output_schema]
path = "schemas/snapshot-value.json"
sha256 = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
```

The example digests are placeholders and must be replaced with actual SHA-256
file hashes. Schema configuration is checked before collection starts, checked
again at invocation and result admission, and cannot resolve external network or
filesystem references. These checks establish interface correspondence. Coverage,
completeness, calibration and review independence remain explicit argument
obligations.

## Executable checks

`crates/eal-runtime/tests/direct_bindings.rs` exercises real SDK stdio and
streamable-HTTP calls against synthetic local fixtures, read-only HTTP acquisition,
negative results, original clocks, exact large integers, malformed input schemas,
changed schema pins, output limits, duplicate JSON, package-relative files and
explicit environment selection. These tests verify adapter contracts; they do not
validate an external production data source.
