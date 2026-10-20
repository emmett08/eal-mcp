# Native visualiser integration

`eal-aspic-solve` accepts an explicit finite theory and emits a checked
`aspic-view/3` snapshot. The companion
[`aspic_visualisation`](https://github.com/emmett08/aspic_visualisation)
development server selects this executable explicitly. Rust owns argument
construction, extension search and recomputed export. The server owns HTTP,
request deadlines, concurrency and child cleanup. The browser retains its
schema and graph checks for both imported and freshly solved snapshots.

The companion integration is
[visualiser PR #6](https://github.com/emmett08/aspic_visualisation/pull/6), tested
at consumer revision
[`a86e0984`](https://github.com/emmett08/aspic_visualisation/commit/a86e0984c8aa36142ba3383ade6edb5dc170b873).
Merge the native producer change before that consumer change so its configured
executable can be installed.

## Installation and request protocol

Install `eal-cli` with the pinned Rust toolchain:

```sh
cargo install --path crates/eal-cli --locked
```

The standalone command reads one UTF-8 JSON object from stdin containing exactly
`theory`. Its theory follows the existing
[`argumentation/aspic/2` contract](aspic.md). It creates no catalogue, database or
collector process. A successful response contains only one checked view and a
newline on stdout, with exit status 0.

```sh
/absolute/installed/bin/eal-aspic-solve < theory-request.json > view.json
```

The adapter rejects duplicate JSON fields, trailing input, invalid UTF-8 and
unsupported theories. It reads at most 64 KiB plus one overflow-detection byte
and emits at most 4 MiB. Normal solver budgets still apply inside those byte
limits. Output is buffered until solving, export recomputation, schema checks
and the response-size check have succeeded.

Request, calculation and export failures leave stdout empty and emit one bounded
JSON object on stderr:

```json
{"error":{"code":"incomplete","message":"The selected resource budget was exhausted"}}
```

| Error code | Exit status | Meaning |
| --- | ---: | --- |
| `invalid_request` | 2 | The request, theory or export is outside the supported contract. |
| `incomplete` | 1 | Computation or response-size limits prevented a complete delivered view. |
| `unavailable` | 3 | The selected file or process IO was unavailable. |

A complete stable-semantics result can have no extensions. That result retains
`query_status: "no_extension"` and succeeds. An incomplete calculation supplies
no graph. A zero-argument result cannot satisfy the unchanged view schema's
minimum of one argument and is rejected during export.

An operator can supply `--limits /absolute/path/limits.toml`. The file must be a
regular TOML file of at most 16 KiB containing a `[limits]` table. These limits
select the finite construction and search profile. Browser requests cannot
select a budget file, raise the process byte limits or change server options.
The executable reads no ambient `EAL_LIMITS_FILE` setting.

## Companion development server

With the consumer integration checked out, enable its optional local editor:

```sh
ASPIC_SOLVER=1 \
ASPIC_SOLVER_EXECUTABLE=/absolute/installed/bin/eal-aspic-solve \
npm run dev -- --host 127.0.0.1
```

The configured executable must be an absolute path. There is no executable-name
search. The child receives a closed environment with an empty `PATH`; no
interpreter, provider credential or collector configuration is needed. Optional
server setting `EAL_LIMITS_FILE=/absolute/path/limits.toml` becomes the explicit
`--limits` argument.

The server accepts one same-origin JSON upload at a time, bounds upload and
solver duration, and terminates its owned solver after timeout or disconnect.
It validates the complete returned graph before returning a successful HTTP
response. A failed edit preserves the last accepted graph and its session
events. The optional bridge is disabled by default and absent from the static
production build. It remains a trusted local development facility.

## Reproducible acceptance

Normal Rust checks use no Python or Java:

```sh
cargo test --workspace --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Install all four packages from the root README into one prefix, then run the
retained application and coordination checks:

```sh
python tools/smoke-installed-applications.py --prefix /absolute/installed/bin
cargo build --release --locked -p eal-coordination --example coordination_software_worker
python tools/smoke-installed-coordination.py \
  --prefix /absolute/installed/bin \
  --worker /absolute/checkout/target/release/examples/coordination_software_worker
```

These standard-library Python programs orchestrate native commands with child
`PATH` empty. They retain their fresh synthetic workspaces and JSON results.
Their orchestration is a development check, separate from production execution.

In the consumer checkout, run the declared TypeScript, unit, build and browser
checks, followed by the actual native bridge suite:

```sh
npm ci
npm run typecheck
npm test
npm run build
npx playwright install chromium
npm run test:browser
ASPIC_SOLVER_EXECUTABLE=/absolute/installed/bin/eal-aspic-solve npm run test:browser:native
```

The static suite exercises file imports and graph inspection. The native suite
submits theories through the real local HTTP bridge and native executable.
Neither suite establishes measurement authenticity, correspondence between
formal atoms and prose, or improvements in model or operator decisions.

The retained producer and consumer changes passed the complete native workspace
check (386 tests, formatting and all-target Clippy), 81 consumer unit tests,
type checking and the production build. Browser checks passed 12 static cases
and five actual native cases using Node 24.19.0, Playwright 1.58.2 and Chromium
143.0.7499.0 on Linux. The native cases cover the engineering demo's exact undercut
and surviving route, preferred query versus grounded labels, stable
`no_extension`, incomplete-result preservation and partial last-link priority.
