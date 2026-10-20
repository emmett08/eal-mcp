//! Finite Bayesian conditioning and Wilson binomial score intervals.
use super::validation::{integer, list, name, number, object, probability, result, text};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn inductive(value: &Value) -> Result<Value, String> {
    let o = object(value, &["successes", "trials", "confidence"], "sample")?;
    let n = integer(&o["trials"], "trials", 1, 1_000_000_000)?;
    let k = integer(&o["successes"], "successes", 0, n)?;
    let confidence = number(&o["confidence"], "confidence")?;
    if !(0.001..=0.999999).contains(&confidence) {
        return Err("confidence must be in [0.001, 0.999999]".into());
    }
    let z = normal_inv_cdf((1.0 + confidence) / 2.0);
    let nf = n as f64;
    let p = k as f64 / nf;
    let denominator = 1.0 + z * z / nf;
    let centre = (p + z * z / (2.0 * nf)) / denominator;
    let half = z * (p * (1.0 - p) / nf + z * z / (4 * n * n) as f64).sqrt() / denominator;
    Ok(result(
        true,
        "Wilson interval computed under the stated binomial sampling model",
        json!({"estimate":p,"lower":0.0f64.max(centre-half),"upper":1.0f64.min(centre+half),"sample_size":n,"successes":k,"confidence":confidence,"method":"wilson_score","interpretation":"approximate_frequentist","assumptions_verified":false}),
    ))
}

pub fn abductive(value: &Value) -> Result<Value, String> {
    let o = object(value, &["observed", "candidates"], "hypotheses")?;
    text(&o["observed"], "observed evidence description")?;
    let mut priors = BTreeMap::new();
    let mut log_weights = BTreeMap::new();
    for item in list(&o["candidates"], "candidates", 2, 128)? {
        let c = object(item, &["name", "prior", "likelihood"], "hypothesis")?;
        let name = name(&c["name"], "hypothesis name")?.to_owned();
        if priors.contains_key(&name) {
            return Err("Hypothesis names must be unique".into());
        }
        let prior = probability(&c["prior"], "prior")?;
        let likelihood = probability(&c["likelihood"], "likelihood")?;
        priors.insert(name.clone(), prior);
        log_weights.insert(
            name,
            if prior > 0.0 && likelihood > 0.0 {
                prior.ln() + likelihood.ln()
            } else {
                f64::NEG_INFINITY
            },
        );
    }
    let total_prior = super::numerical::fsum(priors.values().copied())?;
    if (total_prior - 1.0).abs() > 1e-9 {
        return Err("Prior probabilities must sum to 1 within absolute tolerance 1e-9".into());
    }
    let maximum = log_weights
        .values()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    if maximum == f64::NEG_INFINITY {
        return Ok(result(
            false,
            "The supplied hypotheses assign zero probability to the observed evidence",
            json!({"posterior":{},"best":null,"ties":[],"assumptions_verified":false}),
        ));
    }
    let weights: BTreeMap<_, _> = log_weights
        .iter()
        .map(|(name, w)| (name.clone(), (w - maximum).exp()))
        .collect();
    let normaliser = super::numerical::fsum(weights.values().copied())?;
    let posterior: BTreeMap<_, _> = weights
        .iter()
        .map(|(name, w)| (name.clone(), w / normaliser))
        .collect();
    let best_probability = posterior.values().copied().fold(0.0, f64::max);
    let ties: Vec<_> = posterior
        .iter()
        .filter(|(_, p)| (**p - best_probability).abs() <= 1e-12)
        .map(|(name, _)| name.clone())
        .collect();
    let best = if ties.len() == 1 {
        Some(ties[0].clone())
    } else {
        None
    };
    Ok(result(
        true,
        "Posterior probabilities computed within the supplied finite hypothesis model",
        json!({"posterior":posterior,"best":best,"best_posterior":best_probability,"ties":if ties.len()>1{ties}else{Vec::new()},"log_evidence_probability":maximum+normaliser.ln()-total_prior.ln(),"assumptions_verified":false}),
    ))
}

/// AS241 rational approximation, matching CPython's statistics.NormalDist.
/// Adapted from CPython 3.12 statistics.py; PSF licence retained in CPython-LICENSE.txt.
/// The supported confidence contract keeps tail r below 5, so the extreme-tail
/// branch of the general inverse distribution is outside this method's domain.
#[allow(clippy::excessive_precision)]
fn normal_inv_cdf(p: f64) -> f64 {
    let q = p - 0.5;
    let horner = |x: f64, coefficients: &[f64]| coefficients.iter().fold(0.0, |a, c| a * x + c);
    if q.abs() <= 0.425 {
        let r = 0.180625 - q * q;
        return horner(
            r,
            &[
                2.5090809287301226727e3,
                3.3430575583588128105e4,
                6.7265770927008700853e4,
                4.5921953931549871457e4,
                1.3731693765509461125e4,
                1.9715909503065514427e3,
                1.3314166789178437745e2,
                3.3871328727963666080,
            ],
        ) * q
            / horner(
                r,
                &[
                    5.2264952788528545610e3,
                    2.8729085735721942674e4,
                    3.9307895800092710610e4,
                    2.1213794301586595867e4,
                    5.3941960214247511077e3,
                    6.8718700749205790830e2,
                    4.2313330701600911252e1,
                    1.0,
                ],
            );
    }
    let r = (-(if q <= 0.0 { p } else { 1.0 - p }).ln()).sqrt() - 1.6;
    let x = horner(
        r,
        &[
            7.74545014278341407640e-4,
            2.27238449892691845833e-2,
            2.41780725177450611770e-1,
            1.27045825245236838258,
            3.64784832476320460504,
            5.76949722146069140550,
            4.63033784615654529590,
            1.42343711074968357734,
        ],
    ) / horner(
        r,
        &[
            1.05075007164441684324e-9,
            5.47593808499534494600e-4,
            1.51986665636164571966e-2,
            1.48103976427480074590e-1,
            6.89767334985100004550e-1,
            1.67638483018380384940,
            2.05319162663775882187,
            1.0,
        ],
    );
    if q < 0.0 { -x } else { x }
}
