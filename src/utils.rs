/// Utility functions for OMEGA Protocol

use crate::types::*;
use ndarray::Array1;

/// Convert bytes to 5D vector representation
pub fn vectorize(data: &[u8]) -> Result<OmegaVector> {
    if data.is_empty() {
        return Err(OmegaError::VectorizationError(
            "Cannot vectorize empty data".to_string()
        ));
    }

    // Pad or truncate to multiple of 5
    let target_len = ((data.len() + 4) / 5) * 5;
    let mut padded = data.to_vec();
    padded.resize(target_len, 0);

    // Take first 5 bytes and normalize to [-1, 1]
    let mut vec = Array1::zeros(5);
    for i in 0..5.min(padded.len()) {
        vec[i] = (padded[i] as f64 - 128.0) / 128.0;
    }

    Ok(vec)
}

/// Convert 5D vector back to bytes
pub fn devectorize(v: &OmegaVector) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(v.len());

    for &val in v.iter() {
        // Denormalize from [-1, 1] to [0, 255]
        let byte_val = ((val * 128.0) + 128.0)
            .max(0.0)
            .min(255.0) as u8;
        bytes.push(byte_val);
    }

    Ok(bytes)
}

/// Set frequency component in vector (simplified version)
/// In a full implementation, this would use FFT to inject frequency
pub fn set_frequency(v: OmegaVector, omega: f64) -> Result<OmegaVector> {
    let len = v.len();
    if len == 0 {
        return Ok(v);
    }

    // Create a sinusoidal component at the target frequency
    let mut freq_component = Array1::zeros(len);
    for i in 0..len {
        freq_component[i] = (omega * i as f64).sin() * 0.1;
    }

    // Add frequency component to vector
    Ok(v + freq_component)
}

/// Compute dominant frequency using simple autocorrelation
/// (Simplified version; full implementation uses FFT)
pub fn compute_dominant_frequency(v: &OmegaVector) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }

    // Simple method: look for sign changes (zero crossings)
    let mut zero_crossings = 0;
    for i in 1..v.len() {
        if (v[i] >= 0.0 && v[i-1] < 0.0) || (v[i] < 0.0 && v[i-1] >= 0.0) {
            zero_crossings += 1;
        }
    }

    // Frequency is proportional to zero crossings
    let freq = (zero_crossings as f64 / v.len() as f64) * std::f64::consts::PI;
    freq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vectorize_devectorize() {
        let data = b"Hello";
        let vec = vectorize(data).unwrap();
        let recovered = devectorize(&vec).unwrap();

        // Should have same length after round-trip
        assert_eq!(recovered.len(), 5);
    }

    #[test]
    fn test_set_frequency() {
        let v = Array1::from_vec(vec![0.5, 0.3, 0.1, 0.2, 0.4]);
        let omega = 1.5;

        let result = set_frequency(v.clone(), omega).unwrap();

        // Result should be different from input
        assert_ne!(result, v);
        assert_eq!(result.len(), v.len());
    }

    #[test]
    fn test_compute_dominant_frequency() {
        let v = Array1::from_vec(vec![1.0, -1.0, 1.0, -1.0, 1.0]);
        let freq = compute_dominant_frequency(&v);

        // Should detect high frequency due to alternating pattern
        assert!(freq > 0.0);
    }
}
