//! Type definitions for Circomkit-rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported proving protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// Groth16 proving system
    #[default]
    Groth16,
    /// PLONK proving system
    Plonk,
    /// FFLONK proving system
    Fflonk,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Groth16 => write!(f, "groth16"),
            Protocol::Plonk => write!(f, "plonk"),
            Protocol::Fflonk => write!(f, "fflonk"),
        }
    }
}

/// Supported prime fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Prime {
    /// BN128 curve (default)
    #[default]
    Bn128,
    /// BLS12-381 curve
    Bls12381,
    /// Goldilocks field
    Goldilocks,
}

impl std::fmt::Display for Prime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Prime::Bn128 => write!(f, "bn128"),
            Prime::Bls12381 => write!(f, "bls12381"),
            Prime::Goldilocks => write!(f, "goldilocks"),
        }
    }
}

/// Signal value type - can be a single value or an array
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalValue {
    /// Single numeric value (as string for big integers)
    Single(String),
    /// Single numeric value as number
    Number(i64),
    /// Array of values
    Array(Vec<SignalValue>),
}

impl SignalValue {
    /// Create a single value from a number
    pub fn single<T: ToString>(value: T) -> Self {
        Self::Single(value.to_string())
    }

    /// Create an array of values
    pub fn array<I, T>(values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: ToString,
    {
        Self::Array(values.into_iter().map(|v| Self::single(v)).collect())
    }

    /// Convert to a string representation
    pub fn as_string(&self) -> String {
        match self {
            SignalValue::Single(s) => s.clone(),
            SignalValue::Number(n) => n.to_string(),
            SignalValue::Array(arr) => {
                let values: Vec<String> = arr.iter().map(|v| v.as_string()).collect();
                format!("[{}]", values.join(", "))
            }
        }
    }
}

impl From<i64> for SignalValue {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<&str> for SignalValue {
    fn from(value: &str) -> Self {
        Self::Single(value.to_string())
    }
}

impl From<String> for SignalValue {
    fn from(value: String) -> Self {
        Self::Single(value)
    }
}

impl<T: Into<SignalValue>> From<Vec<T>> for SignalValue {
    fn from(values: Vec<T>) -> Self {
        Self::Array(values.into_iter().map(Into::into).collect())
    }
}

/// Circuit input/output signals
pub type CircuitSignals = HashMap<String, SignalValue>;

/// Configuration for a circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitConfig {
    /// Name of the circuit instance
    pub name: String,
    /// Path to the circuit file (relative to circuits directory)
    pub file: String,
    /// Absolute path to circuit file (if set, takes precedence over `file`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_file: Option<PathBuf>,
    /// Template name within the circuit file
    pub template: String,
    /// Template parameters
    #[serde(default)]
    pub params: Vec<i64>,
    /// Public signals
    #[serde(default)]
    pub public: Vec<String>,
}

impl CircuitConfig {
    /// Create a new circuit configuration
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            file: format!("{}.circom", name),
            absolute_file: None,
            template: name,
            params: Vec::new(),
            public: Vec::new(),
        }
    }

    /// Set the circuit file path (relative to circuits directory)
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = file.into();
        self
    }

    /// Set an absolute path to the circuit file
    /// When set, this takes precedence over the relative `file` path
    pub fn with_absolute_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.absolute_file = Some(path.into());
        self
    }

    /// Check if this config uses an absolute file path
    pub fn has_absolute_file(&self) -> bool {
        self.absolute_file.is_some()
    }

    /// Get the absolute file path if set
    pub fn get_absolute_file(&self) -> Option<&PathBuf> {
        self.absolute_file.as_ref()
    }

    /// Set the template name
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template = template.into();
        self
    }

    /// Set template parameters
    pub fn with_params(mut self, params: Vec<i64>) -> Self {
        self.params = params;
        self
    }

    /// Set public signals
    pub fn with_public(mut self, public: Vec<String>) -> Self {
        self.public = public;
        self
    }

    /// Add a public signal
    pub fn add_public(mut self, signal: impl Into<String>) -> Self {
        self.public.push(signal.into());
        self
    }
}

/// Zero-knowledge proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// Protocol used to generate the proof
    pub protocol: Protocol,
    /// Proof data (protocol-specific structure)
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Verification key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKey {
    /// Protocol this key is for
    pub protocol: Protocol,
    /// Key data (protocol-specific structure)
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Public signals from a proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicSignals(pub Vec<String>);

impl PublicSignals {
    /// Create new public signals
    pub fn new(signals: Vec<String>) -> Self {
        Self(signals)
    }

    /// Get the signals as a slice
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

/// Witness data
#[derive(Debug, Clone)]
pub struct Witness {
    /// Path to the witness file
    pub path: PathBuf,
    /// Number of signals in the witness
    pub num_signals: usize,
}

/// Build artifacts for a circuit
#[derive(Debug, Clone)]
pub struct CircuitArtifacts {
    /// Path to the R1CS file
    pub r1cs: PathBuf,
    /// Path to the WASM file
    pub wasm: PathBuf,
    /// Path to the symbol file
    pub sym: PathBuf,
    /// Path to the proving key (if generated)
    pub pkey: Option<PathBuf>,
    /// Path to the verification key (if generated)
    pub vkey: Option<PathBuf>,
}

/// Circuit information from compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInfo {
    /// Number of constraints
    pub constraints: usize,
    /// Number of private inputs
    pub private_inputs: usize,
    /// Number of public inputs
    pub public_inputs: usize,
    /// Number of public outputs
    pub public_outputs: usize,
    /// Number of labels
    pub labels: usize,
}

/// Result of witness testing
#[derive(Debug, Clone)]
pub struct WitnessTestResult {
    /// Whether the test passed
    pub passed: bool,
    /// Actual output signals
    pub outputs: CircuitSignals,
    /// Expected output signals (if provided)
    pub expected: Option<CircuitSignals>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Result of proof testing
#[derive(Debug, Clone)]
pub struct ProofTestResult {
    /// Whether the proof was valid
    pub valid: bool,
    /// The generated proof
    pub proof: Option<Proof>,
    /// Public signals
    pub public_signals: Option<PublicSignals>,
    /// Error message if failed
    pub error: Option<String>,
}
