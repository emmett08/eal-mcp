# EAL-CONTEXT-001 diagnostic results

Recorded 24 October 2026. The model-session pilot remains unrun. The Rust commands
executed planning simulations and synthetic history traces. Provider token savings,
answer correctness and retention quality remain unmeasured.

| Executed component | Evidence | Scope |
|---|---|---|
| Allocation and sensitivity | [Execution record](planning/execution.json), [allocation](planning/allocation-results.json), [sensitivity](planning/sensitivity-results.json) | 43 cells with 6,000 synthetic studies each. |
| History policies | [Rehearsal report](rehearsal-results.json) | Nine scripted traces and 720 synthetic turns. |
| Native implementation tests | `cargo test --manifest-path experiments/eal-context-001/Cargo.toml --locked` | History, exact usage arithmetic, numerical identities and process boundaries. |

The [README](README.md) gives the executable Rust commands. The experiment remains
outside the production workspace and records its own implementation and lockfile.

## Planning results

The positively dependent cases below assume a true arithmetic token ratio of
0.80 with unchanged quality. The native report identifies its xoshiro256** stream,
cell seeds and numerical estimators.

| Assumed quality SD | Independent scenarios | Three-arm sessions with two repeats | Simulated joint success | 95% Monte Carlo interval |
|---|---:|---:|---:|---:|
| 0.05 | 96 | 576 | 96.40% | 95.90–96.84% |
| 0.10 | 384 | 2,304 | 97.20% | 96.75–97.59% |
| 0.15 | 768 | 4,608 | 94.82% | 94.23–95.35% |

At 384 scenarios and quality SD 0.10, an assumed 12% token saving with unchanged
quality gives 61.05% simulated joint success. An assumed 20% saving with a
one-percentage-point decline on every quality score gives 36.35%. At 1,536
scenarios that assumed quality-loss case reaches 96.63%. The selected effect
therefore materially changes the required allocation.

These figures describe assumed study distributions. The calculation notes define
the paired arithmetic-total contrast and intersection-union rule. Empirical
variance, dependence, missingness and scoring reliability remain unknown.

## Synthetic history rehearsal

Nine scripted traces exercised conditions A, B and C at 40, 80 and 120 turns.
The common ceiling was 32,768 UTF-8 bytes of the complete serialised synthetic
request. Model-token usage remains unavailable.

| Synthetic horizon | A emergency turn removals | B emergency turn removals | C emergency turn removals |
|---|---:|---:|---:|
| 40 | 4 | 0 | 0 |
| 80 | 21 | 0 | 4 |
| 120 | 66 | 42 | 50 |

The host preserves complete exchanges, original assessment identities and
reference metadata. Cache restoration returns the first exposed body. Service
restoration follows the bounded retrieval ladder, including failed requests.
Missing provider readings remain null and reasoning tokens are counted once.
The native report retains source hashes and every eviction event.

The scheduled retrievals, packets and placeholder answers are synthetic. These
traces establish the exercised history behaviour. Model-directed retrieval and
answer quality require separate evidence.

## Native packet comparison scope

`eal-context probe` implements comparison between two explicitly selected clean
Rust checkouts. It exercises actual CLI and MCP processes with an empty child
`PATH`, bounded IO and raw packet byte counts. The diagnostic checks stored
retrieval, identities, registered allowlists and model-context privacy.

This change verified the probe's fixture, identity-normalisation and process
helpers. It performed no full cross-revision probe run. Each future report records
the selected source revisions and binary hashes. [Execution status](execution-status.json)
keeps that boundary explicit.
