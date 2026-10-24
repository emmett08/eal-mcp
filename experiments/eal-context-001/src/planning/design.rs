//! Prespecified planning cells and analytic allocation comparisons.
use super::MARGIN;
use super::numerics::{normal_cdf, normal_quantile};
use super::simulation::{Scenario, simulate};
use serde_json::{Value, json};

pub(super) fn useful(n: usize, sd: f64, seed: u64) -> Scenario {
    Scenario {
        name: "useful-benefit",
        n,
        quality_sd: sd,
        ratio: 0.8,
        quality_means: [0.0; 3],
        quality_correlation: 0.5,
        token_quality_correlation: 0.3,
        seed,
    }
}

pub(super) fn allocation(replications: usize, seed: u64) -> Result<Vec<Value>, String> {
    let mut records = Vec::new();
    for sd in [0.05, 0.10, 0.15] {
        for n in [96, 192, 384, 768] {
            for (rhoq, rhotq) in [(0.0, 0.0), (0.5, 0.3)] {
                let offset = (100_000.0_f64 * sd) as u64 + n as u64 + (100.0_f64 * rhoq) as u64;
                let mut scenario = useful(n, sd, seed.wrapping_add(offset));
                scenario.quality_correlation = rhoq;
                scenario.token_quality_correlation = rhotq;
                records.push(simulate(scenario, replications)?);
            }
        }
    }
    let boundaries = [
        ("token-boundary", 0.90, [0.0, 0.0, 0.0]),
        ("correctness-boundary", 0.80, [-0.02, 0.0, 0.0]),
        ("objection-boundary", 0.80, [0.0, -0.02, 0.0]),
        ("qualification-boundary", 0.80, [0.0, 0.0, -0.02]),
        ("all-boundaries", 0.90, [-0.02, -0.02, -0.02]),
        ("token-harm", 1.05, [0.0, 0.0, 0.0]),
        ("correctness-harm", 0.80, [-0.04, 0.0, 0.0]),
    ];
    for (n, sd) in [(384, 0.10), (768, 0.15)] {
        for (index, (name, ratio, means)) in boundaries.iter().enumerate() {
            let mut scenario = useful(n, sd, seed.wrapping_add(30_000 + n as u64 + index as u64));
            scenario.name = name;
            scenario.ratio = *ratio;
            scenario.quality_means = *means;
            records.push(simulate(scenario, replications)?);
        }
    }
    Ok(records)
}

pub(super) fn sensitivity(replications: usize, seed: u64) -> Result<Vec<Value>, String> {
    let specs = [
        (96, 0.05, 0.85, [0.0; 3]),
        (384, 0.10, 0.88, [0.0; 3]),
        (384, 0.10, 0.80, [-0.01; 3]),
        (768, 0.10, 0.80, [-0.01; 3]),
        (1536, 0.10, 0.80, [-0.01; 3]),
    ];
    specs
        .iter()
        .enumerate()
        .map(|(index, (n, sd, ratio, means))| {
            let mut scenario = useful(*n, *sd, seed.wrapping_add(100_000 + index as u64));
            scenario.name = "boundary-sensitivity";
            scenario.ratio = *ratio;
            scenario.quality_means = *means;
            simulate(scenario, replications)
        })
        .collect()
}

pub(super) fn analytic_allocation() -> Value {
    let sum = normal_quantile(0.95) + normal_quantile(0.975);
    json!([0.05, 0.10, 0.15].map(|sd| {
        let raw = ((sum * sd / MARGIN).powi(2)).ceil() as usize;
        let balanced = raw.div_ceil(24) * 24;
        json!({"sd": sd, "normal_approx_n": raw, "balanced_multiple_24": balanced,
            "two_arm_sessions_with_two_repeats": 4 * balanced,
            "three_arm_sessions_with_two_repeats": 6 * balanced})
    }))
}

pub(super) fn repeat_allocation() -> Value {
    json!(
        [(96, 2), (192, 1), (128, 3), (192, 2), (384, 1)].map(|(n, repeats)| {
            let sigma = (0.08_f64.powi(2) + 0.08_f64.powi(2) / repeats as f64).sqrt();
            let se = sigma / (n as f64).sqrt();
            json!({"n": n, "repeats": repeats, "two_arm_sessions": n * repeats * 2,
            "quality_difference_se": se,
            "quality_marginal_power_approx": normal_cdf(MARGIN / se - normal_quantile(0.95)),
            "quality_95_half_width_approx": 1.96 * se})
        })
    )
}
