# Prospective allocation calculations

The native planning commands calculate operating characteristics under declared
nuisance distributions. Their results describe the specified simulation domain.
Main-study readiness requires an independent pilot and validation of the final
estimator against its intended operating conditions.

Run the isolated experiment crate from the repository root.

```sh
cargo run --release --locked --manifest-path experiments/eal-context-001/Cargo.toml -- \
  planning allocation --output /tmp/eal-allocation-native.json
cargo run --release --locked --manifest-path experiments/eal-context-001/Cargo.toml -- \
  planning sensitivity --output /tmp/eal-sensitivity-native.json
```

Both commands emit JSON on stdout and accept `--replications` and `--seed`.
The defaults are 6,000 replications per cell and seed 20261021. An explicit
`--output` path must be new. A small count such as `--replications 10` checks
execution and produces correspondingly wide Monte Carlo intervals.

## Independent units and estimands

Each independent unit is one scenario-level paired mean after its prespecified
repeats. Additional probes and repeated sessions remain within that scenario.
For baseline mean token cost A_i and treatment mean token cost C_i, the resource
estimand is E(C_i)/E(A_i). The token decision uses the paired contrast
X_i=C_i−0.90 A_i. Its upper one-sided 95% t bound must lie below zero.

Correctness, objection retention and qualification retention use paired score
differences D_i=score_C−score_A. Each lower one-sided 95% t bound must exceed
−0.02. The margin is two percentage points. These boundaries are provisional
scientific choices that require explicit acceptance before a principal study.
Overall success requires all four tests. This intersection-union decision
bounds false overall success by the type-I error of each true component null.
Its joint power depends on the endpoint covariance.

The implemented estimator uses unweighted independent scenario summaries.
Changes to weighting, stratification, missing-data policy or interval method
require a corresponding change to the simulation before allocation is chosen.

## Sampling family

Baseline tokens follow a lognormal law with arithmetic mean 1 and CV 0.5.
The treatment/baseline ratio is independent of baseline cost and lognormal with
CV 0.2. Its arithmetic mean is the selected effect. The population ratio of
arithmetic mean costs therefore equals that selected effect.

Quality differences have bounded uniform marginals with the specified means
and standard deviation. A Gaussian copula gives either independent endpoints
or latent quality correlation 0.5 and token/quality correlation 0.3. Latent
correlations differ from observed Pearson correlations of the uniform scores.
Paired quality scores are 0.8−D_i/2 and 0.8+D_i/2. Parameter validation keeps
both scores within [0,1] throughout the admitted simulation domain.

Allocation evaluates 38 cells across sample sizes 96, 192, 384 and 768.
Quality SDs are 0.05, 0.10 and 0.15. Additional cells place individual endpoints
on their decision boundaries or assign harmful effects. Sensitivity evaluates
five cells closer to the scientific boundaries, including N=1,536.
The sampling law assumes complete telemetry, complete sessions, accurate
scoring and the prespecified retry policy. Pilot evidence must determine the
additional fault and missing-data scenarios needed for the intended study.

## Native numerical contract

The random stream uses xoshiro256** with SplitMix64 initialisation and a
Box–Muller normal transform. Normal CDF evaluation uses a convergent series.
Clamping beyond absolute z of 8 omits at most 6.23×10⁻¹⁶ in either tail.
Student t quantiles use Lanczos log-gamma, an incomplete-beta continued
fraction and bisection. Every planning degree of freedom is checked against
an independent finite trigonometric integral recurrence. Sample variances use
Welford accumulation with the n−1 divisor.

Reports identify the implementation, per-cell seeds, random generator and
numerical profile. Exact stream replay also depends on the floating-point
platform's elementary functions. Per-cell success rates retain Monte Carlo
standard errors and Wilson 95% intervals. Two-sided 95% coverage is measured
separately for the token contrast and three quality differences.

## Allocation after the pilot

The independent pilot must estimate dispersion, covariance, repeat noise,
case heterogeneity, token tails, telemetry completeness and scoring reliability.
Freeze the final estimator and scientific margins before confirmation. Simulate
that estimator across plausible nuisance ranges and independently justified
effect scenarios. Choose an affordable allocation whose joint-success lower
Monte Carlo bound exceeds the declared target and whose calibration criteria
are satisfied. Otherwise retain an estimation or pilot claim.

A conservative normal approximation assigns marginal power at least 0.975 to
each of four endpoints to obtain joint power at least 0.90 by the union bound.
At unchanged quality it gives
N ≥ [(z_0.95+z_0.975)×SD/0.02]².
The resulting counts are 82, 325 and 731 before balancing and finite-sample
calibration. Token variance and proximity to its boundary need their own check.
The repeat-allocation comparison separates scenario heterogeneity from repeat
noise. It quantifies the trade-off between more independent scenarios and more
repeats at equal session counts under the specified variance assumptions.
