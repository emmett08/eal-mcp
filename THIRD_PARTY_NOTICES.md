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

## Coordination material reserved for a later stage

The source implementation embeds a Rust MAS codec derived from the MAS Protocol
Lab toolkit. Its source files, manifest and provenance record are identified in
the source manifest. The codec and its transaction-derived `lab_admission.py`
carry an [MIT licence](contracts/licenses/coordination-MIT.txt):

Copyright (c) 2026 Emmett Miller.

The codec implementation is pending port. Retain this MIT notice when those
sources are imported or translated.
