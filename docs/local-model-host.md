# Run the pinned adapter locally

`eal-model-host` loads the pinned Qwen model and the existing EAL/4 tool-use
LoRA weights, then lets the model propose calls to the native MCP server.
Rust reads the language-independent `.safetensors` files directly and runs
inference, tokenisation and MCP execution without a Python runtime or Python
adapter code.
The host checks each proposal against the discovered schema and authoritative
source, context, assessment time and returned identifiers before dispatch.
Those identity checks apply to arguments present in the proposal. An omitted
optional argument retains the native default, including the native assessment
time when `now` is omitted. Requests that require an explicit time therefore
need inspection of the retained arguments and native result.

An MCP client connects to tools. The process running the language model loads
its adapter weights. Connecting an existing client to `eal-mcp` therefore leaves
that client's model configuration under the client's control.
`eal-model-host` supplies a local model process with an explicit, checked bundle.

| Component | Responsibility |
| --- | --- |
| `eal-model-host` | Load the model and adapter, generate JSON tool proposals and retain the run trace |
| `eal-mcp` | Expose EAL operations and enforce the native execution contracts |
| EAL parser and runtime | Parse supplied EAL source, obtain observations and assess declared support |

Candle and the tokenizer inference dependencies belong to the separate
`eal-model-host` crate. Building `eal-mcp` by itself preserves the server's
existing dependency boundary.
Normal builds use the existing workspace default members. Select the model host
explicitly with `-p eal-model-host`; `--workspace` also includes this optional
crate.

## Build and supply the model bundle

Use Rust 1.90.0 from the repository root.

```sh
cargo build --release --locked -p eal-model-host
```

For `run`, build the native server and its companions into the same directory.
The [first-run guide](getting-started.md) explains their responsibilities.

```sh
cargo build --release --locked \
  -p eal-mcp -p eal-runtime -p eal-process \
  -p eal-tool-runtime -p eal-collectors -p eal-cli --bins
```

The MCP client also needs `eal-process-supervisor` beside `eal-model-host`.
The commands above put both binaries in `target/release`. If the client and
native server use separate installation directories, explicitly select the
supervisor before invoking the client:

```sh
export EAL_COMMAND_SUPERVISOR="$PWD/target/release/eal-process-supervisor"
```

The bundle has fixed identities.

| Asset | Accepted identity |
| --- | --- |
| Base model | `Qwen/Qwen2.5-0.5B-Instruct` |
| Base revision | `7ae557604adf67be50417f59c2c2f167def9a775` |
| Adapter | Original `models/eal4-tool-use` directory from the released EAL/4 Rust tool-use adapter |
| Adapter weight SHA-256 | `2630180e0ede1da7036a14a217348a505fcf2b5b8d27d8b89a39db68a5cd2bcf` |

Choose local paths for the base assets and the already extracted adapter.
The fetch helper obtains the pinned base assets. Supply the original adapter
from the earlier release as a local directory.

```sh
mkdir -p .local-model
export EAL_MODEL_BASE="$PWD/.local-model/base"
export EAL_MODEL_ADAPTER="/absolute/path/to/extracted-release/models/eal4-tool-use"

tools/fetch-model-base.sh "$EAL_MODEL_BASE"

target/release/eal-model-host verify \
  --base "$EAL_MODEL_BASE" --adapter "$EAL_MODEL_ADAPTER"
```

The fixed manifest checks every required base and adapter file, including the
configuration, tokenizer and chat-template assets. Keep the bundle files and
any symlink targets immutable throughout verification and loading.
Successful verification establishes bundle identity. It does not establish the correctness of a model
response. The accepted manifest retains the original tool-use adapter.
Experimental recovery-trained weights remain separately labelled in the earlier
release and require their own reviewed integration and validation.

## Exercise local generation

A small message file exercises loading, tokenisation, the chat template and
adapter inference locally. Both `generate` and `run` require a new output path
and preserve existing result files. Choose a fresh output filename when
repeating either command.

```sh
cat > .local-model/messages.json <<'JSON'
[
  {"role":"system","content":"Return one JSON object."},
  {"role":"user","content":"Return {\"finish\":true}."}
]
JSON

target/release/eal-model-host generate \
  --base "$EAL_MODEL_BASE" --adapter "$EAL_MODEL_ADAPTER" \
  --messages .local-model/messages.json \
  --output .local-model/generation.json
```

The `eal.local-generation/1` result includes verified bundle identities,
configuration, generated text, token counts and timing. This command exercises
inference separately from MCP execution.

## Validate an existing EAL document

The following task embeds the existing
[finite-inventory source](../examples/executable/finite-inventory/case.eal).
`jq` fills `state.source` from that file. The source and context are operator
inputs. The explicit time remains a task input, although source validation
itself performs no assessment. The allow-list deliberately exposes one
validation operation for this example.

