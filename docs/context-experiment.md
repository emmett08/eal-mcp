# Measure packet and context behaviour

The context experiment investigates how retained evidence packets affect a
model-facing history. Its independent Rust package implements planning
calculations, synthetic history policies and native packet diagnostics for
[EAL-CONTEXT-001](../experiments/eal-context-001/eal-context-experiment.md). It is
excluded from the production workspace and has its own lockfile. Run the commands
below from the repository root.

The commands execute software diagnostics and assumed statistical models.
Provider token savings and model answer quality remain unmeasured.

## Build and verify

Use the repository's Rust 1.90 toolchain.

```sh
cargo fmt --manifest-path experiments/eal-context-001/Cargo.toml --check
cargo clippy --manifest-path experiments/eal-context-001/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path experiments/eal-context-001/Cargo.toml --locked
cargo build --manifest-path experiments/eal-context-001/Cargo.toml --release --locked
```

The binary is `experiments/eal-context-001/target/release/eal-context` when no
`CARGO_TARGET_DIR` override is selected. The tests exercise whole-exchange eviction,
exact assessment identities, retrieval budgets, usage accounting and numerical
calibration. Existing output files are protected. Select a fresh output path for
each new run.

## Planning calculations

```sh
experiments/eal-context-001/target/release/eal-context planning allocation --output /tmp/eal-allocation.json
experiments/eal-context-001/target/release/eal-context planning sensitivity --output /tmp/eal-sensitivity.json
```

Both commands default to 6,000 replications per cell and record their seed.
`--replications` and `--seed` select explicit alternatives. The calculation notes
in [planning/calculation-notes.md](../experiments/eal-context-001/planning/calculation-notes.md) define the
assumed distributions, xoshiro256** generator and numerical profile. Reports
identify the implemented generator and estimator. The results describe the
selected planning assumptions.

## Synthetic history rehearsal

```sh
experiments/eal-context-001/target/release/eal-context rehearse --output /tmp/eal-rehearsal.json
```

This runs nine scripted traces totalling 720 synthetic turns. Its common 32,768
request ceiling is measured in serialised UTF-8 JSON bytes. Condition A retains
packets until whole-turn emergency eviction. Condition B evicts old exchanges
and restores the first exposed exchange from the host cache. Condition C requests
stored packets through a three-level retrieval ladder. Failed retrieval consumes
its selected budget level. Original assessment identities and times remain fixed.
The report identifies the source and JSON serialisation used for its byte counts.

## Native packet comparison

```sh
experiments/eal-context-001/target/release/eal-context probe --reference-root /path/to/reference-checkout --reference-revision REFERENCE_SHA40 --candidate-root /path/to/candidate-checkout --candidate-revision CANDIDATE_SHA40 --build --output /tmp/eal-packet-comparison.json
```

Replace both revision arguments with exact 40-digit commit identities. The command
requires distinct clean checkouts. `--build` compiles the CLI, MCP server and
runtime companions into each checkout's `target/debug` directory. `--cargo PATH`
and `--git PATH` explicitly select development executables. Without `--build`,
prepare those binaries beforehand. Reports record source revisions, binary hashes
and whether this invocation rebuilt them.

Native diagnostic children receive an empty `PATH`. Commands have a 240-second
deadline and a combined 8 MiB output allowance. The trusted local run owns the
children and their process groups. It removes temporary private stores on exit.

The diagnostic exercises actual `eal-host` to MCP stdio to `eal-mcp` requests.
It checks default packet compatibility, stored retrieval without new records,
original identity and time after source edits, registered allowlists, model
context and private-observation exclusion. Packet byte counts and hashes use the
raw JSON packet carried by MCP. The cross-revision comparison normalises only
assessment, collection and observation identifiers.

[Results](../experiments/eal-context-001/results.md) distinguish executed diagnostics from the unrun model-session
study. These tools perform no provider requests and assign no model quality scores.

Return to [development](development.md) or the [documentation path](index.md).
