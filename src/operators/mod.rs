/// OMEGA Operator implementations
pub mod masking;
pub mod resonance;
pub mod sweep;
pub mod pfadinvarianz;
pub mod weight_transfer;
pub mod doublekick;

use crate::types::*;

/// Core trait for OMEGA operators
pub trait OmegaOperator {
    type Input;
    type Output;
    type Params;

    /// Apply the operator to input with given parameters
    fn apply(&self, input: Self::Input, params: &Self::Params) -> Result<Self::Output>;

    /// Get the operator name
    fn name(&self) -> &str;

    /// Get the Lipschitz constant for contractivity analysis
    fn lipschitz_constant(&self) -> f64;
}
