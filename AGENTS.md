# EAL/3 Rust port

Port production code from `emmett08/earl` at the revision in
`contracts/source-lock.json`. Keep the source repository unchanged. Production
porting work excludes experiments, papers and research reporting scripts. The
explicitly requested EAL-CONTEXT-001 experiment lives separately under
`experiments/eal-context-001/`; its optional Python dependencies and execution
must remain outside production builds, tests and published binaries.

EAL/3 is the supported language. Preserve `grammar/EAL.g4` byte for byte. Keep
recognition, typed lexical lowering, binding, checking and evaluation separate.
Represent unsupported stages explicitly in the documentation. Successful syntax
recognition establishes syntax only.

Generate recognisers with `tools/regenerate-parser.sh`. Keep generated files
unchanged and isolate their lint allowances. Normal builds and tests use the
checked-in generated sources and fixtures, without Java or Python. The Python
fixture capture tool is a development oracle against the pinned source only.

Use Rust 1.90.0, edition 2024, `cargo fmt`, and `cargo clippy`. Keep `Cargo.lock`.
Handwritten core and CLI crates forbid unsafe code. The parser adapter denies
unsafe code, with a narrow exception for the unchanged generated module's
context casts. Run `cargo test --workspace --locked`,
`cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked --
-D warnings` for changes to the workspace. Keep workflows manually triggered.

Retain the authored EAL evaluator separately from opt-in ASPIC compilation.
Preserve arbitrary integers, binary64 operations and rational quantities at their
respective semantic boundaries. Never replace a solver's incomplete result with
a negative conclusion. Keep IO, process execution and acquisition outside the
pure language core.
