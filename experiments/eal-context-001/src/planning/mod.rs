//! Prospective allocation calculations over explicitly assumed nuisance families.

mod design;
mod numerics;
mod random;
mod simulation;

use design::{allocation, analytic_allocation, repeat_allocation, sensitivity};
use serde_json::{Value, json};
use std::path::PathBuf;

const SEED: u64 = 20_261_021;
const ALPHA: f64 = 0.05;
const TOKEN_BOUNDARY: f64 = 0.90;
const MARGIN: f64 = 0.02;
const MAX_REPLICATIONS: usize = 100_000;
const COMPONENTS: [&str; 4] = ["tokens", "correctness", "objections", "qualifications"];
const COVERAGE: [&str; 4] = [
    "token_contrast",
    "correctness",
    "objections",
    "qualifications",
];

struct Options {
    mode: String,
    replications: usize,
    seed: u64,
    output: Option<PathBuf>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mode = args.first().ok_or(USAGE)?.clone();
        if !matches!(mode.as_str(), "allocation" | "sensitivity") {
            return Err(USAGE.into());
        }
        let mut options = Self {
            mode,
            replications: 6000,
            seed: SEED,
            output: None,
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut arguments = args[1..].iter();
        while let Some(option) = arguments.next() {
            if !seen.insert(option.as_str()) {
                return Err(format!("Repeated option {option}"));
            }
            let value = arguments
                .next()
                .ok_or_else(|| format!("Missing value for {option}"))?;
            match option.as_str() {
                "--replications" => {
                    options.replications =
                        value.parse().map_err(|_| "Invalid replication count")?;
                    if !(1..=MAX_REPLICATIONS).contains(&options.replications) {
                        return Err(format!("Replications must be in 1..={MAX_REPLICATIONS}"));
                    }
                }
                "--seed" => options.seed = value.parse().map_err(|_| "Invalid u64 seed")?,
                "--output" => options.output = Some(PathBuf::from(value)),
                _ => return Err(format!("Unknown planning option {option}")),
            }
        }
        Ok(options)
    }
}

const USAGE: &str =
    "planning allocation|sensitivity [--replications 1..100000] [--seed U64] [--output NEW_FILE]";

