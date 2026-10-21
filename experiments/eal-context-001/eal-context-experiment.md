# Long-session evaluation of EAL packet retrieval

Protocol EAL-CONTEXT-001, version 0.1.0, 21 October 2026. Status: **specified**. This is a prospective design. No model sessions have been run. The attached calculations use stated hypothetical distributions and establish conditional operating characteristics, not empirical effectiveness or principal-study readiness.

## Question and decision

Does a host using bounded historical retrieval reduce cumulative model tokens relative to a reference host using the existing EAL packet implementation, while preserving answer correctness, relevant objections and qualifications through long sessions? How much of any improvement is achievable by caching existing packets in the host?

The principal comparison estimates the effect of a complete host policy. PR #9 supplies retrieval and byte/statement budgets. History eviction and restoration remain host responsibilities. The experiment must therefore specify both layers. The existing implementation already projects compact packets. A raw-source or duplicated-envelope baseline would answer a different question.

The principal conclusion is comparative performance for one pinned model and the sampled workloads. A production adoption decision additionally requires acceptable absolute answer quality and application-specific critical-error limits. Those requirements must be settled before deployment.

## Versions and three conditions

Freeze the existing packet reference at `de3a99dfb1003f43f75d7bfe18007179ecb4f4aa` and the candidate at `71661ea05225157961fe32b355db62dac42e3bba` ([PR #9](https://github.com/emmett08/eal-mcp/pull/9)). Record any subsequent revision as an amendment.

Use the candidate binary for all conditions, with its SQLite locking correction common to all. A compatibility adapter exposes only the original registered MCP descriptors in A and B, applies original default limits, and checks packet output against the pinned reference before the experiment. B additionally exposes a clearly identified host-cache lookup. This isolates context policy from the storage fix. It estimates the old packet behaviour on a common corrected runtime, rather than the performance of every byte of the old executable.

| Condition | Packet and history policy | Recovery of discarded evidence |
|---|---|---|
| A: existing-packet reference host | Original 16,384-byte packet and 1,024-byte statement defaults. Retain conversation and packet messages until the common input cap. | Original registered tools remain available, including reassessment. Historical packets evicted from model context are unavailable to this reference policy. |
| B: existing-packet host cache | Same original packets. After four completed user turns, replace old packet-bearing tool exchanges with reference records. | An explicit host lookup replays the exact cached default packet for its assessment. Cache storage is outside model context. |
| C: bounded historical retrieval | Same eviction rule as B. The first historical reload requests 4,096 packet bytes and 1,024 statement bytes. | `eal_get_known_packet` can expand the same assessment to 16,384/1,024 and then 65,536/4,096 bytes when relevant detail remains missing. Each level may be requested once per assessment per turn. |

The primary contrast is **C minus A**. B minus A and C minus B explain the contribution of host caching and the incremental value of the candidate policy. They receive estimates and intervals, with exploratory interpretation. A result favouring C over A alone supports the complete workflow comparison. If the product decision specifically requires an advantage over a simple cache, make C versus B the principal contrast in a prospective amendment before confirmatory data collection.

Common controls:

- Use `ModelContextBuilder` to give the model one checked representation of each assessment. Supply the applicable messages and the same interpretation instruction in every condition. Avoid sending the adapter's entire inspection envelope, which repeats content.
- Give all conditions the same reference index: entry, claim, assessment identifier, source/context/method identities and assessment time. It contains identifiers and provenance, with no copied semantic summary. Count its input tokens on every request.
- Preserve the same immutable task brief and identifier glossary. Keep all ordinary user turns and final answers subject to the same emergency history rule. Eviction removes whole tool-call/result exchanges and replaces them with an ordinary host reference message, maintaining valid message structure.
- Set the total input ceiling to 32,768 model tokens, including tools, instructions, index and current input. The selected model must support that input plus the output reserve. Before a request would exceed it, remove the oldest completed conversation blocks, excluding the protected brief, index and current turn. Record every removal. Abort the turn if the protected material alone cannot fit.
- Use the same 2,048-output-token limit, six model/tool cycles per user turn and 120-second user-turn deadline. Treat these as provisional experimental settings, frozen after the pilot. Disable undocumented host compaction, external memory and automatic history truncation.
- Keep the registered entry allowlist, method registry, non-target tools and scripted evidence events identical. Operator-only full explanations and raw observations remain outside all three registered-client conditions.
- Pin the method registry throughout the principal sessions. A method-registry change belongs in a separately labelled qualified-rejection diagnostic because historical retrieval requires a compatible registry fingerprint.
- Keep a process cache in B for previously exposed default packets, including historical revisions. A holds only the common reference index once a packet is evicted. C retrieves through the actual service. Record cache hits, local lookups, MCP requests and transferred bytes separately.
- Keep one deployment model and exact model version/configuration fixed. A second model is a separately reported replication. No averaging across models supports a claim about a model class.

The C policy is model-directed within the fixed ladder. The common instruction tells the model to retrieve an absent assessment when needed and to recover relevant omitted detail where authorised. It must qualify unresolved omissions. The host enforces the ladder and call ceilings, but does not consult the answer key or silently retrieve extra information. Count every intermediate model request and tool response actually shown to the model.

## Long-session workload

Use 24 independently authored pilot scenarios: four in each stratum below, with eight 40-turn, eight 80-turn and eight 120-turn sessions. Each stratum covers all three lengths, with its fourth scenario's length rotated across strata. Domain renamings and parameter substitutions from one underlying graph stay in one scenario family and count as one independent unit.

| Stratum | Required content and failure to expose |
|---|---|
| Stable interleaved decisions | Six to ten registered entries, repeated comparisons and returns to earlier decisions. Test confusion between similar entries and unnecessary reassessment. |
| Objections and alternative support | Active, undecided and defeated objections, their targets and alternative supporting routes. Test omission of a decisive objection or preservation of an obsolete one. |
| Time and scope | Stale evidence, failed predicates, unavailable observations, time-limited assumptions and environment mismatch. Test whether historical support becomes an unjustified current claim. |
| Calculation and interpretation | Checked numerical outputs, unfavourable calculated results, prose correspondence and method-binding qualifications. Test whether formal support is mistaken for a favourable practical conclusion. |
| Revision and correction | Scheduled changes to evidence, source, context and requirements, plus explicit historical/current questions. Test incorrect revision selection and failure to retire superseded conclusions. |
| Projection pressure | Long statements and dense graphs crossing byte/statement limits, plus separate cases crossing count/traversal limits. Test recoverable detail and appropriate acknowledgement of residual omissions. |

Adapt the tasks from deployment reviews, performance investigations, sensor acceptance and operational recovery. Have an experienced engineer review whether their event sequences resemble actual work. Without a sample of actual session traces, describe them as representative-by-design engineering workloads, with field representativeness still unvalidated.

Freeze all user turns and source/evidence updates before assignment. Use deterministic collector replay and a controlled assessment clock, with isolated store/catalogue copies per run. Both current assessments and historical retrieval operate on the event state appropriate to that turn. Exogenous changes occur by scripted turn, not by elapsed provider time or the model's choices.

Interleave substantive work instead of padding with meaningless filler. Balance early, middle and late placement of decisive material and vary lexical identifiers so projection ordering does not consistently protect the important facts. Include feasible revisit lags near 8, 24 and 48 intervening turns. The 48-turn lag applies only to sufficiently long sessions. Record realised input occupancy, accumulated tokens and eviction events. Length alone is not evidence of context pressure.

At every tenth user turn, use a normal decision probe, for example: “Can release R proceed in this environment? Give the decision and its basis.” The final turn is also a decision probe and is counted once. Predeclare the required facts, objections and qualifications at each probe. Use direct requests for objections and historical/current comparisons as separately labelled secondary probes. Do not use a preceding explicit reminder to improve the primary probe's score. If a direct diagnostic probe would contaminate later primary probes, run it on a forked session and charge its tokens to diagnostic overhead.

A common anchor packet/event dossier at the first exposure of each retention item supplies the same default-budget information to all conditions. Updated items receive the same scheduled anchor exposure. This fixes retention denominators before treatment-dependent retrieval. Items absent from the shared anchor are acquisition or omission-awareness targets, rather than retained-information targets.

Each scenario supplies at least two primary probes with nonempty gold sets for each retention endpoint. Probe eligibility is fixed in the ledger. Probes with no relevant objection are scored separately for correct absence and fabrication. Record actual re-exposures in intervening questions, retrieved packets and assistant answers, since these can rehearse the information.

## Measurements and scoring

| Outcome | Operational definition | Aggregation |
|---|---|---|
| Total model tokens | Sum provider-reported input plus output tokens over every workflow model request from initialisation through the final scheduled turn. Include tool selection, answer generation, retries, any helper model and repeated context. Cached input is a subset of input, counted once. Add reasoning tokens only if the provider's output total excludes them. | Average repeats within scenario, then ratio of arithmetic mean totals C/A. Report input/output/cached/reasoning components and cumulative curves at turns 20/40/80/120 where present. |
| Retrieval calls | Every logical request for prior assessment information, including cache lookup, MCP retrieval, escalation, failed call and retry. Classify successful result, cache hit and transport separately. Count discovery and reassessment separately, and report all tool calls as well. | Mean paired difference per scenario and calls per 100 scheduled user turns. A's zero calls to the new endpoint must not be described as zero evidence reacquisition. |
| Latency | Monotonic time from each scripted user turn's submission to its complete answer or timeout, including model, tool and retry time. Record time to first token and model/tool components. Human thinking and imposed gaps are excluded. | Session mean and cumulative observed wall time, with median/p95 user-turn latency. Also report deadline-restricted time, assigning failed or unserved scheduled turns the 120-second ceiling. Label penalised and observed times separately. |
| Answer correctness | Binary correctness of each primary decision probe: all predeclared decision-critical factual/numerical requirements, claim status and requested revision are correct. Freeze numerical tolerances. Correctly qualified uncertainty is correct when gold requires it. | Fraction of scheduled primary probes correct, then equal scenario weighting. An unanswered probe scores zero. Record item-level factual accuracy secondarily. |
| Objection retention | Fraction of predeclared, previously shared, currently relevant active/undecided objection atoms accurately carried into the answer with the correct target, status and consequence. | Equal probe weights within scenario, then equal scenario weights. Never divide by the number of objections the model chose to mention. |
| Qualification retention | Fraction of predeclared qualifications preserved so they actually constrain the conclusion: scope, original time, evidence availability, assumptions, prose correspondence, interpretation and incompleteness. | Same fixed-denominator aggregation. Separate each qualification class and final-turn scores. |

Provider usage is the primary token instrument. Reconcile streaming and non-streaming usage, cache fields, reasoning fields and retry records against known requests before the pilot. Save raw usage objects with the normalised totals. Tokeniser estimates support diagnosis and occupancy control, with their tokenizer/version recorded, but never silently replace missing billed usage. Evaluation-judge tokens and scenario-authoring tokens are reported as research overhead, outside workflow tokens. Common pre-acquisition of frozen evidence has its own setup time/cost ledger. If it uses a model, that model cost must be assigned consistently to the workflow or explicitly excluded from both with a stated amortisation assumption.

Create an independent, time-indexed gold ledger with `item_id`, meaning, target, required status, applicability interval, source/context/method revision, common exposure turn, eligible probe IDs, criticality, numerical tolerance and availability class at each packet budget. Preserve an answerability ledger showing what each policy can legitimately obtain. Reference EAL outputs can check implementation consistency, while the decision truth conditions and argument graphs are independently specified and reviewed.

Packet objections contain graph/status fields rather than authored objection prose. Assumptions also expose status/validation/time fields rather than all narrative meaning. Use meaningful projected premise-claim statements or the same protected identifier glossary in every condition. Score only semantic material supplied or legitimately recoverable. Questions requiring inaccessible raw details require a qualified response, not invented recall.

Each retention atom scores one only when its full required meaning is retained. “There are caveats” scores zero. “The assessment supported release at t0, but this retrieval establishes no fresh measurement at t1” can satisfy a historical-time qualification. Score invented objections, unsupported certainty, reversed meanings, and defeated/superseded objections presented as current separately. Report a strict session indicator for losing or contradicting any predesignated decision-changing item. This indicator supports failure analysis, with uncertainty on its frequency, and cannot establish zero risk from zero observed errors.

Use two assessors blinded to condition and token/latency outcomes for every primary decision answer. Adjudicate disagreements while retaining original scores. Freeze calibration examples with correct, subtly wrong, ambiguous and overconfident answers, including qualification loss. Require all deliberately decision-reversing errors in that calibration set to be detected and at least 95% atom-level agreement with the independently reviewed key. This validates tested examples, not all possible answers. An LLM scorer may assist triage only after validation. Its agreement with itself or another model is not the independent correctness standard.

## Assignment, failures and analysis

Each scenario runs in all three conditions, twice, from fresh state. Randomise condition order within scenario/repeat and balance order across provider-time blocks. Pair the same scripted scenario and model configuration. Record provider seeds if available, without assuming a seed makes calls deterministic. Isolate conversations, databases, local caches and memory. Record provider prompt-cache usage because it can change latency even when total token accounting remains complete.

The independent analysis unit is the scenario family. User turns, atoms, repeated runs and assessors are nested observations. Average the two repeats and the predeclared probe scores before primary inference. Equal scenario weights give equal weight to the six strata and three lengths under the declared allocation. Preserve the allocation proportions when enlarging the study. Report length/stratum effects as estimates, with uncertainty, rather than claiming subgroup superiority from separate significant/nonsignificant results.

Analyse all assigned sessions. Keep malformed tool requests, refusal, retrieval exhaustion, context overflow and early termination. Unserved primary probes and required retention atoms score zero. Count all actual resource consumption, report completion rates, and present deadline-penalised latency alongside observed latency. Cheap early failures cannot support a useful-efficiency conclusion unless quality also passes. Never compare only successful sessions.

Retry only transient provider/transport failures, at most twice with predeclared backoff inside the original deadline. Retain and charge each attempt. Do not retry bad answers for a better score. A proven harness failure invalidates its whole paired block under a logged rule, with the original retained and a fresh complete block rerun. Unknown token usage remains missing: recover it from provider records where possible, otherwise report bounds and suspend the principal token claim if the bound crosses the decision boundary. No zero imputation or complete-case cost comparison.

For each scenario i, let T_Ai and T_Ci be the mean total tokens over repeats. Estimate R_T = mean(T_Ci) / mean(T_Ai). Test the 10% saving boundary with the paired contrast X_i = T_Ci − 0.90 T_Ai and its one-sided 95% t upper bound. A negative upper bound supports R_T < 0.90. Obtain the ratio confidence bound by inverting the corresponding paired contrast test, rather than substituting a geometric mean of session ratios. For each quality endpoint, use the paired scenario-score difference and its one-sided 95% t lower bound. Report two-sided 95% intervals for estimation as well.

The proposed principal success rule is the conjunction of all four conditions:

1. Token ratio upper bound < 0.90.
2. Correctness difference lower bound > −0.02.
3. Objection-retention difference lower bound > −0.02.
4. Qualification-retention difference lower bound > −0.02.

The 10% saving and two-percentage-point margins are **provisional engineering tolerances**, not established values or user-approved losses. They distinguish useful savings from incidental differences and bound mean degradation separately. Confirm their practical acceptability before freezing confirmation. The simulated useful alternative is a 20% token saving with no quality loss. Power at the 10% boundary is the false-positive rate, not the target power.

This is one intersection-union claim: every component must pass. Marginal one-sided 5% tests control false success at at most 5% under a valid component test when any required condition fails. That argument does not cover choosing whichever model, subgroup or comparator looks best. Secondary comparisons and diagnostic probes remain explanatory. Latency and retrieval are trade-off estimates rather than additional unpowered pass criteria. If latency becomes a release gate, specify its acceptable margin and size that joint rule before confirmation.

Use paired scenario bootstrap intervals, tail inspection and workload-stratified summaries as sensitivity analyses. Validate the principal estimator under the final pilot-informed distributions, including non-normality, score discreteness, heterogeneous task difficulty, scoring error, differential failures and missing telemetry. The attached preliminary simulations cover a narrower assumed family and do not substitute for that step.

## Pilot and numerical allocation assessment

First rehearse assignment, data collection and scoring on six separate short calibration cases, including known omission, revision, timeout and telemetry failures. Those cases and existing source fixtures are machinery checks, excluded from confirmation. Then run the 24-scenario pilot: three conditions, two repeats, **144 long sessions and 11,520 scripted user turns before tool loops**. Its purpose is to measure paired variance, repeat dependence, calls, latency tails, censoring, scoring disagreement, policy compliance and resource use. It can diagnose design defects and estimate effects descriptively. It is too small to promise the requested non-inferiority precision.

Freeze a new protocol after the pilot. Use fresh scenario families for confirmation. Select sample size from conservative uncertainty bounds for nuisance quantities and independently chosen effects, not from a favourable pilot treatment effect. Retain poor pilot outcomes and policy amendments in the record.

The supplied `allocation.py` executed 6,000 simulated studies per cell with independent scenario means, lognormal token costs and bounded score differences. It tests the actual arithmetic-total contrast and the three quality differences. These illustrative scenarios assume a true token ratio of 0.80, zero quality changes and two-repeat scenario means. They compare independent endpoints with positively dependent endpoints.

| Assumed SD of paired scenario quality differences | Independent scenarios | Three-arm sessions, two repeats | Simulated joint success, dependent case | 95% Monte Carlo interval |
|---|---:|---:|---:|---|
| 0.05 | 96 | 576 | 96.85% | 96.38–97.26% |
| 0.10 | 384 | 2,304 | 96.78% | 96.31–97.20% |
| 0.15 | 768 | 4,608 | 94.92% | 94.33–95.44% |

These are evaluated candidates, not a final selected allocation or a proof they are minimal. Across the two dependence settings, the corresponding lower joint-success point estimates are 96.73%, 96.40% and 94.40%. At SD 0.10, 96 scenarios yield only 35.45% joint success in the dependent case. At SD 0.15, 384 yield 65.82%. The quality margins dominate allocation.

Effects nearer the decision boundary need substantially more information. At 384 scenarios and quality SD 0.10, a true 12% token saving with unchanged quality yields only 61.07% joint success. A true 20% saving with a one-percentage-point decline on each quality score yields 37.38%, rising to 97.03% at 1,536 scenarios. These sensitivity calculations are in `sensitivity.py`. The table therefore does not establish that 384 scenarios will suffice for every practically acceptable effect.

Boundary simulations put one component at its null boundary while the others are favourable. Joint false-success point estimates range from 4.57% to 5.48%, with Monte Carlo uncertainty up to 6.09%. This is broadly consistent with a nominal 5% procedure within the simulated family, rather than evidence that empirical false success is guaranteed below 5%. The script also reports component power, 95% coverage, harmful cases and repeat-allocation comparisons.

For the secondary estimation outcomes, hypothetical paired SDs of 0.30 for normalised latency and three retrieval calls give approximate 95% half-widths of 3.0 percentage points and 0.30 calls per session at 384 scenarios. These are precision illustrations, not measured variability or precision for the latency p95. The pilot must estimate latency tails and the actual count distribution.

For an initial quality calculation, N ≈ [(z_0.95 + z_0.975) × SD / 0.02]^2 allocates at most 2.5% failure probability to each of four components, giving at least 90% joint power by a union bound when the assumed approximations and component powers hold. It gives 82, 325 and 731 independent scenarios for the three SDs before balancing and exact evaluation. It does not size the token component or establish the final design by itself.

After the pilot, compare one versus two repeats and spend additional runs on new independent scenarios when between-scenario variation dominates. Evaluate candidate sizes using the exact planned analysis and the observed missingness/censoring mechanisms. Require the conservative Monte Carlo bound on joint success to reach 90% across the retained useful-effect and adverse-nuisance scenarios, with adequately calibrated null-boundary performance. Verify the selected design with fresh simulation seeds. If no affordable design meets the targets, report an estimation study with its attainable precision. Keep the quality margins intact.

Model choice, endpoint prices and the spending ceiling remain unspecified. Before execution, calculate the quote from the calibrated per-call input/output distributions and current model prices, including repeats, tool loops, retries, scenario authoring and blinded scoring. The pilot already contains 11,520 user turns, so its cost must be explicit. No paid runs are authorised or executed by this design. A budget stop produces a labelled partial dataset and cannot silently become the planned confirmatory sample.

## Interpretation and next action

| Observation | Permitted interpretation | Next action |
|---|---|---|
| Four principal bounds pass, with valid measurement | C improves token efficiency relative to the specified A host within the stated quality margins and sampled conditions. | Inspect absolute quality, critical losses, latency and C–B before choosing a deployment policy. |
| Tokens improve but a quality bound shows a loss beyond tolerance | Resource saving carries unacceptable loss under the proposed criterion. | Revise retrieval/eviction or protect required information, then evaluate fresh tasks. |
| Quality intervals cross the margin | Preservation remains unresolved, even if the average difference is small. | Report precision and collect the prospectively justified additional independent cases or retain an inconclusive finding. |
| The C–B interval remains compatible with both useful incremental benefit and negligible benefit | The incremental value over host caching remains unresolved. Nonsignificance establishes no equivalence. | If the implementation choice depends on that increment, prospectively make C–B the principal comparison and size it accordingly. |
| Mean quality passes but a decisive qualification is systematically lost | Comparative average performance alone is inadequate for the affected use. | Investigate the mechanism and apply a task-specific release gate. |
| An independently evidenced instrumentation, scoring or harness-delivery fault invalidates a block | The affected empirical inference is suspended. | Repair the apparatus and rerun the complete paired block, retaining the original. Model noncompliance, malformed requests, refusal and retrieval exhaustion remain outcomes, with their original costs and quality penalties. |

The immediate work is to implement the small replay/instrumentation harness outside the production port, author the 24 pilot scenarios and their independent gold ledgers, validate the six calibration cases, and freeze the model and pilot budget. The codebase's AGENTS.md places experiments and research reporting outside the Rust port, so this protocol does not add research infrastructure to eal-mcp.

A larger input-cap sensitivity and a separately pinned second model can test transport beyond the principal 32,768-token condition. Treat each as a declared extension. The primary result does not establish intrinsic model-memory improvement, transcript-summarisation effectiveness, or universal performance across models and workloads.

## Evidence and reproducibility

Retain full model requests/responses, usage objects, tool arguments/results, visible-history hashes, eviction/reference records, event schedules, all attempts, monotonic/UTC timestamps, assessment identities, scoring/adjudication records, assignment seeds and all code/configuration hashes. Required log fields are specified in `record-schema.json`. Record cache warmth and runtime platform. Store gold answers outside the model-visible workspace. Freeze the analysis before confirmatory outcomes, with amendments timestamped and prospective/exploratory status explicit.

The following sources informed specific choices. They do not establish EAL effectiveness or validate the provisional margins:

- [EAL packet implementation at the pinned reference](https://github.com/emmett08/eal-mcp/blob/de3a99dfb1003f43f75d7bfe18007179ecb4f4aa/crates/eal-runtime/src/packets.rs) and [candidate packet documentation](https://github.com/emmett08/eal-mcp/blob/71661ea05225157961fe32b355db62dac42e3bba/docs/packets.md): define defaults, disclosure boundaries, recoverable byte/statement omissions and remaining count/traversal limits.
- [NIST, Randomized block designs](https://www.itl.nist.gov/div898/handbook/pri/section3/pri332.htm): motivates scenario pairing and provider-time blocks for nuisance control.
- [Lakens (2022), Sample Size Justification](https://doi.org/10.1525/collabra.33267): motivates explicit meaningful effects, precision and resource constraints instead of a conventional sample count.
- [Morris, White and Crowther (2019), Using simulation studies to evaluate statistical methods](https://doi.org/10.1002/sim.8086): motivates explicit estimands, data-generating assumptions, null-boundary calibration and Monte Carlo uncertainty.
- [Wu et al., LongMemEval, ICLR 2025](https://arxiv.org/abs/2410.10813v2): motivates separate temporal reasoning, knowledge-update and abstention tasks. This protocol uses independently authored EAL scenarios and makes no claim of benchmark-equivalent scores.

Bundle contents: this protocol, schema-1.2 investigation JSON, logging field contract, executed allocation script/results, validation result and reproduction instructions. Implementation and empirical execution remain planned.
