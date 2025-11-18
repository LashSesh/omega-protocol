/// Sweep Operator Ŝ_τ
///
/// Adaptive threshold filtering with temporal scheduling.
/// Provides DoS resilience through dynamic threshold adjustment.

use crate::types::*;
use crate::operators::OmegaOperator;

#[derive(Clone)]
pub struct Sweep {
    tau0: f64,      // Base threshold
    beta: f64,      // Gate width
    schedule: String, // "cosine" or "linear"
    t: f64,         // Current time
    period: f64,    // Schedule period
    delta_tau: f64, // Threshold variation
}

impl Sweep {
    pub fn new(tau0: f64, beta: f64, schedule: String) -> Self {
        Self {
            tau0,
            beta,
            schedule,
            t: 0.0,
            period: 100.0,
            delta_tau: 0.2,
        }
    }

    /// Apply sweep operator to vector
    pub fn transform(&mut self, v: &OmegaVector) -> OmegaVector {
        let mu = self.compute_mean(v);
        let tau = self.compute_threshold(self.t);
        let gate = self.sigmoid_gate(mu, tau);

        self.t += 1.0; // Advance time

        v.mapv(|x| gate * x)
    }

    /// Compute mean of vector
    fn compute_mean(&self, v: &OmegaVector) -> f64 {
        if v.is_empty() {
            return 0.0;
        }
        v.sum() / v.len() as f64
    }

    /// Sigmoid gate function
    fn sigmoid_gate(&self, x: f64, tau: f64) -> f64 {
        let z = (x - tau) / self.beta;
        1.0 / (1.0 + (-z).exp())
    }

    /// Compute threshold based on schedule
    fn compute_threshold(&self, t: f64) -> f64 {
        match self.schedule.as_str() {
            "cosine" => {
                let phase = std::f64::consts::PI * t / self.period;
                self.tau0 + 0.5 * (1.0 + phase.cos()) * self.delta_tau
            }
            "linear" => {
                let cycle = (t % self.period) / self.period;
                self.tau0 + cycle * self.delta_tau
            }
            _ => self.tau0,
        }
    }

    /// Get current threshold value
    pub fn current_threshold(&self) -> f64 {
        self.compute_threshold(self.t)
    }

    /// Reset time counter
    pub fn reset(&mut self) {
        self.t = 0.0;
    }
}

impl Default for Sweep {
    fn default() -> Self {
        Self::new(0.5, 0.1, "cosine".to_string())
    }
}

impl OmegaOperator for Sweep {
    type Input = OmegaVector;
    type Output = OmegaVector;
    type Params = SweepParams;

    fn apply(&self, input: Self::Input, _params: &Self::Params) -> Result<Self::Output> {
        let mut sweep = self.clone();
        Ok(sweep.transform(&input))
    }

    fn name(&self) -> &str {
        "Sweep"
    }

    fn lipschitz_constant(&self) -> f64 {
        1.0 // Non-expansive (gate ∈ [0,1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr1;

    #[test]
    fn test_sweep_filtering() {
        let mut sweep = Sweep::new(0.5, 0.1, "cosine".to_string());

        // High mean vector should pass
        let v_high = arr1(&[1.0, 1.0, 1.0, 1.0, 1.0]);
        let result_high = sweep.transform(&v_high);
        assert!(result_high.sum() > 0.0);

        // Low mean vector should be attenuated
        let v_low = arr1(&[0.1, 0.1, 0.1, 0.1, 0.1]);
        let result_low = sweep.transform(&v_low);
        assert!(result_low.sum() < v_low.sum());
    }

    #[test]
    fn test_threshold_schedule() {
        let mut sweep = Sweep::new(0.5, 0.1, "cosine".to_string());

        let tau0 = sweep.current_threshold();
        for _ in 0..50 {
            sweep.transform(&arr1(&[0.5; 5]));
        }
        let tau50 = sweep.current_threshold();

        // Threshold should change over time
        assert_ne!(tau0, tau50);
    }

    #[test]
    fn test_contractivity() {
        let mut sweep = Sweep::new(0.5, 0.1, "cosine".to_string());
        let v = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = sweep.transform(&v);

        // Result should not exceed input magnitude
        let max_input = v.iter().map(|&y| y.abs()).fold(0.0f64, f64::max);
        assert!(result.iter().all(|&x| x.abs() <= max_input));
    }
}
