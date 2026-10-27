# EAL/4 Rust implementation

Use Rust 1.90.0, edition 2024, `cargo fmt`, and `cargo clippy`. Keep `Cargo.lock`.
Handwritten crates forbid unsafe code. The parser adapter denies unsafe code,
with a narrow exception for the unchanged generated module's context casts.
Keep workflow execution manually triggered.

EAL/4 is the supported language. Preserve `grammar/EAL.g4` unless the requested
change explicitly revises the language. Generate recognisers with
`tools/regenerate-parser.sh`. Keep generated files unchanged and isolate their
lint allowances. Normal builds use checked-in generated Rust sources.

Keep recognition, lexical lowering, composition, static validation and assessment
separate. Preserve arbitrary integers, binary64 operations and rational quantities
at their respective boundaries. Preserve authored assessment separately from
opt-in ASPIC compilation. Exhausted computation returns an incomplete result.
Keep IO, process execution and acquisition outside the pure language core.

Collectors receive bounded JSON requests through stdin. Operator configuration
selects external executables, endpoints, credentials and acquisition scope.
New adapters must preserve measured negatives, failed acquisition and unavailable
conclusions as separate outcomes. Reuse preserves original measurement age.

Tests establish native behavior and current public contracts. Use independently
authored cases, exhaustive finite definitions and adversarial checks as applicable.
Avoid original-implementation comparison fixtures and historical snapshots.
Keep current examples under `examples/` and document implemented behavior only.
Required third-party attribution belongs in `THIRD_PARTY_NOTICES.md`.

Run these commands for workspace changes:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

`eal-smoke` consolidates installed application, coordination and consumer checks.
The context experiment is a separate Rust crate under
`experiments/eal-context-001/`, outside production builds and published binaries.
Test and lint it explicitly through its manifest when changing experiment code.