```sh
jq --rawfile source examples/executable/finite-inventory/case.eal \
  '.state.source = $source' > .local-model/task.json <<'JSON'
{
  "schema": "eal.tool-use-task/1",
  "id": "validate-finite-inventory",
  "request": "Use eal_validate to check the supplied EAL source and return its diagnostics without acquiring observations.",
  "state": {
    "source": "",
    "context": {},
    "claim": "inventory_verified",
    "now": "2026-11-08T12:00:00Z"
  },
  "allowed_tools": ["eal_validate"]
}
JSON

target/release/eal-model-host run \
  --base "$EAL_MODEL_BASE" --adapter "$EAL_MODEL_ADAPTER" \
  --task .local-model/task.json \
  --native-executable "$PWD/target/release/eal-mcp" \
  --workspace "$PWD/.local-model/validation-run" \
  --output .local-model/run.json
```

The task uses actual source text. Model proposals can copy authoritative values
through `{"$state":"source"}` and the corresponding context, time or identifier
references. Successful native responses supply collection and assessment IDs.
The host retains those returned values for later calls and rejects changes to
task-bound arguments.

For acquisition or registered-source workflows, select the operator manifest
with `--registry FILE` and expose registered entries through repeated
`--known-entry ENTRY_ID` options. The [MCP hosting guide](applications.md)
describes their scope. Source validation in this example needs no collector
manifest.

The `eal.local-model-run/1` output retains verified bundle identities,
configuration, initialisation, catalogue and the nested `run` trace.
The outer status describes trace capture. Runtime, model and close errors retain
a failed result and return a nonzero exit status. A recorded policy run can
reach its step limit or retain a transport error, so inspect `run.stop`, admission
errors and each native response under
`run.calls[].response.structuredContent`. A `finished` policy run records
the model’s finish decision. Determine operation completion from the actual
native calls and responses. For assessment requests, interpret the native
`valid`, `complete` and claim-status fields to decide what was established.
A completed call can return an unsupported claim or a validation diagnostic.
Uncertain transport outcomes are retained without automatic replay.

## Select an observational catalogue scope

The default catalogue follows native discovery and any operator `allowed_tools`.
`--evidence-guided` adds an opt-in scope for observational workflows. It hides
tools whose required retained identifier is absent and offers `eal_reason` only
when an authoritative collection ID is known. The offered subset changes after
successful calls return identifiers.

Native reasoning also permits an omitted collection ID. Baseline mode preserves
that native operation, including no-collection reasoning. Choose baseline mode
for such tasks. Evidence guidance intentionally narrows the operator's admitted
operations. It uses host state and discovered schemas, preserves submitted
arguments and performs no automatic argument filling. The model still selects
an offered tool from the request.

The host uses the baseline tool-use prompt and the pinned repetition penalty.
System-prefix caching is enabled by default. `--no-prefix-cache` provides the
uncached configuration. `--max-steps`, `--max-new-tokens` and `--timeout-secs`
bound policy execution and native requests. Steps must be between 1 and 32.
The rendered prompt and generated
output share a 4,096-token limit, with at most 512 generated tokens. These
controls establish operational limits. Loader availability and bundle
verification provide no accuracy gain by themselves.

## Build on Apple Silicon

Install the Xcode command-line tools if needed and use the pinned Rust toolchain.
The CPU build uses the same command as above.

```sh
xcode-select --install
rustup toolchain install 1.90.0 --profile minimal
cargo build --release --locked -p eal-model-host
```

CPU float32 is the reference configuration exercised on Linux. Apple Silicon
build instructions and the optional Metal backend require validation on the
chosen Mac. The available CPU measurements provide no Metal performance result.

```sh
cargo build --release --locked -p eal-model-host --features metal

target/release/eal-model-host generate \
  --base "$EAL_MODEL_BASE" --adapter "$EAL_MODEL_ADAPTER" \
  --messages .local-model/messages.json \
  --device metal --precision f32 \
  --output .local-model/generation-metal.json
```

The `run` command accepts the same device and precision options.
The implementation uses Candle, Hugging Face Tokenizers and MiniJinja, with
native tokenisation and platform dependencies described in the
[attribution notices](../THIRD_PARTY_NOTICES.md#local-model-host).

## Scope of the model path

This integration loads previously trained LoRA assets for EAL/4 MCP JSON tool
use. The model and host runtime are Rust. EAL grammar recognition and source
validation remain in the native parser and runtime. Model output uses greedy
token selection with the pinned repetition penalty. EAL source authoring,
model training and token-level grammar-constrained generation remain separate
capabilities.

## Verify the installed model and MCP path

The default test suite needs no model download. After building release binaries
and supplying the pinned assets, run the explicitly ignored installed-path test.
It checks an actual model-selected native validation, exact source reuse, native
validity and refusal to overwrite an existing trace. This single validation task
is an integration check rather than a tool-use accuracy benchmark.

```sh
EAL_MODEL_HOST_BIN="$PWD/target/release/eal-model-host" \
EAL_MODEL_NATIVE="$PWD/target/release/eal-mcp" \
cargo test --locked -p eal-model-host --test local_model_smoke -- --ignored
```

Export `EAL_MODEL_BASE` and `EAL_MODEL_ADAPTER` as shown above before this command.
The test runs Rust binaries and uses no Python runtime.
