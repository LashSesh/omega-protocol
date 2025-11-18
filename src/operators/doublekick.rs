/// DoubleKick Operator D̂_α
///
/// Dual orthogonal impulse for equilibrium escape.
/// Enables exploration and prevents local equilibria.

use crate::types::*;
use crate::operators::OmegaOperator;
use ndarray::Array1;
use rand::Rng;

pub struct DoubleKick {
    alpha1: f64,
    alpha2: f64,
    pub eta: f64, // Perturbation magnitude bound
}

impl DoubleKick {
    pub fn new(alpha1: f64, alpha2: f64) -> Self {
        let eta = alpha1.abs() + alpha2.abs();
        Self { alpha1, alpha2, eta }
    }

    /// Apply dual orthogonal kick
    pub fn apply(&self, v: &OmegaVector) -> OmegaVector {
        let dim = v.len();
        if dim == 0 {
            return v.clone();
        }

        // Generate two orthonormal vectors
        let (u1, u2) = self.generate_orthonormal_basis(dim);

        // Apply kicks: v' = v + α₁u₁ + α₂u₂
        v + &(u1 * self.alpha1) + &(u2 * self.alpha2)
    }

    /// Generate two random orthonormal vectors using Gram-Schmidt
    fn generate_orthonormal_basis(&self, dim: usize) -> (OmegaVector, OmegaVector) {
        let mut rng = rand::thread_rng();

        // Generate first random vector and normalize
        let mut u1 = Array1::from_vec(
            (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
        );
        let norm1 = u1.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm1 > 1e-10 {
            u1 = u1 / norm1;
        }

        // Generate second random vector
        let mut u2 = Array1::from_vec(
            (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
        );

        // Gram-Schmidt orthogonalization: u2 = u2 - (u2·u1)u1
        let dot_product: f64 = u1.iter().zip(u2.iter()).map(|(a, b)| a * b).sum();
        u2 = u2 - &(u1.clone() * dot_product);

        // Normalize u2
        let norm2 = u2.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm2 > 1e-10 {
            u2 = u2 / norm2;
        }

        (u1, u2)
    }

    /// Check if two vectors are orthogonal
    #[cfg(test)]
    fn are_orthogonal(u1: &OmegaVector, u2: &OmegaVector) -> bool {
        let dot: f64 = u1.iter().zip(u2.iter()).map(|(a, b)| a * b).sum();
        dot.abs() < 1e-6
    }

    /// Check if vector is normalized
    #[cfg(test)]
    fn is_normalized(u: &OmegaVector) -> bool {
        let norm_sq: f64 = u.iter().map(|x| x * x).sum();
        (norm_sq - 1.0).abs() < 1e-6
    }
}

impl Default for DoubleKick {
    fn default() -> Self {
        Self::new(0.05, -0.03)
    }
}

impl OmegaOperator for DoubleKick {
    type Input = OmegaVector;
    type Output = OmegaVector;
    type Params = DoubleKickParams;

    fn apply(&self, input: Self::Input, _params: &Self::Params) -> Result<Self::Output> {
        Ok(self.apply(&input))
    }

    fn name(&self) -> &str {
        "DoubleKick"
    }

    fn lipschitz_constant(&self) -> f64 {
        1.0 + self.eta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr1;

    #[test]
    fn test_doublekick_perturbation() {
        let dk = DoubleKick::new(0.1, -0.05);
        let v = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        let result = dk.apply(&v);

        // Result should be different from input
        assert_ne!(result, v);

        // Perturbation should be bounded
        let diff = &result - &v;
        let diff_norm = diff.iter().map(|x| x * x).sum::<f64>().sqrt();

        // Should be roughly α₁ + α₂ due to orthonormal basis
        assert!(diff_norm > 0.0);
        assert!(diff_norm < 1.0); // Reasonable bound
    }

    #[test]
    fn test_orthonormal_basis() {
        let dk = DoubleKick::default();
        let (u1, u2) = dk.generate_orthonormal_basis(5);

        // Check orthogonality
        assert!(DoubleKick::are_orthogonal(&u1, &u2));

        // Check normalization
        assert!(DoubleKick::is_normalized(&u1));
        assert!(DoubleKick::is_normalized(&u2));
    }

    #[test]
    fn test_near_isometry() {
        let dk = DoubleKick::new(0.01, 0.01);
        let v = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        let result = dk.apply(&v);

        let v_norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        let result_norm = result.iter().map(|x| x * x).sum::<f64>().sqrt();

        // For small perturbations, norm should be approximately preserved
        let lipschitz = dk.lipschitz_constant();
        assert!(result_norm <= lipschitz * v_norm + 0.1);
    }
}
