# EAL-CONTEXT-001 execution report

Recorded 21 October 2026. The original protocol version is 0.1.0.

**The long-session model pilot has not run.** Executed planning simulations and
machinery checks are reported below. Empirical token savings, correctness,
objection retention and qualification retention remain unmeasured.

## What was executed

| Component | Evidence | Interpretation |
|---|---|---|
| Original protocol | [Retained specification](eal-context-experiment.md); SHA-256 in [execution status](execution-status.json) | Byte-identical to the supplied attachment. |
| Allocation and sensitivity simulations | [Execution record](planning/execution.json), [allocation results](planning/allocation-results.json), [sensitivity results](planning/sensitivity-results.json) | 258,000 synthetic studies in 43 cells; both result files reproduce the supplied bundle exactly. |
| Host-policy machinery | [Rehearsal results](rehearsal-results.json) | Synthetic history fixtures test implementation behaviour; they supply no empirical model outcomes. |
| Native packet machinery | [Native results](native-results.json) | Actual pinned reference/candidate processes test packet compatibility and retrieval. |
| Model pilot | [Execution status and prerequisites](execution-status.json) | Zero model sessions; provider usage is unavailable. |

The scripts, dependency pins, commands, exact revisions and binary hashes are
retained alongside these records. [Reproduction instructions](README.md) keep
all experimental dependencies outside the production build.

## Planning results

The scripts ran 6,000 synthetic studies per cell with seed 20261021 and declared
cell offsets. The runtime was Python 3.12.14, NumPy 2.3.5 and SciPy 1.17.0.
The 38 allocation cells and five sensitivity cells reproduce the protocol's
calculations. The following positively dependent cases assume a true arithmetic
token ratio of 0.80 and unchanged quality:

| Assumed SD of paired scenario quality differences | Independent scenarios | Three-arm sessions, two repeats | Simulated joint success | 95% Monte Carlo interval |
|---|---:|---:|---:|---:|
| 0.05 | 96 | 576 | 96.85% | 96.38–97.26% |
| 0.10 | 384 | 2,304 | 96.78% | 96.31–97.20% |
| 0.15 | 768 | 4,608 | 94.92% | 94.33–95.44% |

At 384 scenarios and quality SD 0.10, a true 12% token saving with unchanged
quality produces 61.07% simulated joint success. A true 20% token saving with a
one-percentage-point loss on each quality endpoint produces 37.38%; 1,536
scenarios raise that simulated probability to 97.03%. These sensitivities show
how strongly the assumed effect determines allocation.

Putting one component at its null boundary while making the other components
favourable produces joint false-success estimates from 4.57% to 5.48%. Monte
Carlo uncertainty and the assumed distributions constrain that calibration.
The [calculation notes](planning/calculation-notes.md) explain the paired
arithmetic-total contrast, quality differences and intersection-union rule.

These are conditional design calculations. They neither measure EAL performance
nor establish the final empirical sample size. The pilot must estimate
dispersion, dependence, missingness, scoring reliability and actual cost.

## Native retrieval diagnostic

All 14 native checks passed against the exact pinned revisions. Default packets
matched between the reference and candidate for simple, byte-pressure and
count-pressure fixtures after normalising generated assessment, collection and
observation identifiers. This checks those fixtures rather than every possible
packet.

For the byte-pressure fixture, the actual compact packet sizes were:

| Requested packet/statement budget, bytes | Actual packet bytes | Result |
|---|---:|---|
| 4,096 / 1,024 | 678 | Navigation fallback; detail omitted by the packet byte limit. |
| 16,384 / 1,024 | 9,600 | Detailed projection; 4,296 authored statement bytes remain omitted. |
| 65,536 / 4,096 | 13,705 | Complete projection for this fixture. |

Increasing the budget preserved the stored assessment, source, context, method
identity and original assessment time. No assessment, collection or observation
records were added by retrieval. After a current-source edit and reassessment,
retrieving the earlier assessment returned its original packet unchanged.

The count-pressure fixture remained incomplete at the largest byte/statement
budget: 24 arguments were retained and 16 were omitted. Its packet occupied
6,683 bytes. This demonstrates the continuing argument-count limit; requesting
more bytes does not recover details excluded by that limit. Registered allowlist
rejection and exclusion of a raw-observation sentinel also passed. A separate
CLI call exercised the native `ModelContextBuilder` message path; it created a
new assessment and was not used to simulate historical model context.

## Synthetic host rehearsal

Six software checks passed. Nine scripted traces exercised A/B/C at 40, 80 and
120 turns, completing 720 synthetic turns. The common request ceiling in this
diagnostic was **32,768 UTF-8 bytes**, counted over the complete serialised
synthetic request. It was not the protocol's 32,768-model-token ceiling.

| Synthetic horizon | A emergency turn removals | B emergency turn removals | C emergency turn removals |
|---|---:|---:|---:|
| 40 | 4 | 0 | 0 |
| 80 | 21 | 0 | 4 |
| 120 | 66 | 42 | 50 |

These traces check whole-exchange and complete-turn eviction, protected material,
exact cached bodies, historical revision identity, retrieval-ladder exhaustion,
deadlines and preservation of missing usage. The retrieval schedule was scripted,
packets and answers were invented fixtures, and requests were sampled once per
synthetic user turn. The counts establish exercised machinery behaviour; they
do not estimate model-directed retrieval, model tokens or answer quality.

## Empirical execution limits

The host exposes no authenticated provider endpoint with per-request usage.
Available native assistant delegation cannot expose the controlled model
history, exact deployment configuration and token instrument required here.
It therefore cannot substitute for the specified three-condition pilot.

The independent 24-family engineering corpus, time-indexed gold/answerability
ledgers, domain review and blinded dual scoring also remain outstanding. The
history module and packet diagnostic cover separate machinery paths. An
integrated provider runner with checked model-context messages, actual registered
tools, scripted event replay, calibrated usage accounting and complete raw logs
remains to be implemented and validated.

Consequently the four principal bounds cannot be evaluated. No comparative
efficiency, quality preservation or production-adoption conclusion follows from
this execution. The original pilot allocation remains 144 sessions and 11,520
scheduled user turns before tool loops; confirmation requires fresh families and
a pilot-informed prospective allocation.

## Next execution

Supply a reachable authenticated provider endpoint, freeze its exact model and
tokenizer, and calculate the pilot quote and monetary ceiling. Complete the
independent corpus and assessor calibration, then integrate and rehearse the
metered provider path. Execute all assigned pilot sessions, retain failures and
raw usage, and report descriptive paired estimates before sizing confirmation.

The [execution status](execution-status.json) records each missing prerequisite.
The 10% token-saving boundary and two-percentage-point quality margins remain
the protocol's provisional engineering tolerances.
