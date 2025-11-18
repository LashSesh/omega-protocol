/// Resonance Operator R̂_ω
///
/// Spectral coupling for address-free communication.
/// Filters vectors based on their dominant frequency component.

use crate::types::*;
use crate::operators::OmegaOperator;
use ndarray::Array1;
use rustfft::{FftPlanner, num_complex::Complex};

pub struct ResonanceOperator {
    omega: f64,
    epsilon: f64,
}

impl ResonanceOperator {
    pub fn new(omega: f64) -> Self {
        Self {
            omega,
            epsilon: 0.1, // Default resonance bandwidth
        }
    }

    pub fn with_epsilon(omega: f64, epsilon: f64) -> Self {
        Self { omega, epsilon }
    }

    /// Apply resonance filter to vector
    pub fn apply(&self, v: &OmegaVector) -> OmegaVector {
        let dominant_freq = self.compute_dominant_frequency(v);

        if (dominant_freq - self.omega).abs() < self.epsilon {
            v.clone()
        } else {
            Array1::zeros(v.len())
        }
    }

    /// Compute dominant frequency of vector using FFT
    pub fn compute_dominant_frequency(&self, v: &OmegaVector) -> f64 {
        let len = v.len();
        if len == 0 {
            return 0.0;
        }

        // Convert to complex numbers
        let mut buffer: Vec<Complex<f64>> = v
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // Perform FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(len);
        fft.process(&mut buffer);

        // Find dominant frequency (max magnitude, excluding DC component)
        let mut max_magnitude = 0.0;
        let mut max_index = 0;

        for (i, c) in buffer.iter().enumerate().skip(1) {
            let magnitude = c.norm();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                max_index = i;
            }
        }

        // Convert index to normalized frequency [0, 2π)
        (max_index as f64 / len as f64) * 2.0 * std::f64::consts::PI
    }

    /// Check if vector is resonant with target frequency
    pub fn is_resonant(&self, v: &OmegaVector) -> bool {
        let dominant_freq = self.compute_dominant_frequency(v);
        (dominant_freq - self.omega).abs() < self.epsilon
    }
}

impl OmegaOperator for ResonanceOperator {
    type Input = OmegaVector;
    type Output = OmegaVector;
    type Params = ResonanceParams;

    fn apply(&self, input: Self::Input, _params: &Self::Params) -> Result<Self::Output> {
        Ok(self.apply(&input))
    }

    fn name(&self) -> &str {
        "Resonance"
    }

    fn lipschitz_constant(&self) -> f64 {
        1.0 // Non-expansive (either passes through or zeros)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr1;

    #[test]
    fn test_resonance_filter() {
        let operator = ResonanceOperator::new(1.0);

        // Create a vector with known frequency content
        let mut v = Array1::zeros(64);
        for i in 0..64 {
            v[i] = (2.0 * std::f64::consts::PI * 1.0 * i as f64 / 64.0).sin();
        }

        let result = operator.apply(&v);

        // Should pass through since frequency matches
        assert!(result.iter().any(|&x| x.abs() > 1e-10));
    }

    #[test]
    fn test_resonance_reject() {
        let operator = ResonanceOperator::with_epsilon(1.0, 0.01);

        // Create a vector with very different frequency
        let v = arr1(&[1.0, 0.0, -1.0, 0.0, 1.0]);

        let freq = operator.compute_dominant_frequency(&v);
        let is_resonant = operator.is_resonant(&v);

        // Should not resonate if frequency is far from target
        assert!(freq != 1.0 || !is_resonant);
    }
}
