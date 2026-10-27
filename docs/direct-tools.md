# Connect evidence requests to authorised operations

After the source states what must be observed, the operator chooses how to obtain
it. `tools.toml` binds each EAL tool name and version to one concrete operation.
An existing command, MCP tool, HTTP endpoint or retained file can be used directly.
Discovery checks a selected interface. Source completeness and inference adequacy
remain obligations in the argument.

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
supplied environment variables. The manifest names them rather than storing their
values. Name every variable the selected implementation needs.

## Transport contracts

| `kind` | Configuration | Input | Output |
|---|---|---|---|
| `command` | `argv`, optional environment and `pinned_files` | Bounded EAL request envelope on stdin | Observation envelope on stdout |
| `json_file` | `path` | Declared request is checked against retained envelope | Original envelope.  `value`, `observed_at`, `context` and `request` required |
| `mcp_stdio` | `argv`, environment, `direct.tool` | Resolved evidence `input` object as MCP tool arguments | Explicit structured content or JSON text mapping |
| `mcp_http` | `direct.url`, `direct.tool`, optional `direct.token_env` | Same MCP input contract | Same MCP output contract |
| `http_get` | `direct.url`, optional `direct.token_env` and `direct.headers_env` | Object keys become query parameters. Scalar strings, booleans or numbers only | JSON response mapped by configured pointers |

HTTP URLs must use HTTPS, except loopback HTTP for local services. Embedded
credentials, query strings, fragments, redirects and proxy discovery are rejected
at this boundary. HTTP GET never substitutes an input-provided endpoint. Remote
MCP connections use the shared client's existing bearer-authentication policy.

MCP uses the official SDK client, with one deadline covering initialisation,
bounded interface discovery and invocation. It calls exactly the configured
operation and never retries an uncertain invocation. Interactive or deferred
responses are rejected. Discovery and response sizes are bounded by the operation
allowance. Discovery also has finite tool and page limits. HTTP GET uses the
existing origin-bound JSON adapter.

Command and MCP stdio acquisition use EAL's independent process supervisor and
acquisition lease. A bounded duplex stream connects the official MCP SDK to the
server. Deadline, cancellation and host death stop the owned process group before
the companion releases its inherited lease descriptor. The supervisor also bounds
the complete protocol input/output transcript and discarded stderr. Runtime MCP
sessions inherit the binding's exact lifetime limit and never replay an uncertain
invocation. Generic persistent SDK clients have a one-hour maximum session lifetime
with separate per-operation deadlines. Callers can supply a tighter explicit
supervision contract. These local process guarantees do not undo external effects
already performed by a remote tool.

## Response mappings and schemas

`direct.value_pointer` addresses the normalised observation value, with `""`
selecting the whole decoded result. For MCP, `response_format = "structured"`
requires `structuredContent`.  `response_format = "json_text"` selects the text
block at `text_block` (default zero) and decodes strict JSON. Duplicate JSON keys,
missing pointers, malformed clocks, oversized results and tool errors do not
become usable negative observations. A successful result containing `false`
remains a usable measured negative.

`observed_at_pointer` preserves the original timestamp from the response. Omit it
only when collection time is the intended measurement time. Retained-file import
always requires the original clock and never uses its reread time.

MCP's discovered input schema is checked before invocation, and any advertised
output schema is checked on a successful response. Generate reproducibility pins
from the operator-configured interfaces with:

```sh
eal-host prepare case.eal --tools tools.toml --workspace prepare-run \
  --claim inventory_verified --output-tools prepared.tools.toml
eal-host run case.eal --tools prepared.tools.toml --workspace run \
  --claim inventory_verified
```

Preparation uses the same `eal_prepare(source, context, claim?)` service available
through MCP. It admits the selected claim's complete evidence dependency closure,
resolves at most 128 distinct configured tools and has a 120-second discovery
budget. Bounded local-file and schema admission precedes discovery. It cannot
interrupt an operating-system file read mid-call. For MCP tools it performs initialisation and bounded `tools/list`
discovery, then selects the exact named operation. It issues no `tools/call` and
creates no observations or assessment. Omit `--claim` to prepare all evidence.
Non-MCP bindings are admitted and identified without invoking them.

Standard output contains an inspectable receipt with source, executable, manifest,
binding and discovered interface identities. `--output-tools` writes a new private
TOML file beside the input manifest, preserving relative resources and the original
operator configuration. Only discovered schema hashes and server identity are added.
Discovery cannot alter commands, endpoints, environment, response mappings or limits.
The receipt contains no acquired values or resolved credentials. Existing explicit
environment values remain in the operator's TOML. Inherited credential references
remain references. Execution needs only the EAL source, the generated TOML and tool
implementations, so the receipt and preparation workspace can be removed.

The generated fields are `input_schema_sha256`, optional `output_schema_sha256`
and `direct.server_identity` (`name`, `version`, `protocol`). When input is pinned
and output is omitted, the discovered tool must have no output schema. Digests
use EAL's canonical schema identity, not source-file bytes. Preparation and
execution both reject a changed server identity or schema before invocation.
Every completed MCP call retains its actual `mcp_interface` identity in the
observation, even when no pins were configured and when the tool reports an error.
The server name/version is advertised metadata, not independent attestation of a
remote binary or its source. Pins establish interface correspondence. They do not
establish a claim's truth.

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
explicit environment selection. `mcp_preparation.rs` verifies discovery without
operation calls, pre-call drift rejection and retained actual identities.
`executable_packages.rs` moves and executes generated TOML without its receipt.
These tests verify adapter contracts. They do not
validate an external production data source.


For a new command implementation, continue with [collectors](collectors.md).
For an implementation generated and verified by EngArg, use
[portable tool packages](tool-packages.md). Return to the
[documentation path](index.md).
