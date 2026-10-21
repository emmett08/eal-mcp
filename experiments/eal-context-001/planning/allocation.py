#!/usr/bin/env python3
"""Prospective, assumed-nuisance design calculations; no empirical model data.

Units are independent scenario-level paired means, after the chosen repeat
count. The main estimand is the ratio of arithmetic mean total tokens, not a
geometric mean of ratios. Quality estimands are mean paired score differences.
"""
import json
import math
from pathlib import Path

import numpy as np
import scipy
from scipy.special import ndtr
from scipy.stats import norm, t

SEED = 20261021
REPS = 6000
ALPHA = .05
TOKEN_BOUNDARY = .90
MARGIN = .02
N_VALUES = [96, 192, 384, 768]


def wilson(k, n, alpha=.05):
    z = norm.ppf(1-alpha/2)
    p = k/n
    den = 1+z*z/n
    centre = (p+z*z/(2*n))/den
    width = z*math.sqrt(p*(1-p)/n+z*z/(4*n*n))/den
    return [0.0 if k == 0 else float(max(0.0, centre-width)),
            1.0 if k == n else float(min(1.0, centre+width))]


def rate(values):
    k = int(np.sum(values))
    p = k/len(values)
    return {"estimate": p, "mcse": math.sqrt(p*(1-p)/len(values)),
            "wilson_95": wilson(k, len(values)), "replications": len(values)}


def simulate(n, sigma_quality, ratio=.8, quality_means=(0., 0., 0.),
             rho_quality=.5, rho_token_quality=.3, seed=SEED):
    """Run the actual paired-t decision on bounded score differences.

    TA is lognormal with mean 1 and CV .5. R is independent of TA,
    lognormal with mean `ratio` and CV .2. TB=TA*R. Thus E(TB)/E(TA)
    is exactly `ratio`. Their contrast TB-.9*TA tests arithmetic saving.

    Three score differences have uniform margins on mean +/-sqrt(3)*SD,
    formed by a Gaussian copula. Paired scores .8 +/- D/2 stay in [0,1]
    for all scenarios here. Correlations supplied are latent correlations,
    not Pearson correlations of the observed uniform differences. This
    deliberately exposes covariance sensitivity without claiming pilot fit.
    """
    rng = np.random.default_rng(seed)
    cov = np.full((4, 4), rho_quality)
    np.fill_diagonal(cov, 1.)
    cov[0, 1:] = rho_token_quality
    cov[1:, 0] = rho_token_quality
    chol = np.linalg.cholesky(cov)
    decisions, coverage = [], []
    crit = t.ppf(1-ALPHA, n-1)
    crit95 = t.ppf(.975, n-1)
    tau = math.sqrt(math.log(1.25))
    omega = math.sqrt(math.log(1.04))
    truth = np.array([ratio-TOKEN_BOUNDARY, *quality_means])
    for first in range(0, REPS, 200):
        m = min(200, REPS-first)
        z = rng.standard_normal((m, n, 4)) @ chol.T
        ta = np.exp(rng.normal(-tau*tau/2, tau, size=(m, n)))
        ratios = np.exp(math.log(ratio)-omega*omega/2+omega*z[:, :, 0])
        token_contrast = ta*(ratios-TOKEN_BOUNDARY)
        d = np.asarray(quality_means) + sigma_quality*math.sqrt(3)*(2*ndtr(z[:, :, 1:])-1)
        assert np.min(.8-np.abs(d)/2) >= 0 and np.max(.8+np.abs(d)/2) <= 1
        observations = np.concatenate([token_contrast[:, :, None], d], axis=2)
        avg = observations.mean(axis=1)
        se = observations.std(axis=1, ddof=1)/math.sqrt(n)
        keep = avg-crit*se > -MARGIN
        keep[:, 0] = avg[:, 0]+crit*se[:, 0] < 0
        decisions.append(keep)
        coverage.append(np.abs(avg-truth) <= crit95*se)
    decisions = np.concatenate(decisions)
    coverage = np.concatenate(coverage)
    return {
        "independent_scenarios": n, "sigma_paired_quality": sigma_quality,
        "arithmetic_token_ratio": ratio, "quality_means": list(quality_means),
        "latent_quality_correlation": rho_quality,
        "latent_token_quality_correlation": rho_token_quality,
        "component_pass": {name: rate(decisions[:, i]) for i, name in enumerate(
            ["tokens", "correctness", "objections", "qualifications"])},
        "joint_success": rate(decisions.all(axis=1)),
        "two_sided_95_coverage": {name: rate(coverage[:, i]) for i, name in enumerate(
            ["token_contrast", "correctness", "objections", "qualifications"])},
    }


