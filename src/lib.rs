/*!
# OMEGA Protocol

Operator-driven Meta-architecture for Encrypted Ghost-like Autonomous Networks

## Overview

OMEGA is a revolutionary network protocol built upon six fundamental operator classes:

1. **Masking Operator (M̂_θ,σ)**: Information-theoretic encryption via permutation-rotation
2. **Resonance Operator (R̂_ω)**: Spectral coupling for address-free communication
3. **Sweep Operator (Ŝ_τ)**: Adaptive threshold filtering with temporal scheduling
4. **Pfadinvarianz Operator (P̂_Γ)**: Path-equivalence projection ensuring determinism
5. **Weight Transfer Operator (Ŵ_γ)**: Multi-scale coherence redistribution
6. **DoubleKick Operator (D̂_α)**: Dual orthogonal impulse for stability escape

## Key Properties

- **Information-theoretic privacy** through masking involutions
- **Provable convergence** via contractive operator sequences
- **Adaptive resilience** through multi-scale weight transfer
- **Path independence** via idempotent projections
- **Equilibrium escape** through dual orthogonal impulses

## Example Usage

```rust
use omega_protocol::{OmegaNode, NodeConfig, OmegaParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a node with frequency 1.5
    let config = NodeConfig {
        omega: 1.5,
        params: OmegaParams::default(),
    };

    let mut node = OmegaNode::new(config)?;

    // Send a message
    let message = b"Hello, OMEGA Protocol!";
    node.send_message(message, 1.5).await?;

    // Receive messages
    if let Some(received) = node.receive_message().await? {
        println!("Received: {:?}", String::from_utf8_lossy(&received));
    }

    Ok(())
}
```

## References

See `OMEGA_Protocol.pdf` for complete mathematical formalization and security analysis.
*/

pub mod types;
pub mod operators;
pub mod node;
pub mod utils;

// Re-export main types
pub use types::{
    OmegaVector, OmegaParams, OmegaError, Result,
    NodeConfig, MaskingParams, ResonanceParams,
    SweepParams, PfadinvarianzParams, WeightTransferParams,
    DoubleKickParams, ScaleLevel,
};

pub use node::OmegaNode;

pub use operators::{
    OmegaOperator,
    masking::MaskingOperator,
    resonance::ResonanceOperator,
    sweep::Sweep,
    pfadinvarianz::Pfadinvarianz,
    weight_transfer::WeightTransfer,
    doublekick::DoubleKick,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_basics() {
        // Test that we can create basic types
        let params = OmegaParams::default();
        assert!(params.resonance.epsilon > 0.0);

        let config = NodeConfig::default();
        assert!(config.omega > 0.0);
    }
}
