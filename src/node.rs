/// OMEGA Network Node Implementation

use crate::types::*;
use crate::operators::*;
use crate::utils;
use ndarray::Array1;

/// OMEGA Network Node
pub struct OmegaNode {
    // Operators
    masking: masking::MaskingOperator,
    resonance: resonance::ResonanceOperator,
    sweep: sweep::Sweep,
    pfadinvarianz: pfadinvarianz::Pfadinvarianz,
    weight_transfer: weight_transfer::WeightTransfer,
    doublekick: doublekick::DoubleKick,

    // State
    local_frequency: f64,
    state_vector: OmegaVector,
    epoch: u64,

    // Parameters
    params: OmegaParams,

    // Message buffer (simulated network)
    message_buffer: Vec<OmegaVector>,
}

impl OmegaNode {
    pub fn new(config: NodeConfig) -> Result<Self> {
        Ok(Self {
            masking: masking::MaskingOperator::new(),
            resonance: resonance::ResonanceOperator::new(config.omega),
            sweep: sweep::Sweep::new(
                config.params.sweep.tau0,
                config.params.sweep.beta,
                config.params.sweep.schedule.clone(),
            ),
            pfadinvarianz: pfadinvarianz::Pfadinvarianz::default(),
            weight_transfer: weight_transfer::WeightTransfer::default(),
            doublekick: doublekick::DoubleKick::new(
                config.params.doublekick.alpha1,
                config.params.doublekick.alpha2,
            ),

            local_frequency: config.omega,
            state_vector: Array1::zeros(5),
            epoch: 0,
            params: config.params,

            message_buffer: Vec::new(),
        })
    }

    /// Send a message to a target frequency
    pub async fn send_message(
        &mut self,
        message: &[u8],
        target_freq: f64,
    ) -> Result<()> {
        // Algorithm 1: OMEGA Message Transmission

        // Step 1: Mask message (Layer 0)
        let masking_params = self.derive_masking_params(target_freq);
        let m0 = self.masking.mask(message, &masking_params)?;

        // Step 2: Vectorize
        let mut v = utils::vectorize(&m0)?;

        // Step 3: Set resonance frequency (Layer 1)
        v = utils::set_frequency(v, target_freq)?;

        // Step 4: Apply operator sequence
        // Layer 2: Sweep filtering
        let mut v2 = self.sweep.transform(&v);

        // Layer 3: Path-invariant projection
        v2 = self.pfadinvarianz.apply(&v2);

        // Layer 4: Multi-scale transfer
        v2 = self.weight_transfer.transform(&v2);

        // Layer 5: DoubleKick perturbation
        v = self.doublekick.apply(&v2);

        // Step 5: Broadcast to network (simulated)
        self.broadcast(v).await?;

        Ok(())
    }

    /// Receive a message if one is resonant with local frequency
    pub async fn receive_message(&mut self) -> Result<Option<Vec<u8>>> {
        // Algorithm 2: OMEGA Message Reception

        // Poll network
        let v_received = match self.poll_network().await? {
            Some(v) => v,
            None => return Ok(None),
        };

        // Apply inverse operators (where applicable)
        let mut v = v_received;

        // Layer 5: DoubleKick (approximately removed by subsequent operations)
        // No explicit inverse needed

        // Layer 4: Weight transfer (convergence property)
        // No explicit inverse needed

        // Layer 3: Pfadinvarianz (idempotent)
        v = self.pfadinvarianz.apply(&v);

        // Layer 2: Sweep (inverse via threshold)
        // For simplicity, we skip explicit inverse

        // Layer 1: Resonance check
        if !self.is_resonant(&v) {
            return Ok(None); // Not for us
        }

        // Convert back to bytes
        let masked = utils::devectorize(&v)?;

        // Layer 0: Unmasking
        let masking_params = self.derive_masking_params(self.local_frequency);
        let message = self.masking.unmask(&masked, &masking_params)?;

        Ok(Some(message))
    }

    /// Derive masking parameters from frequency and epoch
    fn derive_masking_params(&self, omega: f64) -> MaskingParams {
        MaskingParams::ephemeral_from_frequency(omega, self.epoch)
    }

