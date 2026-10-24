//! Explicitly versioned native random stream for prospective simulations.

pub(super) struct Random {
    state: [u64; 4],
    spare: Option<f64>,
}

impl Random {
    pub(super) fn new(seed: u64) -> Self {
        let mut state = seed;
        let mut words = [0; 4];
        for word in &mut words {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = state;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            *word = value ^ (value >> 31);
        }
        Self {
            state: words,
            spare: None,
        }
    }

    fn next(&mut self) -> u64 {
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let shift = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= shift;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }

    fn uniform(&mut self) -> f64 {
        // 52 random bits select midpoints strictly inside (0, 1).
        ((self.next() >> 12) as f64 + 0.5) * (1.0 / 4_503_599_627_370_496.0)
    }

    pub(super) fn normal(&mut self) -> f64 {
        if let Some(value) = self.spare.take() {
            return value;
        }
        let radius = (-2.0 * self.uniform().ln()).sqrt();
        let angle = std::f64::consts::TAU * self.uniform();
        let (sine, cosine) = angle.sin_cos();
        self.spare = Some(radius * sine);
        radius * cosine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_is_seeded_and_finite_with_calibrated_first_two_moments() {
        let mut first = Random::new(20261021);
        let mut second = Random::new(20261021);
        let count = 100_000;
        let mut sum = 0.0;
        let mut square = 0.0;
        for _ in 0..count {
            let sample = first.normal();
            assert_eq!(sample, second.normal());
            assert!(sample.is_finite());
            sum += sample;
            square += sample * sample;
        }
        assert!((sum / count as f64).abs() < 0.02);
        assert!((square / count as f64 - 1.0).abs() < 0.03);
    }

    #[test]
    fn stream_transition_matches_explicit_state_witness() {
        let mut random = Random {
            state: [1, 2, 3, 4],
            spare: None,
        };
        assert_eq!(random.next(), 11_520);
        assert_eq!(random.state, [7, 0, 262_146, 211_106_232_532_992]);
        assert_eq!(random.next(), 0);
    }
}
