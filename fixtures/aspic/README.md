# Finite ASPIC reference

`reference.json` retains 149 synthetic solver inputs captured from the unchanged
Python implementation at `earl@e2d6f346a37f92483b9f9a13923b98feb053447b`.
There are 100 structurally valid theories, 87 completed solver results and 13
solver budget failures. Every case runs both theory validation and full solving;
invalid payloads also retain the explicit theory-validation input. The native
comparison checks every output field, ordered argument/subargument/defeat and
extension array, digest, exact error string and invalid/incomplete error class.
It applies no numerical tolerance or backend metadata projection.

The corpus covers axioms and ordinary premises, strict and defeasible rules,
all three defeat kinds, one-way and reciprocal contraries, propagated attacks,
multiple defeat witnesses, all three preference policies, explicit partial
priorities and closure, and all semantics/query-mode combinations. It retains
alternative, Cartesian and shared-diamond derivations, conclusion quantifiers
across distinct argument witnesses, premise-founded cycles, unfounded cycles,
unconstructed goals, stable absence, strict/profile consistency rejections,
all formal resource budgets and raised argument budgets, including undecided
preferred/stable alternatives whose extension masks cross 128 bits. Seven cases use the
production source-to-ASPIC compiler on synthetic source regression variants.

The independent native checks use their own finite-subset definitions of
conflict freedom, defence, admissibility, maximal admissibility and stability.
They compare 12,606 queries across all 531 directed graphs with zero through
three nodes, including self-attacks. Each node is represented by a defeasible
rule over an isolated axiom basis; each graph edge undercuts the target rule's
distinct applicability atom. Removing that fixed axiom from returned extensions
recovers the original graph exactly. An additional 414 queries check all 69
irreflexive graphs with one through three nodes using ordinary-premise
contraries. These tests share no production extension-enumeration logic.

PyArg is not used by this retained capture or required by native tests. This
oracle records the identified Python implementation; the exhaustive subset
checks provide the separate implementation check. All labels and source-bound
records are synthetic, with no experimental measurement claims.

Regenerate or verify only with the pinned development source:

```sh
python3 tools/capture-aspic-reference.py ../earl-audit
python3 tools/capture-aspic-reference.py ../earl-audit --check
```

Native checks require neither Python nor external solvers.

The companion captures exercise the public composition boundaries:

| File | Captured cases | Comparison |
| --- | --- | --- |
| `compiler-reference.json` | 76, including 64 snapshots and 61 solved projections | Complete theories, source maps, assessments and projections; declared backend metadata and dependent digests are checked independently |
| `export-reference.json` | 62: 20 full views and 42 rejections | Exact views and rejection categories, including supplied provenance and availability codes |
| `method-reference.json` | 5 typed-source assessments | Accepted theory, query mismatch, scope mismatch, stale observation and incomplete construction |

Each has a corresponding `capture-aspic-compiler-reference.py`,
`capture-aspic-export-reference.py` or `capture-aspic-method-reference.py` under
`tools`, accepting the same pinned source directory and `--check` option.
Native method identity differs from Python's process backend. Tests project
only the known contract locations, verify every native identity separately and
check the one incomplete diagnostic's exact budget message independently of
its backend-specific prefix. Literal observation payloads remain unchanged.
