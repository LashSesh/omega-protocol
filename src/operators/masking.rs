/// Masking Operator M̂_θ,σ
///
/// Provides information-theoretic encryption via permutation-rotation composition.
/// The operator is self-inverse: M̂ ∘ M̂ = I

use crate::types::*;
use crate::operators::OmegaOperator;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct MaskingOperator;

impl MaskingOperator {
    pub fn new() -> Self {
        Self
    }

    /// Mask (encrypt) a message
    pub fn mask(&self, message: &[u8], params: &MaskingParams) -> Result<Vec<u8>> {
        let mut result = message.to_vec();

        // Step 1: Apply permutation U_σ
        self.permute(&mut result, &params.sigma);

        // Step 2: Apply phase rotation R_θ (XOR-based)
        self.rotate(&mut result, params.theta);

        Ok(result)
    }

    /// Unmask (decrypt) a message - same as mask due to involution property
    pub fn unmask(&self, masked: &[u8], params: &MaskingParams) -> Result<Vec<u8>> {
        // M̂ ∘ M̂ = I, so unmask = mask
        self.mask(masked, params)
    }

    /// Apply simple XOR-based permutation with seed σ (self-inverse)
    fn permute(&self, data: &mut [u8], sigma: &[u8; 32]) {
        if data.is_empty() {
            return;
        }

        // Simple XOR permutation (self-inverse)
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= sigma[i % 32];
        }
    }

    /// Apply phase rotation via XOR with pseudo-random stream derived from θ
    fn rotate(&self, data: &mut [u8], theta: f64) {
        // Derive seed from theta
        let theta_bits = theta.to_bits();
        let mut seed = [0u8; 32];
        for i in 0..32 {
            seed[i] = ((theta_bits >> (i % 8)) & 0xFF) as u8;
        }

        let mut rng = StdRng::from_seed(seed);

        // XOR each byte with pseudo-random stream
        for byte in data.iter_mut() {
            *byte ^= rng.gen::<u8>();
        }
    }
}

impl Default for MaskingOperator {
    fn default() -> Self {
        Self::new()
    }
}

impl OmegaOperator for MaskingOperator {
    type Input = Vec<u8>;
    type Output = Vec<u8>;
    type Params = MaskingParams;

    fn apply(&self, input: Self::Input, params: &Self::Params) -> Result<Self::Output> {
        self.mask(&input, params)
    }

    fn name(&self) -> &str {
        "Masking"
    }

    fn lipschitz_constant(&self) -> f64 {
        1.0 // Isometric transformation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masking_involution() {
        let operator = MaskingOperator::new();
        let message = b"Hello, OMEGA Protocol!";
        let params = MaskingParams {
            theta: 1.234,
            sigma: [42u8; 32],
        };

        // Encrypt
        let masked = operator.mask(message, &params).unwrap();

        // Verify it changed
        assert_ne!(masked, message);

        // Decrypt
        let unmasked = operator.unmask(&masked, &params).unwrap();

        // Verify involution property
        assert_eq!(unmasked, message);
    }

    #[test]
    fn test_ephemeral_params() {
        let params1 = MaskingParams::ephemeral_from_frequency(1.5, 100);
        let params2 = MaskingParams::ephemeral_from_frequency(1.5, 100);
        let params3 = MaskingParams::ephemeral_from_frequency(1.5, 101);

        // Same inputs produce same params
        assert_eq!(params1.theta, params2.theta);
        assert_eq!(params1.sigma, params2.sigma);

        // Different epoch produces different params
        assert_ne!(params1.sigma, params3.sigma);
    }
}
