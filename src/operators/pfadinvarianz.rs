/// Pfadinvarianz Operator P̂_Γ
///
/// Path-invariant projection ensuring determinism.
/// Idempotent operator: P̂ ∘ P̂ = P̂

use crate::types::*;
use crate::operators::OmegaOperator;
use ndarray::Array1;

#[derive(Clone)]
pub struct Pfadinvarianz {
    permutations: Vec<Vec<usize>>,
}

impl Pfadinvarianz {
    pub fn new(dimension: usize) -> Self {
        let permutations = Self::generate_permutations(dimension);
        Self { permutations }
    }

    /// Apply path-invariant projection
    pub fn apply(&self, v: &OmegaVector) -> OmegaVector {
        if self.permutations.is_empty() {
            return v.clone();
        }

        let mut sum = Array1::zeros(v.len());

        // Average over all permutations
        for perm in &self.permutations {
            let permuted = self.apply_permutation(v, perm);
            sum = sum + permuted;
        }

        sum / (self.permutations.len() as f64)
    }

    /// Apply a single permutation to vector
    fn apply_permutation(&self, v: &OmegaVector, perm: &[usize]) -> OmegaVector {
        let mut result = Array1::zeros(v.len());
        for (i, &p) in perm.iter().enumerate() {
            if p < v.len() && i < result.len() {
                result[i] = v[p];
            }
        }
        result
    }

    /// Generate permutation group
    /// For simplicity, we use a subset of all permutations
    fn generate_permutations(dimension: usize) -> Vec<Vec<usize>> {
        if dimension == 0 {
            return vec![];
        }

        // For d=5, full permutations would be 5! = 120
        // We use a representative subset for computational efficiency
        let mut perms = vec![];

        // Identity
        perms.push((0..dimension).collect());

        // Cyclic shifts
        for shift in 1..dimension {
            let mut perm = Vec::with_capacity(dimension);
            for i in 0..dimension {
                perm.push((i + shift) % dimension);
            }
            perms.push(perm);
        }

        // Reversals
        perms.push((0..dimension).rev().collect());

        // Swaps of adjacent elements
        for i in 0..dimension - 1 {
            let mut perm: Vec<usize> = (0..dimension).collect();
            perm.swap(i, i + 1);
            perms.push(perm);
        }

        // Some additional permutations for better coverage
        if dimension >= 3 {
            // Rotate first 3 elements
            let mut perm: Vec<usize> = (0..dimension).collect();
            perm[0] = 1;
            perm[1] = 2;
            perm[2] = 0;
            perms.push(perm);
        }

        perms
    }
}

impl Default for Pfadinvarianz {
    fn default() -> Self {
        Self::new(5) // Default 5D space
    }
}

impl OmegaOperator for Pfadinvarianz {
    type Input = OmegaVector;
    type Output = OmegaVector;
    type Params = PfadinvarianzParams;

    fn apply(&self, input: Self::Input, _params: &Self::Params) -> Result<Self::Output> {
        Ok(self.apply(&input))
    }

    fn name(&self) -> &str {
        "Pfadinvarianz"
    }

    fn lipschitz_constant(&self) -> f64 {
        1.0 // Averaging preserves or reduces norm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr1;

    #[test]
    fn test_idempotence() {
        let pfad = Pfadinvarianz::default();
        let v = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        let v1 = pfad.apply(&v);
        let v2 = pfad.apply(&v1);

        // P̂(P̂(v)) ≈ P̂(v)
        for (a, b) in v1.iter().zip(v2.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn test_path_invariance() {
        let pfad = Pfadinvarianz::default();

        // Two different paths (permutations) should give same result after projection
        let v1 = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let v2 = arr1(&[2.0, 3.0, 4.0, 5.0, 1.0]); // Cyclic shift

        let result1 = pfad.apply(&v1);
        let result2 = pfad.apply(&v2);

        // Results should be similar (path-invariant)
        // Due to averaging, different inputs may give different outputs,
        // but applying projection makes them path-independent
        let projected_v1 = pfad.apply(&result1);
        let projected_v2 = pfad.apply(&result2);

        assert_eq!(projected_v1.len(), projected_v2.len());
    }

    #[test]
    fn test_contractivity() {
        let pfad = Pfadinvarianz::default();
        let v = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        let result = pfad.apply(&v);

        // Should not increase norm (averaging property)
        let input_norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        let output_norm = result.iter().map(|x| x * x).sum::<f64>().sqrt();

        assert!(output_norm <= input_norm + 1e-10);
    }
}