/// Run the selected assumed-nuisance simulation with an explicit output destination.
pub fn run(args: &[String]) -> Result<Value, String> {
    let options = Options::parse(args)?;
    let records = if options.mode == "allocation" {
        allocation(options.replications, options.seed)
    } else {
        sensitivity(options.replications, options.seed)
    }?;
    let mut output = json!({
        "status": "prospective_assumed_nuisance_calculation_not_empirical_evidence",
        "schema": "EAL/context-planning/1",
        "implementation": "eal-context-experiment/native-planning/1",
        "mode": options.mode,
        "seed": options.seed,
        "replications_per_cell": options.replications,
        "random_generator": "xoshiro256**/splitmix64-seed/box-muller-52bit-midpoint/1",
        "random_stream_reproducibility": "Exact replay requires the recorded seed and implementation with the same floating-point platform.",
        "numerical_profile": {
            "arithmetic": "IEEE-754 binary64 with platform elementary functions",
            "normal_cdf": "convergent central series, tails clamped beyond absolute z of 8",
            "normal_cdf_omitted_tail_bound": 6.23e-16,
            "student_quantiles": "Lanczos log-gamma, incomplete-beta continued fraction and 100-step bisection",
            "sample_variance": "Welford accumulation with n-1 denominator"
        },
        "one_sided_alpha_per_component": ALPHA,
        "principal_claim": "all four component tests pass (intersection-union)",
        "token_estimand": "E(total_tokens_treatment)/E(total_tokens_baseline)",
        "token_boundary": TOKEN_BOUNDARY,
        "quality_margin_percentage_points": 2,
        "assumptions": [
            "Independent scenarios. Paired differences are averages over prespecified repeats and scored probes.",
            "The simulations assume complete telemetry and complete sessions with accurate scores and the prespecified retry policy.",
            "Uniform bounded quality differences and lognormal costs are assumed planning families.",
            "The estimator is an unstratified paired t at scenario level. A changed estimator requires a corresponding simulation.",
            "Parameter uncertainty requires an independent pilot before main-study readiness can be assessed."
        ],
        "simulation_results": records
    });
    if options.mode == "allocation" {
        output["analytic_quality_conservative_allocation"] = analytic_allocation();
        output["repeat_allocation_comparison"] = repeat_allocation();
    }
    if let Some(path) = options.output {
        // A fresh explicit destination protects previously retained evidence.
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("Cannot create {}: {error}", path.display()))?;
        let bytes = serde_json::to_vec_pretty(&output).map_err(|error| error.to_string())?;
        file.write_all(&bytes)
            .and_then(|()| file.write_all(b"\n"))
            .map_err(|error| format!("Cannot write {}: {error}", path.display()))?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::design::useful;
    use super::simulation::{cholesky, rate, simulate};
    use super::*;

    #[test]
    fn analytic_allocation_matches_conservative_normal_formula() {
        let rows = analytic_allocation();
        let counts: Vec<_> = rows
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["normal_approx_n"].as_u64().unwrap())
            .collect();
        assert_eq!(counts, [82, 325, 731]);
    }

    #[test]
    fn wilson_endpoints_keep_uncertainty_after_zero_or_all_successes() {
        let zero = rate(0, 6000);
        assert_eq!(zero["wilson_95"][0], 0.0);
        let z_squared = 1.959_963_984_540_054_f64.powi(2);
        let exact_zero_upper = z_squared / (6000.0 + z_squared);
        assert!((zero["wilson_95"][1].as_f64().unwrap() - exact_zero_upper).abs() < 1e-14);
        let all = rate(6000, 6000);
        assert_eq!(all["wilson_95"][1], 1.0);
        assert!(all["wilson_95"][0].as_f64().unwrap() < 1.0);
    }

    #[test]
    fn covariance_factor_reconstructs_selected_latent_correlations() {
        let scenario = useful(96, 0.1, SEED);
        let factor = cholesky(scenario).unwrap();
        for row in 0..4 {
            for col in 0..4 {
                let value: f64 = (0..4).map(|k| factor[row][k] * factor[col][k]).sum();
                let expected = if row == col {
                    1.0
                } else if row == 0 || col == 0 {
                    0.3
                } else {
                    0.5
                };
                assert!((value - expected).abs() < 1e-15);
            }
        }
    }

    #[test]
    fn deterministic_simulation_distinguishes_benefit_from_harm() {
        let benefit = simulate(useful(192, 0.05, SEED), 100).unwrap();
        let mut harmful = useful(192, 0.05, SEED);
        harmful.ratio = 1.05;
        let harm = simulate(harmful, 100).unwrap();
        assert!(benefit["joint_success"]["estimate"].as_f64().unwrap() > 0.9);
        assert_eq!(harm["joint_success"]["estimate"], 0.0);
        assert_eq!(simulate(harmful, 100).unwrap(), harm);
    }

    #[test]
    fn command_preserves_cell_coverage_and_native_identity() {
        let args = ["allocation", "--replications", "1"].map(str::to_owned);
        let report = run(&args).unwrap();
        assert_eq!(report["simulation_results"].as_array().unwrap().len(), 38);
        assert_eq!(
            report["implementation"],
            "eal-context-experiment/native-planning/1"
        );
        let args = ["sensitivity", "--replications", "1"].map(str::to_owned);
        assert_eq!(
            run(&args).unwrap()["simulation_results"]
                .as_array()
                .unwrap()
                .len(),
            5
        );
        assert!(run(&["allocation".into(), "--replications".into(), "0".into()]).is_err());
    }
}
