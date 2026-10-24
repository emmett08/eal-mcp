# Attribution and licences

This repository uses the [GPL version 3 licence](LICENSE). Rust code declares
`GPL-3.0-only` in the Cargo workspace.

## EAL source materials

The EAL grammar and schema materials are authored by Emmett Miller and supplied
under the [Unlicense](contracts/licenses/earl-Unlicense.txt). The required licence
text is included locally. The ASPIC visualiser schema is maintained in
`contracts/schemas/aspic-view.schema.json`.

## Generated recognisers and ANTLR runtime

The Rust recognisers are generated from the retained EAL grammar with
`antlr4rust` version `0.6.0`. The generator pin and SHA-256 are recorded by the
regeneration tool. The linked `antlr4rust` runtime is licensed under
[BSD 3-Clause](contracts/licenses/antlr4rust-BSD-3-Clause.txt):

Copyright (c) 2020–2021 Konstantin Anisimov, Alex Snaps. All rights reserved.

Other Cargo dependencies retain their own upstream licences. The lockfile fixes
the resolved dependency versions, and Cargo package metadata identifies their
licences. Binary distribution packaging must retain the applicable dependency
notices.

## CPython numerical algorithms

The native statistical methods adapt the inverse-normal approximation from
CPython 3.12's `statistics.py` (`NormalDist`), its scaled rational square-root
calculation (`_float_sqrt_of_frac`), and the built-in `sum` specialisation from
`Python/bltinmodule.c`, including compensated binary64 accumulation. The Rust
adaptations retain these algorithms while using Rust binary64 operations and
arbitrary-precision integers/rationals. They do not embed or invoke the Python
runtime.

These adaptations retain the Python Software Foundation License Version 2 and
the accompanying historical notices in
[`CPython-LICENSE.txt`](crates/eal-core/src/methods/CPython-LICENSE.txt).
Copyright (c) 2001–2023 Python Software Foundation; all rights reserved.

## Official MCP Rust SDK

The MCP transports use the official `rmcp` SDK, version `3.6.0`, under the
[Apache License 2.0](vendor/rmcp/LICENSE). Its original published crate is
retained with a verified digest in `vendor/rmcp`; the build applies the reviewed
codec changes described in [the patch notes](vendor/rmcp/README.md). These
changes preserve literal JSON object keys with arbitrary-precision numbers and
bound HTTP response decoding. The original SDK supplies protocol negotiation,
dispatch, cancellation and transports. Upstream source identity, archive hash
and the extraction procedure are retained in that directory.

## Coordination material

The source implementation embeds a Rust MAS codec derived from the MAS Protocol
Lab toolkit. Its source files, manifest and provenance record are identified in
the source manifest. The codec and its transaction-derived `lab_admission.py`
carry an [MIT licence](contracts/licenses/coordination-MIT.txt):

Copyright (c) 2026 Emmett Miller.

The native nom codec and mailbox admission translate this material. The MIT
notice remains retained with its source provenance; the native implementation
does not embed the Python Lab runtime.
