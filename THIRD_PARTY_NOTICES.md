# Attribution and licences

The target repository's [GPL version 3 licence](LICENSE) is retained. New Rust
implementation code uses `GPL-3.0-only` as declared in the Cargo workspace.

## EAL source materials

The grammar, contract schemas and reference fixtures derive from
[Emmett Miller's EAL implementation](https://github.com/emmett08/earl/tree/e2d6f346a37f92483b9f9a13923b98feb053447b),
package `engineering-argument-language` version `3.2.4`. Its licence is the
[Unlicense](contracts/licenses/earl-Unlicense.txt). Imported artefact hashes and
source paths appear in `contracts/source-lock.json`. The source repository is
unchanged by this port.

The current visualiser schema was independently compared at
[`aspic_visualisation@a38bff4a193634556ef81e64f9b7823a1b1de6ec`](https://github.com/emmett08/aspic_visualisation/tree/a38bff4a193634556ef81e64f9b7823a1b1de6ec).
The retained schema is copied from the Unlicensed EAL producer.

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

## Coordination material reserved for a later stage

The source implementation embeds a Rust MAS codec derived from the MAS Protocol
Lab toolkit. Its source files, manifest and provenance record are identified in
the source manifest. The codec and its transaction-derived `lab_admission.py`
carry an [MIT licence](contracts/licenses/coordination-MIT.txt):

Copyright (c) 2026 Emmett Miller.

The codec implementation is pending port. Retain this MIT notice when those
sources are imported or translated.