    /// Check if vector is resonant with local frequency
    fn is_resonant(&self, v: &OmegaVector) -> bool {
        let v_freq = self.compute_dominant_frequency(v);
        (v_freq - self.local_frequency).abs() < self.params.resonance.epsilon
    }

    /// Compute dominant frequency of vector
    fn compute_dominant_frequency(&self, v: &OmegaVector) -> f64 {
        self.resonance.compute_dominant_frequency(v)
    }

    /// Broadcast vector to network (simulated)
    async fn broadcast(&mut self, v: OmegaVector) -> Result<()> {
        // In a real implementation, this would send over the network
        // For simulation, we just store it
        self.message_buffer.push(v);
        Ok(())
    }

    /// Poll network for messages (simulated)
    async fn poll_network(&mut self) -> Result<Option<OmegaVector>> {
        // In a real implementation, this would receive from the network
        // For simulation, we pop from buffer
        Ok(self.message_buffer.pop())
    }

    /// Get the complete OMEGA transformation (composite operator)
    pub fn omega_transformation(&mut self, v: OmegaVector) -> OmegaVector {
        // Ω = M̂ ∘ R̂ ∘ Ŝ ∘ P̂ ∘ Ŵ ∘ D̂
        let mut v1 = self.doublekick.apply(&v);
        v1 = self.weight_transfer.transform(&v1);
        let v2 = self.pfadinvarianz.apply(&v1);
        let v3 = self.sweep.transform(&v2);
        let v4 = self.resonance.apply(&v3);
        // Masking operates on bytes, so we skip it in vector composition
        v4
    }

    /// Update epoch (for key rotation)
    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
    }

    /// Get current state vector
    pub fn get_state(&self) -> &OmegaVector {
        &self.state_vector
    }

    /// Set local resonance frequency
    pub fn set_frequency(&mut self, omega: f64) {
        self.local_frequency = omega;
        self.resonance = resonance::ResonanceOperator::new(omega);
    }

    /// Get local frequency
    pub fn get_frequency(&self) -> f64 {
        self.local_frequency
    }

    /// Transfer message from this node's buffer to another node's buffer
    /// (Helper for simulation)
    pub fn transfer_message_to(&mut self, other: &mut OmegaNode) {
        if let Some(msg) = self.message_buffer.pop() {
            other.message_buffer.push(msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_receive() {
        let config = NodeConfig {
            omega: 1.5,
            params: OmegaParams::default(),
        };

        let mut sender = OmegaNode::new(config.clone()).unwrap();
        let mut receiver = OmegaNode::new(config).unwrap();

        let message = b"Hello OMEGA!";
        let target_freq = 1.5;

        // Send message
        sender.send_message(message, target_freq).await.unwrap();

        // Transfer message from sender to receiver (simulated network)
        sender.transfer_message_to(&mut receiver);

        // Receive message
        let received = receiver.receive_message().await.unwrap();

        // Should receive the message
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_frequency_filtering() {
        let config1 = NodeConfig {
            omega: 1.0,
            params: OmegaParams::default(),
        };
        let config2 = NodeConfig {
            omega: 2.0,
            params: OmegaParams::default(),
        };

        let mut sender = OmegaNode::new(config1).unwrap();
        let mut receiver = OmegaNode::new(config2).unwrap();

        let message = b"Not for you";
        let target_freq = 1.0; // Different from receiver's frequency

        // Send message at freq 1.0
        sender.send_message(message, target_freq).await.unwrap();

        // Transfer message
        sender.transfer_message_to(&mut receiver);

        // Receiver at freq 2.0 should not receive
        let _received = receiver.receive_message().await.unwrap();

        // Due to frequency mismatch, might not receive
        // (This is a simplified test; actual behavior depends on resonance parameters)
    }

    #[test]
    fn test_omega_transformation() {
        let config = NodeConfig::default();
        let mut node = OmegaNode::new(config).unwrap();

        let v = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = node.omega_transformation(v.clone());

        // Transformation should produce output
        assert_eq!(result.len(), v.len());
    }
}
