# Portable verified tools

An existing MCP, HTTP, command or retained-file operation needs only a direct
TOML binding. Construct a tool package only when an operation is missing.
EngArg authors and verifies that implementation; EAL owns its execution.

Install the shared runner beside the process companion:

```sh
cargo install --path crates/eal-process --locked
cargo install --path crates/eal-tool-runtime --locked
```

`eal-tool-run --package tool.toml` receives the bounded JSON input on stdin and
executes the pinned implementation using the toolchain recorded in the manifest.
The generated `tools.toml` binds an EAL operation to this command. Run the complete
argument through the ordinary host:

```sh
eal-host run case.eal --tools tools.toml --workspace run
```

The package contains source files, input and output schemas, a frozen toolchain
profile and verification provenance. Hashes bind the files to the package. Keep
its relative directory layout when moving it. Build instructions describe the
actual toolchain; the runtime requires no EngArg executable or authoring model.

The operator installs Docker, the exact previously verified image and matching
`eal-tool-run` / `eal-process-supervisor` binaries. Execution never pulls a new
image or falls back to a native unsandboxed process. The profile fixes build and
run arguments, resource limits, network policy and permitted environment names.
Credentials remain external values. Builds and execution use temporary working
resources under the shared bounded lifecycle.

The same EAL-owned isolation implementation serves EngArg fixture verification
and standalone execution. EngArg keeps the authoring state machine and acceptance
criteria; the shared runtime supplies process and container mechanics. Original
verification receipts identify what was checked. They neither assert that a new
host has the same binary identity nor prove arbitrary properties of the generated
program.

A changed implementation or schema requires a newly verified package or an
explicitly reviewed deployment change. A missing toolchain, unavailable image,
invalid pin, exhausted budget or failed acquisition is an execution failure;
it cannot become a successful observation. Source measurement clocks are
preserved where the contract requires them.

See [direct tools](direct-tools.md) for transport bindings and
[executable programs](executable-programs.md) for the source package workflow.
