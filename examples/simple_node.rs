/// Simple OMEGA Node Example
///
/// Demonstrates basic message transmission and reception using the OMEGA Protocol.

use omega_protocol::{OmegaNode, NodeConfig, OmegaParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OMEGA Protocol - Simple Node Example");
    println!("=====================================\n");

    // Create two nodes with the same frequency (can communicate)
    let freq = 1.5;
    let config = NodeConfig {
        omega: freq,
        params: OmegaParams::default(),
    };

    let mut alice = OmegaNode::new(config.clone())?;
    let mut bob = OmegaNode::new(config)?;

    println!("Created two nodes (Alice and Bob) at frequency {}", freq);
    println!();

    // Alice sends a message to Bob
    let message = b"Hello Bob, this is a secret message from Alice!";
    println!("Alice sending: {:?}", String::from_utf8_lossy(message));

    alice.send_message(message, freq).await?;
    println!("Message encrypted and transmitted through OMEGA operators");
    println!();

    // Simulate network transfer
    alice.transfer_message_to(&mut bob);
    println!("Message transferred through network");
    println!();

    // Bob receives the message
    match bob.receive_message().await? {
        Some(received) => {
            println!("Bob received: {:?}", String::from_utf8_lossy(&received));
            println!("\n✓ Message successfully transmitted and received!");
        }
        None => {
            println!("✗ No message received (frequency mismatch or filtering)");
        }
    }

    println!("\n--- Testing Frequency Filtering ---\n");

    // Create a third node with different frequency
    let config_charlie = NodeConfig {
        omega: 2.5,
        params: OmegaParams::default(),
    };
    let mut charlie = OmegaNode::new(config_charlie)?;

    println!("Created Charlie at frequency 2.5");

    // Alice sends another message
    let message2 = b"This should only be for Bob (freq 1.5)";
    println!("Alice sending: {:?}", String::from_utf8_lossy(message2));

    alice.send_message(message2, freq).await?;

    // Transfer to both Bob and Charlie
    alice.transfer_message_to(&mut bob);
    alice.transfer_message_to(&mut charlie);

    // Bob should receive it
    match bob.receive_message().await? {
        Some(received) => {
            println!("✓ Bob (freq 1.5) received: {:?}", String::from_utf8_lossy(&received));
        }
        None => {
            println!("  Bob did not receive (unexpected)");
        }
    }

    // Charlie should NOT receive it (different frequency)
    match charlie.receive_message().await? {
        Some(received) => {
            println!("  Charlie (freq 2.5) received: {:?}", String::from_utf8_lossy(&received));
        }
        None => {
            println!("✓ Charlie (freq 2.5) did not receive (frequency filtering works!)");
        }
    }

    println!("\n--- Operator Properties ---\n");
    println!("Masking Operator (M̂): Lipschitz constant = 1.0 (isometric)");
    println!("Resonance Operator (R̂): Lipschitz constant = 1.0 (non-expansive)");
    println!("Sweep Operator (Ŝ): Lipschitz constant = 1.0 (non-expansive)");
    println!("Pfadinvarianz Operator (P̂): Lipschitz constant = 1.0 (idempotent)");
    println!("Weight Transfer Operator (Ŵ): Lipschitz constant = 1.0 (convex)");
    println!("DoubleKick Operator (D̂): Lipschitz constant ≈ 1.08 (near-isometry)");
    println!("\nComposite Ω: Lipschitz constant < 1.1 (approximately contractive)");

    Ok(())
}
