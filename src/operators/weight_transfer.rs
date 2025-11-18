/// Weight Transfer Operator Ŵ_γ
///
/// Multi-scale coherence redistribution for adaptive resilience.

use crate::types::*;
use crate::operators::OmegaOperator;
use ndarray::Array1;
use std::collections::HashMap;

#[derive(Clone)]
pub struct WeightTransfer {
    gamma: f64,
    weights: HashMap<ScaleLevel, f64>,
    target_weights: HashMap<ScaleLevel, f64>,
}

impl WeightTransfer {
    pub fn new(gamma: f64, weights: Vec<(ScaleLevel, f64)>) -> Self {
        let mut weight_map = HashMap::new();
        for (level, weight) in weights {
            weight_map.insert(level, weight);
        }

        let target_weights = weight_map.clone();

        Self {
            gamma,
            weights: weight_map,
            target_weights,
        }
    }

    /// Apply weight transfer
    pub fn transform(&mut self, v: &OmegaVector) -> OmegaVector {
        // Update weights: w' = (1-γ)w + γw̃
        self.update_weights();

        // Project onto multi-scale components
        let mut result = Array1::zeros(v.len());

        for (level, &weight) in &self.weights {
            let projection = self.project_to_scale(v, level);
            result = result + projection * weight;
        }

        result
    }

    /// Update weights adaptively
    fn update_weights(&mut self) {
        for (level, weight) in self.weights.iter_mut() {
            let target = self.target_weights.get(level).copied().unwrap_or(0.0);
            *weight = (1.0 - self.gamma) * *weight + self.gamma * target;
        }
    }

    /// Project vector to specific scale
    fn project_to_scale(&self, v: &OmegaVector, level: &ScaleLevel) -> OmegaVector {
        match level {
            ScaleLevel::Micro => {
                // High-frequency components (detail)
                self.highpass_filter(v)
            }
            ScaleLevel::Meso => {
                // Mid-frequency components (structure)
                self.bandpass_filter(v)
            }
            ScaleLevel::Macro => {
                // Low-frequency components (trend)
                self.lowpass_filter(v)
            }
        }
    }

    /// Simple lowpass filter (moving average)
    fn lowpass_filter(&self, v: &OmegaVector) -> OmegaVector {
        let mut result = Array1::zeros(v.len());
        let window = 3;

        for i in 0..v.len() {
            let mut sum = 0.0;
            let mut count = 0;

            for j in i.saturating_sub(window/2)..=(i + window/2).min(v.len() - 1) {
                sum += v[j];
                count += 1;
            }

            result[i] = sum / count as f64;
        }

        result
    }

    /// Simple highpass filter (difference from lowpass)
    fn highpass_filter(&self, v: &OmegaVector) -> OmegaVector {
        let lowpass = self.lowpass_filter(v);
        v - &lowpass
    }

    /// Simple bandpass filter (combination)
    fn bandpass_filter(&self, v: &OmegaVector) -> OmegaVector {
        let lowpass = self.lowpass_filter(v);
        let highpass = self.highpass_filter(v);
        (lowpass + highpass) * 0.5
    }

    /// Set target weights for adaptation
    pub fn set_target_weights(&mut self, targets: Vec<(ScaleLevel, f64)>) {
        self.target_weights.clear();
        for (level, weight) in targets {
            self.target_weights.insert(level, weight);
        }
    }

    /// Get current weights
    pub fn get_weights(&self) -> &HashMap<ScaleLevel, f64> {
        &self.weights
    }
}

impl Default for WeightTransfer {
    fn default() -> Self {
        Self::new(
            0.3,
            vec![
                (ScaleLevel::Micro, 0.2),
                (ScaleLevel::Meso, 0.5),
                (ScaleLevel::Macro, 0.3),
            ],
        )
    }
}

impl OmegaOperator for WeightTransfer {
    type Input = OmegaVector;
    type Output = OmegaVector;
    type Params = WeightTransferParams;

    fn apply(&self, input: Self::Input, _params: &Self::Params) -> Result<Self::Output> {
        let mut wt = self.clone();
        Ok(wt.transform(&input))
    }

    fn name(&self) -> &str {
        "WeightTransfer"
    }

    fn lipschitz_constant(&self) -> f64 {
        1.0 // Convex combination preserves norm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr1;

    #[test]
    fn test_weight_transfer() {
        let mut wt = WeightTransfer::default();
        let v = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        let result = wt.transform(&v);

        // Result should have same dimension
        assert_eq!(result.len(), v.len());

        // Should not be all zeros
        assert!(result.iter().any(|&x| x.abs() > 1e-10));
    }

    #[test]
    fn test_weight_adaptation() {
        let mut wt = WeightTransfer::default();
        let initial_micro = *wt.get_weights().get(&ScaleLevel::Micro).unwrap();

        // Set new target
        wt.set_target_weights(vec![
            (ScaleLevel::Micro, 0.8),
            (ScaleLevel::Meso, 0.1),
            (ScaleLevel::Macro, 0.1),
        ]);

        // Apply several times to adapt
        let v = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        for _ in 0..10 {
            wt.transform(&v);
        }

        let final_micro = *wt.get_weights().get(&ScaleLevel::Micro).unwrap();

        // Weight should have moved toward target
        assert_ne!(initial_micro, final_micro);
    }

    #[test]
    fn test_convex_combination() {
        let wt = WeightTransfer::default();
        let weights = wt.get_weights();

        let sum: f64 = weights.values().sum();

        // Weights should sum to approximately 1
        assert!((sum - 1.0).abs() < 0.01);
    }
}
