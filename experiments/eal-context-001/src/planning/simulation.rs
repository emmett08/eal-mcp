//! Scenario-level paired decisions under assumed joint sampling distributions.
use super::numerics::{normal_cdf, normal_quantile, student_quantile};
use super::random::Random;
use super::{ALPHA, COMPONENTS, COVERAGE, MARGIN, TOKEN_BOUNDARY};
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub(super) struct Scenario {
    pub(super) name: &'static str,
    pub(super) n: usize,
    pub(super) quality_sd: f64,
    pub(super) ratio: f64,
    pub(super) quality_means: [f64; 3],
    pub(super) quality_correlation: f64,
    pub(super) token_quality_correlation: f64,
    pub(super) seed: u64,
}

pub(super) fn cholesky(scenario: Scenario) -> Result<[[f64; 4]; 4], String> {
    let mut result = [[0.0; 4]; 4];
    for row in 0..4 {
        for col in 0..=row {
            let mut value = if row == col {
                1.0
            } else if col == 0 {
                scenario.token_quality_correlation
            } else {
                scenario.quality_correlation
            };
            for k in 0..col {
                value -= result[row][k] * result[col][k];
            }
            if row == col {
                if value <= 0.0 || !value.is_finite() {
                    return Err("The latent correlation matrix must be positive definite".into());
                }
                result[row][col] = value.sqrt();
            } else {
                result[row][col] = value / result[col][col];
            }
        }
    }
    Ok(result)
}

pub(super) fn simulate(scenario: Scenario, replications: usize) -> Result<Value, String> {
    let Scenario {
        n,
        quality_sd,
        ratio,
        quality_means,
        seed,
        ..
    } = scenario;
    if n < 2
        || ratio <= 0.0
        || !ratio.is_finite()
        || quality_sd < 0.0
        || !quality_sd.is_finite()
        || quality_means
            .iter()
            .any(|mean| !mean.is_finite() || mean.abs() + quality_sd * 3.0_f64.sqrt() > 0.4)
    {
        return Err("Simulation parameters must preserve finite costs and scores in [0, 1]".into());
    }
    let chol = cholesky(scenario)?;
    let critical = student_quantile(1.0 - ALPHA, n - 1);
    let critical95 = student_quantile(0.975, n - 1);
    let tau = 1.25_f64.ln().sqrt();
    let omega = 1.04_f64.ln().sqrt();
    let truth = [
        ratio - TOKEN_BOUNDARY,
        quality_means[0],
        quality_means[1],
        quality_means[2],
    ];
    let mut passes = [0_usize; 4];
    let mut covers = [0_usize; 4];
    let mut joint = 0;
    let mut random = Random::new(seed);
    for _ in 0..replications {
        let mut mean = [0.0; 4];
        let mut m2 = [0.0; 4];
        for sample in 1..=n {
            let independent = std::array::from_fn::<_, 4, _>(|_| random.normal());
            let z = std::array::from_fn::<_, 4, _>(|row| {
                (0..=row)
                    .map(|col| chol[row][col] * independent[col])
                    .sum::<f64>()
            });
            let baseline = (-tau * tau / 2.0 + tau * random.normal()).exp();
            let treatment_ratio = (ratio.ln() - omega * omega / 2.0 + omega * z[0]).exp();
            let mut observations = [0.0; 4];
            observations[0] = baseline * (treatment_ratio - TOKEN_BOUNDARY);
            for component in 0..3 {
                observations[component + 1] = quality_means[component]
                    + quality_sd * 3.0_f64.sqrt() * (2.0 * normal_cdf(z[component + 1]) - 1.0);
            }
            for component in 0..4 {
                let difference = observations[component] - mean[component];
                mean[component] += difference / sample as f64;
                m2[component] += difference * (observations[component] - mean[component]);
            }
        }
        let mut all = true;
        for component in 0..4 {
            let se = (m2[component].max(0.0) / (n - 1) as f64 / n as f64).sqrt();
            let pass = if component == 0 {
                mean[component] + critical * se < 0.0
            } else {
                mean[component] - critical * se > -MARGIN
            };
            passes[component] += usize::from(pass);
            covers[component] +=
                usize::from((mean[component] - truth[component]).abs() <= critical95 * se);
            all &= pass;
        }
        joint += usize::from(all);
    }
    let rates = |names: [&str; 4], counts: [usize; 4]| -> Value {
        Value::Object(
            names
                .into_iter()
                .zip(counts)
                .map(|(name, count)| (name.to_owned(), rate(count, replications)))
                .collect(),
        )
    };
    Ok(json!({
        "scenario_kind": scenario.name,
        "seed": seed,
        "independent_scenarios": n,
        "sigma_paired_quality": quality_sd,
        "arithmetic_token_ratio": ratio,
        "quality_means": quality_means,
        "latent_quality_correlation": scenario.quality_correlation,
        "latent_token_quality_correlation": scenario.token_quality_correlation,
        "component_pass": rates(COMPONENTS, passes),
        "joint_success": rate(joint, replications),
        "two_sided_95_coverage": rates(COVERAGE, covers)
    }))
}

pub(super) fn rate(successes: usize, replications: usize) -> Value {
    let n = replications as f64;
    let proportion = successes as f64 / n;
    let z = normal_quantile(0.975);
    let denominator = 1.0 + z * z / n;
    let centre = (proportion + z * z / (2.0 * n)) / denominator;
    let width =
        z * (proportion * (1.0 - proportion) / n + z * z / (4.0 * n * n)).sqrt() / denominator;
    json!({"estimate": proportion, "mcse": (proportion * (1.0 - proportion) / n).sqrt(),
        "wilson_95": [if successes == 0 {0.0} else {(centre - width).max(0.0)},
            if successes == replications {1.0} else {(centre + width).min(1.0)}],
        "replications": replications})
}
