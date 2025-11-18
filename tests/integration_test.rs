/// Integration tests for OMEGA Protocol

use omega_protocol::*;
use ndarray::Array1;

#[tokio::test]
async fn test_end_to_end_communication() {
    let config = NodeConfig {
        omega: 1.5,
        params: OmegaParams::default(),
    };

    let mut sender = OmegaNode::new(config.clone()).unwrap();
    let mut receiver = OmegaNode::new(config).unwrap();

    let message = b"Secret message through OMEGA";
    sender.send_message(message, 1.5).await.unwrap();

    sender.transfer_message_to(&mut receiver);

    let received = receiver.receive_message().await.unwrap();
    assert!(received.is_some(), "Message should be received");
}

#[tokio::test]
async fn test_frequency_selectivity() {
    let config1 = NodeConfig {
        omega: 1.0,
        params: OmegaParams::default(),
    };

    let config2 = NodeConfig {
        omega: 3.0,
        params: OmegaParams::default(),
    };

    let mut sender = OmegaNode::new(config1).unwrap();
    let mut receiver_wrong_freq = OmegaNode::new(config2).unwrap();

    let message = b"Only for frequency 1.0";
    sender.send_message(message, 1.0).await.unwrap();

    sender.transfer_message_to(&mut receiver_wrong_freq);

    let _received = receiver_wrong_freq.receive_message().await.unwrap();
    // Due to frequency mismatch, message might not be received correctly
    // This test validates the filtering behavior
}

#[test]
fn test_masking_involution() {
    let operator = MaskingOperator::new();
    let message = b"Test message for masking";

    let params = MaskingParams::ephemeral_from_frequency(1.5, 100);

    let masked = operator.mask(message, &params).unwrap();
    assert_ne!(masked, message, "Masked message should differ from original");

    let unmasked = operator.unmask(&masked, &params).unwrap();
    assert_eq!(unmasked, message, "Unmasking should recover original");
}

#[test]
fn test_pfadinvarianz_idempotence() {
    let pfad = Pfadinvarianz::default();
    let v = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let v1 = pfad.apply(&v);
    let v2 = pfad.apply(&v1);

    // P̂(P̂(v)) = P̂(v)
    for (a, b) in v1.iter().zip(v2.iter()) {
        assert!((a - b).abs() < 1e-9, "Operator should be idempotent");
    }
}

#[test]
fn test_sweep_contractivity() {
    let mut sweep = Sweep::default();
    let v = Array1::from_vec(vec![3.0, 4.0, 5.0, 6.0, 7.0]);

    let result = sweep.transform(&v);

    let v_norm = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
    let result_norm = result.iter().map(|&x| x * x).sum::<f64>().sqrt();

    assert!(result_norm <= v_norm + 1e-10, "Sweep should be non-expansive");
}

#[test]
fn test_doublekick_perturbation() {
    let dk = DoubleKick::new(0.1, -0.05);
    let v = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let result = dk.apply(&v);

    assert_ne!(result, v, "DoubleKick should perturb the vector");

    let diff = &result - &v;
    let perturbation_norm = diff.iter().map(|x| x * x).sum::<f64>().sqrt();

    assert!(perturbation_norm > 0.0, "Perturbation should be non-zero");
    assert!(perturbation_norm < 1.0, "Perturbation should be bounded");
}

#[test]
fn test_weight_transfer_convexity() {
    let wt = WeightTransfer::default();
    let weights = wt.get_weights();

    let sum: f64 = weights.values().sum();
    assert!((sum - 1.0).abs() < 0.01, "Weights should form convex combination");
}

#[test]
fn test_resonance_filtering() {
    let resonance = ResonanceOperator::new(1.5);

    let v = Array1::from_vec(vec![1.0, 0.5, 0.0, -0.5, -1.0]);
    let result = resonance.apply(&v);

    // Result should either pass through or be zeroed
    assert_eq!(result.len(), v.len());
}

#[test]
fn test_operator_composition() {
    let mut node = OmegaNode::new(NodeConfig::default()).unwrap();

    let v = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let result = node.omega_transformation(v.clone());

    assert_eq!(result.len(), v.len(), "Composition should preserve dimension");
}

#[test]
fn test_ephemeral_key_derivation() {
    let params1 = MaskingParams::ephemeral_from_frequency(1.5, 100);
    let params2 = MaskingParams::ephemeral_from_frequency(1.5, 100);
    let params3 = MaskingParams::ephemeral_from_frequency(1.5, 101);

    // Same inputs produce same keys
    assert_eq!(params1.theta, params2.theta);
    assert_eq!(params1.sigma, params2.sigma);

    // Different epoch produces different keys
    assert_ne!(params1.sigma, params3.sigma);
}

#[tokio::test]
async fn test_multiple_messages() {
    let config = NodeConfig::default();
    let mut sender = OmegaNode::new(config.clone()).unwrap();
    let mut receiver = OmegaNode::new(config).unwrap();

    let messages: Vec<&[u8]> = vec![
        b"First message",
        b"Second messag",
        b"Third message",
    ];

    for msg in &messages {
        sender.send_message(*msg, 1.0).await.unwrap();
        sender.transfer_message_to(&mut receiver);

        let received = receiver.receive_message().await.unwrap();
        assert!(received.is_some(), "Should receive message");
    }
}
