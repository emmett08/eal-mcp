# Install EAL and run a complete argument

The quickest way to understand the execution boundary is to run a programme whose
observations and calculations you can inspect. The finite-inventory example
retrieves three synthetic strings over MCP, checks them with a command tool and
recomputes count and distinctness in EAL. A finite deductive step combines those
two supported subclaims.

The claim concerns the retrieved fixture snapshot. It gives no evidence about
an external catalogue. That small scope lets this first run test the installation
without depending on production credentials or an authoring model.

## Install the commands together

From the repository root, use the pinned Rust toolchain and install the runtime
packages into one prefix. Python 3 is needed by the example tools.

```sh
cargo build --release --workspace --locked
cargo install --path crates/eal-process --locked
cargo install --path crates/eal-runtime --locked
cargo install --path crates/eal-tool-runtime --locked
cargo install --path crates/eal-cli --locked
cargo install --path crates/eal-mcp --locked
cargo install --path crates/eal-coordination --locked
cargo install --path crates/eal-collectors --locked
```

The host resolves its companion executables beside itself. Keep the installed
set together when moving or deploying it.

| Package | Installed commands |
| --- | --- |
| `eal-process` | `eal-process-supervisor` |
| `eal-runtime` | `eal-method-worker` |
| `eal-tool-runtime` | `eal-tool-run` |
| `eal-cli` | `eal`, `eal-syntax`, `eal-core`, `eal-runtime`, `eal-aspic-solve` |
| `eal-mcp` | `eal-mcp`, `eal-host` |
| `eal-coordination` | `eal-coordinate`, `eal-mas-server`, `mas-protocol-codec` |
| `eal-collectors` | `eal-collector` |

The durable runtime requires POSIX ownership, file modes and process facilities.
Windows private-store ACL validation is unimplemented. Build and development
alternatives are in [development](development.md).

## Execute the supplied package

From the repository root:

```sh
eal-host run examples/executable/finite-inventory/case.eal \
  --tools examples/executable/finite-inventory/tools.toml \
  --workspace inventory-run --claim inventory_verified > inventory-run.json
```

`eal-host` starts its sibling `eal-mcp` and sends `eal_execute`. The server
collects the required observations, performs the pure derivations and assesses
the selected claim. The returned JSON contains `collection` and `assessment`.
Inspect the assessment's claim status as well as its validity and completeness.
Command completion alone does not assert support.

Read the [source](../examples/executable/finite-inventory/case.eal) beside the
[manifest](../examples/executable/finite-inventory/tools.toml). The source names
`snapshot_source` and `identifier_audit`. The manifest selects their actual
implementations and allows `PATH` for their Python launchers. Relative tool paths
resolve beside the manifest, so the package can move independently of its run
store.

The second supplied package demonstrates seven reasoning modes over explicitly
synthetic observations and reviews:

```sh
eal-host run examples/executable/reasoning-modes/case.eal \
  --tools examples/executable/reasoning-modes/tools.toml \
  --workspace modes-run --claim all_modes > modes-run.json
```

[Executable programmes](executable-programs.md#portable-acceptance-packages)
explains the method-specific inputs and the limits of those conclusions.

## Separate collection from assessment

Use the production `eal` CLI when you want to inspect the plan and retain an
identified collection before assessment. The commands below run from the
repository root and keep the example package inside the selected workspace.
They require `jq` to select returned identities.

```sh
example=examples/executable/finite-inventory

eal --workspace "$example" --registry "$example/tools.toml" \
  validate case.eal

eal --workspace "$example" --registry "$example/tools.toml" \
  plan case.eal --claim inventory_verified > inventory-plan.json

eal --workspace "$example" --registry "$example/tools.toml" \
  collect case.eal --claim inventory_verified --context '{}' \
  > inventory-collection.json

collection_id=$(jq -er '.collection_id' inventory-collection.json)
eal --workspace "$example" --registry "$example/tools.toml" \
  reason case.eal --collection "$collection_id" --context '{}' \
  > inventory-assessment.json

assessment_id=$(jq -er '.assessment_id' inventory-assessment.json)
eal --workspace "$example" --registry "$example/tools.toml" \
  explain "$assessment_id" --claim inventory_verified

eal --workspace "$example" --registry "$example/tools.toml" \
  packet "$assessment_id" --claim inventory_verified > inventory-packet.json
```

The runtime records an assessment time when `--now` is omitted. Supplying a time
makes the judgement reproducible, but it must represent the intended assessment
instant and preserve sufficient timestamp precision. A truncated instant can
precede an observation collected in the same second.

These commands retain records in the example workspace's private `.eal` store.
A supported result is conditional on its declared scope and observations. A
negative measurement, expired observation or failed acquisition changes what the
argument can establish. Keep those records when investigating the result.

## Replace the fixture with your investigation

State the claim's system boundary and acceptance criterion, then identify the
observations and reviews each inference requires. For the CI example, specify
which commit, workload, runner allocation and outcome vector the timing comparison
covers. A successful benchmark needs a separate argument connecting that sample
to the proposed deployment.

Write those obligations in [EAL/4](language-v4.md), or use the
[EngArg engineer workflow](https://github.com/emmett08/engarg/blob/main/docs/USAGE.md)
to author and export the package. Select actual operations with a
[direct tool manifest](direct-tools.md), review their access and execute the
package with the same host command. Revisit the argument when evidence changes
its premises or scope.

Return to the [documentation path](index.md) for hosting, repeated assessment,
coordination and implementation details.
