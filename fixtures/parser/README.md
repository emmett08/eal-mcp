# Python parser reference corpus

`reference.json` records the pinned Python implementation's results for the
101 authored sources in `manifest.json`. The corpus covers all 76 parser rules
in the unchanged EAL/3 grammar. It includes the maintained synthetic API example,
existing regression sources, malformed input and 62 controlled newline, comment
and Unicode variants.

Each case separates two observations:

- `recognised` records generated lexer/parser acceptance with **both** error
  listeners checked. A recovered tree accompanied by an error is rejected.
- `lowered` records the source implementation's complete `parse` operation,
  including structural field checks, trusted import resolution and composition.
  It is retained for later port stages. It is broader than lexical AST lowering.

Recognition is also compared through `tree`, Python ANTLR's `toStringTree`, and
all `tokens`, including hidden comments and excluding EOF. Token offsets count
Unicode scalar values with an exclusive end. Lines and columns are 1-based.
Bare carriage returns follow the pinned ANTLR runtime's line accounting. Rust's
additional UTF-8 byte offsets can be checked directly against the source.
Diagnostic wording for rejected cases is retained as reference information.

Grammar acceptance permits cases that structural checks reject, such as
repeated fields, repeated decoded JSON object keys and unpaired surrogate
escapes. The corpus also includes recognised/lowered sources with deliberately
invalid semantic properties, including an unknown reference, unsupported
language version and negative evidence age. Acceptance here establishes the
parser-stage result for that source, rather than a valid engineering argument.

`import` is lowered in the Python reference with an explicit in-memory resolver.
`import-without-resolver` is grammatically accepted and fails complete source
lowering. Reading an import statement in Rust's lexical AST performs no IO.

## Refreshing the oracle

The normal Rust test suite reads these files and needs neither Python nor Java.
To refresh the oracle during development, use the unchanged source repository at
`e2d6f346a37f92483b9f9a13923b98feb053447b` with
`antlr4-python3-runtime==4.13.2` installed:

```sh
python tools/capture-parser-reference.py /path/to/earl
python tools/capture-parser-reference.py /path/to/earl --check
```

The capture rejects a different source revision or modified production Python
and grammar files. It disables bytecode writes and imports only the pinned
reference package. The generated JSON uses one line per case to keep retained
parse trees and token streams compact. Edit the source fixtures and manifest,
then regenerate the oracle.

The source repository is attributed in `THIRD_PARTY_NOTICES.md`. Test fixtures
are derived from the source paths identified in the manifest. Comments label
synthetic data where examples represent observations.