def main():
    records = []
    for sigma in [.05, .10, .15]:
        for n in N_VALUES:
            for rhoq, rhotq in [(0., 0.), (.5, .3)]:
                key = int(1e5*sigma)+n+int(100*rhoq)
                result = simulate(n, sigma, rho_quality=rhoq,
                                  rho_token_quality=rhotq, seed=SEED+key)
                result["scenario_kind"] = "useful-benefit"
                records.append(result)
    boundaries = [
        ("token-boundary", .90, (0., 0., 0.)),
        ("correctness-boundary", .80, (-.02, 0., 0.)),
        ("objection-boundary", .80, (0., -.02, 0.)),
        ("qualification-boundary", .80, (0., 0., -.02)),
        ("all-boundaries", .90, (-.02, -.02, -.02)),
        ("token-harm", 1.05, (0., 0., 0.)),
        ("correctness-harm", .80, (-.04, 0., 0.)),
    ]
    for n, sigma in [(384, .10), (768, .15)]:
        for j, (name, r, q) in enumerate(boundaries):
            result = simulate(n, sigma, ratio=r, quality_means=q, seed=SEED+30000+n+j)
            result["scenario_kind"] = name
            records.append(result)
    # Normal approximation: each endpoint's failure probability <= .1/4
    # gives joint power >= .90 by union bound, without endpoint independence.
    zsum = norm.ppf(.95)+norm.ppf(.975)
    analytic = []
    for sigma in [.05, .10, .15]:
        raw_n = math.ceil((zsum*sigma/MARGIN)**2)
        balanced_n = math.ceil(raw_n/24)*24
        analytic.append({"sd": sigma, "normal_approx_n": raw_n,
                         "balanced_multiple_24": balanced_n,
                         "two_arm_sessions_with_two_repeats": 4*balanced_n,
                         "three_arm_sessions_with_two_repeats": 6*balanced_n})
    repeat_comparison = []
    for n, repeats in [(96, 2), (192, 1), (128, 3), (192, 2), (384, 1)]:
        # Scenario heterogeneity and model randomness each SD .08.
        sigma = math.sqrt(.08**2+.08**2/repeats)
        se = sigma/math.sqrt(n)
        marginal = norm.cdf(MARGIN/se-norm.ppf(.95))
        repeat_comparison.append({"n": n, "repeats": repeats,
                                  "two_arm_sessions": n*repeats*2,
                                  "quality_difference_se": se,
                                  "quality_marginal_power_approx": marginal,
                                  "quality_95_half_width_approx": 1.96*se})
    output = {
        "status": "prospective_assumed_nuisance_calculation_not_empirical_evidence",
        "seed": SEED, "replications_per_cell": REPS, "numpy": np.__version__,
        "scipy": scipy.__version__, "one_sided_alpha_per_component": ALPHA,
        "principal_claim": "all four component tests pass (intersection-union)",
        "token_estimand": "E(total_tokens_treatment)/E(total_tokens_baseline)",
        "token_boundary": TOKEN_BOUNDARY, "quality_margin_percentage_points": 2,
        "assumptions": [
            "Independent scenarios; paired differences are averages over prespecified repeats and scored probes.",
            "No missing telemetry, partial sessions, differential retries or scoring errors in these simulations.",
            "Uniform bounded quality differences and lognormal costs are planning families, not fitted evidence.",
            "Unstratified paired t at scenario level; a blocked/stratified production estimator requires corresponding resimulation.",
            "Parameter uncertainty remains unestimated until an independent pilot. This cannot establish main-study readiness.",
        ],
        "analytic_quality_conservative_allocation": analytic,
        "repeat_allocation_comparison": repeat_comparison,
        "simulation_results": records,
    }
    path = Path(__file__).with_name("allocation-results.json")
    path.write_text(json.dumps(output, indent=2)+"\n")
    print(path)
    for row in analytic:
        print("analytic", row)
    for row in records:
        if row["scenario_kind"] != "useful-benefit" or row["latent_quality_correlation"] == .5:
            print(row["scenario_kind"], row["independent_scenarios"], row["sigma_paired_quality"],
                  "joint", row["joint_success"]["estimate"], row["joint_success"]["wilson_95"])


if __name__ == "__main__":
    main()
