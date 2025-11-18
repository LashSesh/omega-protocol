/// Core type definitions for the OMEGA Protocol
use ndarray::Array1;
use serde::{Deserialize, Serialize};

/// 5-dimensional vector space for OMEGA operations
pub type OmegaVector = Array1<f64>;

/// Masking parameters for information-theoretic encryption
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaskingParams {
    /// Phase rotation parameter [0, 2π)
    pub theta: f64,
    /// Permutation seed (256-bit)
    pub sigma: [u8; 32],
}

impl MaskingParams {
    /// Derive ephemeral masking parameters from frequency and epoch
    pub fn ephemeral_from_frequency(omega: f64, epoch: u64) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(omega.to_le_bytes());
        hasher.update(epoch.to_le_bytes());
        let hash = hasher.finalize();

        let mut sigma = [0u8; 32];
        sigma.copy_from_slice(&hash);

        // Derive theta from hash
        let theta_bytes = u64::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7],
        ]);
        let theta = (theta_bytes as f64 / u64::MAX as f64) * 2.0 * std::f64::consts::PI;

        Self { theta, sigma }
    }
}

/// Resonance parameters for spectral coupling
#[derive(Clone, Debug)]
pub struct ResonanceParams {
    /// Target frequency
    pub omega: f64,
    /// Resonance bandwidth
    pub epsilon: f64,
}

impl Default for ResonanceParams {
    fn default() -> Self {
        Self {
            omega: 1.0,
            epsilon: 0.1,
        }
    }
}

/// Sweep parameters for adaptive threshold filtering
#[derive(Clone, Debug)]
pub struct SweepParams {
    /// Base threshold
    pub tau0: f64,
    /// Gate width parameter
    pub beta: f64,
    /// Schedule type: "cosine" or "linear"
    pub schedule: String,
}

impl Default for SweepParams {
    fn default() -> Self {
        Self {
            tau0: 0.5,
            beta: 0.1,
            schedule: "cosine".to_string(),
        }
    }
}

/// Pfadinvarianz parameters for path-invariant projection
#[derive(Clone, Debug)]
pub struct PfadinvarianzParams {
    /// Number of permutations in group
    pub permutation_count: usize,
}

impl Default for PfadinvarianzParams {
    fn default() -> Self {
        Self {
            permutation_count: 24, // 4! for dimension 5
        }
    }
}

/// Scale levels for multi-scale weight transfer
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScaleLevel {
    Micro,
    Meso,
    Macro,
}

/// Weight transfer parameters
#[derive(Clone, Debug)]
pub struct WeightTransferParams {
    /// Transfer rate γ ∈ [0, 1]
    pub gamma: f64,
    /// Weight for each scale level
    pub levels: Vec<(ScaleLevel, f64)>,
}

impl Default for WeightTransferParams {
    fn default() -> Self {
        Self {
            gamma: 0.3,
            levels: vec![
                (ScaleLevel::Micro, 0.2),
                (ScaleLevel::Meso, 0.5),
                (ScaleLevel::Macro, 0.3),
            ],
        }
    }
}

/// DoubleKick parameters for dual orthogonal impulse
#[derive(Clone, Debug)]
pub struct DoubleKickParams {
    /// First impulse magnitude
    pub alpha1: f64,
    /// Second impulse magnitude
    pub alpha2: f64,
}

impl Default for DoubleKickParams {
    fn default() -> Self {
        Self {
            alpha1: 0.05,
            alpha2: -0.03,
        }
    }
}

/// Complete OMEGA parameters
#[derive(Clone, Debug)]
pub struct OmegaParams {
    pub masking: MaskingParams,
    pub resonance: ResonanceParams,
    pub sweep: SweepParams,
    pub pfadinvarianz: PfadinvarianzParams,
    pub weight_transfer: WeightTransferParams,
    pub doublekick: DoubleKickParams,
}

impl Default for OmegaParams {
    fn default() -> Self {
        Self {
            masking: MaskingParams {
                theta: 0.0,
                sigma: [0u8; 32],
            },
            resonance: ResonanceParams::default(),
            sweep: SweepParams::default(),
            pfadinvarianz: PfadinvarianzParams::default(),
            weight_transfer: WeightTransferParams::default(),
            doublekick: DoubleKickParams::default(),
        }
    }
}

/// Node configuration
#[derive(Clone, Debug)]
pub struct NodeConfig {
    /// Local resonance frequency
    pub omega: f64,
    /// OMEGA parameters
    pub params: OmegaParams,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            omega: 1.0,
            params: OmegaParams::default(),
        }
    }
}

/// Result type for OMEGA operations
pub type Result<T> = std::result::Result<T, OmegaError>;

/// Error types for OMEGA protocol
#[derive(Debug, thiserror::Error)]
pub enum OmegaError {
    #[error("Vectorization error: {0}")]
    VectorizationError(String),

    #[error("Masking error: {0}")]
    MaskingError(String),

    #[error("Resonance error: {0}")]
    ResonanceError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parameter error: {0}")]
    ParameterError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
