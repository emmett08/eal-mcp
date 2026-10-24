//! Binary64 distribution functions used by the planning decision rules.

/// The normal CDF series has absolute error below 3e-15 on the tested domain.
/// Outside [-8, 8], the omitted tail is at most 6.23e-16.
pub(super) fn normal_cdf(x: f64) -> f64 {
    if x <= -8.0 {
        return 0.0;
    }
    if x >= 8.0 {
        return 1.0;
    }
    let square = x * x;
    let mut term = x;
    let mut sum = x;
    for odd in (3..1024).step_by(2) {
        term *= square / odd as f64;
        let next = sum + term;
        if next == sum {
            break;
        }
        sum = next;
    }
    (0.5 + sum * (-0.5 * square - 0.918_938_533_204_672_8).exp()).clamp(0.0, 1.0)
}

pub(super) fn normal_quantile(p: f64) -> f64 {
    bisect_quantile(p, -9.0, 9.0, normal_cdf)
}

fn bisect_quantile(p: f64, mut lower: f64, mut upper: f64, cdf: impl Fn(f64) -> f64) -> f64 {
    for _ in 0..100 {
        let midpoint = (lower + upper) * 0.5;
        if cdf(midpoint) < p {
            lower = midpoint;
        } else {
            upper = midpoint;
        }
    }
    (lower + upper) * 0.5
}

pub(super) fn student_quantile(p: f64, degrees: usize) -> f64 {
    let degrees = degrees as f64;
    let cdf = |x: f64| {
        let tail = 0.5 * regularized_beta(degrees / (degrees + x * x), degrees * 0.5, 0.5);
        if x >= 0.0 { 1.0 - tail } else { tail }
    };
    let mut bound = 2.0;
    while cdf(bound) < p || cdf(-bound) > p {
        bound *= 2.0;
    }
    bisect_quantile(p, -bound, bound, cdf)
}

fn log_gamma(x: f64) -> f64 {
    // Lanczos approximation, g=7, for positive arguments.
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        return std::f64::consts::PI.ln()
            - (std::f64::consts::PI * x).sin().ln()
            - log_gamma(1.0 - x);
    }
    let z = x - 1.0;
    let mut sum = COEFFICIENTS[0];
    for (index, coefficient) in COEFFICIENTS.iter().enumerate().skip(1) {
        sum += coefficient / (z + index as f64);
    }
    let t = z + 7.5;
    0.918_938_533_204_672_8 + (z + 0.5) * t.ln() - t + sum.ln()
}

fn regularized_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let factor =
        (log_gamma(a + b) - log_gamma(a) - log_gamma(b) + a * x.ln() + b * (-x).ln_1p()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        factor * beta_fraction(a, b, x) / a
    } else {
        1.0 - factor * beta_fraction(b, a, 1.0 - x) / b
    }
}

fn beta_fraction(a: f64, b: f64, x: f64) -> f64 {
    const FLOOR: f64 = 1e-300;
    let keep_nonzero = |value: f64| {
        if value.abs() < FLOOR {
            FLOOR.copysign(value)
        } else {
            value
        }
    };
    let mut c = 1.0;
    let mut d = 1.0 / keep_nonzero(1.0 - (a + b) * x / (a + 1.0));
    let mut result = d;
    for iteration in 1..=1024 {
        let m = iteration as f64;
        let twice = 2.0 * m;
        let first = m * (b - m) * x / ((a - 1.0 + twice) * (a + twice));
        d = 1.0 / keep_nonzero(1.0 + first * d);
        c = keep_nonzero(1.0 + first / c);
        result *= d * c;
        let second = -(a + m) * (a + b + m) * x / ((a + twice) * (a + 1.0 + twice));
        d = 1.0 / keep_nonzero(1.0 + second * d);
        c = keep_nonzero(1.0 + second / c);
        let change = d * c;
        result *= change;
        if (change - 1.0).abs() <= 4.0 * f64::EPSILON {
            return result;
        }
    }
    panic!("Incomplete beta continued fraction failed to converge in its planning domain");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_values_match_published_quantiles_and_symmetry() {
        for (x, expected) in [
            (0.0, 0.5),
            (1.0, 0.841_344_746_068_542_9),
            (-1.0, 0.158_655_253_931_457_07),
            (3.0, 0.998_650_101_968_369_9),
            (7.0, 0.999_999_999_998_720_1),
        ] {
            assert!((normal_cdf(x) - expected).abs() < 3e-15);
            assert!((normal_cdf(x) + normal_cdf(-x) - 1.0).abs() < 3e-15);
        }
        assert!((normal_quantile(0.95) - 1.644_853_626_951_472_2).abs() < 3e-14);
        assert!((normal_quantile(0.975) - 1.959_963_984_540_054).abs() < 3e-14);
    }

    #[test]
    fn student_quantiles_match_independent_distribution_identities() {
        // df=1 is the Cauchy distribution, df=2 has a closed form.
        assert!((student_quantile(0.95, 1) - (0.45 * std::f64::consts::PI).tan()).abs() < 1e-12);
        let t = student_quantile(0.975, 2);
        assert!((0.5 + t / (2.0 * (t * t + 2.0).sqrt()) - 0.975).abs() < 1e-14);
        // Odd degrees used by every planning cell have an independent finite
        // trigonometric integral recurrence. It avoids gamma and beta functions.
        for degrees in [95, 191, 383, 767, 1535] {
            for probability in [0.95, 0.975] {
                let t = student_quantile(probability, degrees);
                let angle = (t / (degrees as f64).sqrt()).atan();
                let mut partial = angle;
                let mut total = std::f64::consts::FRAC_PI_2;
                for exponent in (2..degrees).step_by(2) {
                    partial = angle.sin() * angle.cos().powi(exponent as i32 - 1) / exponent as f64
                        + (exponent - 1) as f64 / exponent as f64 * partial;
                    total *= (exponent - 1) as f64 / exponent as f64;
                }
                assert!((0.5 + partial / (2.0 * total) - probability).abs() < 4e-12);
            }
        }
    }
}
