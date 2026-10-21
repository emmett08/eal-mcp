# Prospective allocation calculations

These files contain computed design operating characteristics under declared,
invented nuisance distributions. They contain no model-session observations and
do not establish that the principal empirical experiment is adequately powered.

Run `python3 allocation.py` and `python3 sensitivity.py`. The executed environment
used Python 3, NumPy 2.3.5 and SciPy 1.17.0. Each simulation cell contains 6,000
independent simulated studies; seeds and package versions are retained in JSON.
The calculations use individual scenario-level paired means, with 2 repeats
already averaged when defining the assumed standard deviation. Session outcomes,
probes and repeat runs do not increase the number of independent scenarios.

## Decision and estimands

For scenario i, let A_i and C_i be mean total tokens across its repeated sessions
in the existing and proposed workflows. The deployment estimand is
E(C_i)/E(A_i), the ratio of arithmetic means, preserving total resource use.
To establish at least a 10% reduction, test the mean of X_i=C_i−0.90 A_i using
an upper one-sided 95% t bound; success requires that bound below zero.

For correctness, objection retention and qualification retention, use paired
scenario score differences D_i=score_C−score_A. Each lower one-sided 95% t bound
must exceed −0.02. The scientific margin is 2 percentage points, not a 2%
relative decline. These four margins are provisional design choices, rather
than accepted product thresholds. Overall success requires all four tests to
pass. This is an intersection-union claim: if any component null is true,
false overall success is bounded by that component's type-I error. Endpoint
independence is unnecessary for that bound. Joint power does depend on covariance.

The implemented simulation analyses unweighted independent scenario summaries.
If the final analysis changes to unequal weights, stratified variance estimates,
bootstrap bounds, mixed models or a different missing-data rule, reproduce that
procedure in the final numerical assessment before calling it ready.

## Assumed distributions

Baseline case tokens have a lognormal distribution with mean 1 and CV 0.5.
The treatment/baseline ratio is independent of baseline size and lognormal with
CV 0.2. Its arithmetic mean is the specified effect (normally 0.80).
Consequently the ratio of population arithmetic mean token totals equals that
effect exactly. Multiplying all token values by a common real token count leaves
the test and ratio unchanged.

Quality differences have bounded uniform marginal distributions, with their
specified mean and SD. A Gaussian copula creates either independent endpoints,
or latent quality correlation 0.5 and token/quality correlation 0.3. These are
latent, not observed Pearson correlations. Actual paired quality scores are
0.8−D/2 and 0.8+D/2, which remain within [0,1] in every simulated cell.
The quality SD scenarios 0.05, 0.10 and 0.15 describe scenario paired means.
They are not estimates from the repository or any model trial.

No missing telemetry, incomplete sessions, scoring errors or unequal lengths
are generated. The pilot must measure these, and the confirmatory allocation
must include materially plausible variants. Long-session length can be handled
by representative equal-frequency sampling and case-level inference, or by
prespecified weighting with a corresponding numerical assessment.

## Results and consequences

With true 20% token saving and no quality loss, the following are the lower joint
success estimates across the two evaluated correlation conditions. Brackets are
95% Wilson Monte Carlo intervals for that cell, not empirical treatment CIs.

| Paired quality SD | Independent scenarios | Joint success | Sessions, 3 arms × 2 repeats |
|---|---:|---:|---:|
| 0.05 | 96 | 0.9673 [0.9625, 0.9715] | 576 |
| 0.10 | 384 | 0.9640 [0.9590, 0.9684] | 2,304 |
| 0.15 | 768 | 0.9440 [0.9379, 0.9495] | 4,608 |

At SD 0.10, only 96 scenarios give joint success 0.3545 in the correlated
condition. At SD 0.15, 384 scenarios give 0.6582. These results expose how a
small experiment can fail to establish non-inferiority even when quality is
truly unchanged. Two-arm main-study costs are 4N sessions; retaining a third
explanatory cache arm costs 6N sessions.

Across the simulated partial-null cells, false overall success estimates range
from 0.0457 to 0.0548. The highest 95% Wilson upper bound is 0.0609. Two-sided
95% coverage estimates across all reported endpoint/cell combinations range
from 0.9437 to 0.9557. These are calibration diagnostics for the specified
distributions, not universal validation of t inference. Zero false successes in
some harmful/all-boundary cells have a nonzero 95% upper bound of 0.00064.

Effect assumptions matter as much as nuisance variation. With N=384, SD=0.10
and unchanged quality, a true token ratio 0.88 gives joint success 0.6107.
With ratio 0.80 but a true 1-percentage-point loss on all three quality scores,
joint success is 0.3738 at N=384, 0.7307 at N=768 and 0.9703 at N=1,536.
The 10% saving and 2pp loss boundaries must not be confused with the more
favourable 20% saving and zero-loss planning effects.

## Allocation rule after the independent pilot

Use the 24-scenario, 3-arm, 2-repeat pilot to estimate dispersion and covariance,
repeat noise versus case heterogeneity, token-tail behaviour, failure/telemetry
rates, scoring reliability and actual cost. Preserve its separation from
confirmation; do not select the principal comparison using a favourable pilot
treatment effect. A normal-reference 95% upper bound for a SD based on 24
independent cases is 1.326 times the observed SD, illustrating the substantial
nuisance uncertainty. Bounded/non-normal quality data require their own model.

Before principal collection, freeze the deployed model and estimator, retain
the fixed scientific margins, and simulate the final procedure over the pilot's
plausible nuisance ranges and independently justified effect scenarios. Choose
the least costly allocation whose joint-success lower Monte Carlo bound exceeds
0.90 and whose calibration criteria are met. If no affordable allocation meets
those conditions, retain an estimation/pilot claim and report the attainable
interval width. A budget cap is not a power justification.

For preliminary quality sizing, requiring each of four components to have
power at least 0.975 guarantees joint power at least 0.90 by the union bound.
The normal approximation at unchanged quality is
N ≥ [(z_0.95 + z_0.975) × SD / 0.02]^2.
It gives 82, 325 and 731 cases for SD 0.05, 0.10 and 0.15 before balancing and
finite-sample calibration. The simulation candidates above deliberately exceed
these counts. Token variance and proximity to its boundary must be checked too.

Additional independent scenarios often buy more information than repeats. With
case heterogeneity SD 0.08 and repeat noise SD 0.08, 96 scenarios × 2 repeats and
192 scenarios × 1 repeat both require 384 two-arm sessions. Approximate marginal
quality power rises from 0.639 to 0.789 with the larger independent case count.
At 768 two-arm sessions, powers are 0.789 for 128×3, 0.882 for 192×2 and 0.966
for 384×1. Actual authoring/scoring costs and within-case variance determine the
choice. Repeated variants of a shared underlying scenario remain clustered.

Latency and retrieval counts can initially be estimated rather than made
additional success gates. For a paired latency contrast normalised by baseline
mean latency with SD 0.30, a two-sided 95% interval has approximate half-width
0.060, 0.042, 0.030 and 0.021 at N=96,192,384,768. For retrieval-count paired SD
3 calls/session, those half-widths are 0.600, 0.424, 0.300 and 0.212 calls.
These are illustrative precision calculations. Tail latency and an operational
10% latency guardrail need their own pilot-calibrated analysis if used to decide
adoption.
