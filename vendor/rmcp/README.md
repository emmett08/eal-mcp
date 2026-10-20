# Pinned official rmcp SDK

This package retains the official `rmcp` 3.6.0 crates.io archive verbatim.
`UPSTREAM.json` records its origin, registry checksum, byte counts and license.
The archive contains the complete upstream source and tests; the original
Apache-2.0 license is also retained as `LICENSE`. The archive has no NOTICE file.

The local Cargo manifest preserves the official production dependencies and
features. It omits upstream test targets and development dependencies, disables
the vendor library's test, doctest and benchmark harnesses, and uses the local
build wrapper. The native MCP crate runs the wire regression tests. Upstream's
build script installs git hooks and is
**never executed** here.

At build time, Rust code verifies the archive's SHA-256 and exact size, bounds
decompression and each extracted file, admits only unique normal relative paths
and regular files, and reconstructs the source under Cargo's `OUT_DIR`. It then
applies the exact ordered hunks declared in `patches.rs`, requiring each original
fragment to occur exactly once. Any archive or patch mismatch fails the build.
The generated crate root gives upstream modules explicit source paths. Patched
root module files are copied byte for byte to adjacent `module/mod.rs` paths so
their existing child module directories resolve correctly; nested declarations
are unchanged. The local wrapper retains upstream's crate attributes and
documentation, forbids unsafe code, and allows only the compatibility lint for
crate-qualified exported macros introduced by the generated `include!`. Archive
reconstruction does not invoke Python, Java, shell tools, git, upstream build
scripts or network requests. Cargo resolves dependencies through the workspace
lockfile; once cached, the package builds offline.

The local codec patch addresses JSON value preservation when the SDK shares
`serde_json`'s arbitrary-precision feature with the EAL evaluator. Its behavioral
tests belong to the native MCP crate. This remains the official SDK with an
explicit local modification; it is not an unmodified upstream release.
