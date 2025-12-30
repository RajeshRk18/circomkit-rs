//! Circomkit configuration

use crate::error::{CircomkitError, Result};
use crate::types::{Prime, Protocol};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration for Circomkit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CircomkitConfig {
    /// Version of the configuration format
    #[serde(default = "default_version")]
    pub version: String,

    /// Protocol to use for proving
    #[serde(default)]
    pub protocol: Protocol,

    /// Prime field to use
    #[serde(default)]
    pub prime: Prime,

    /// Optimization level (0, 1, or 2)
    #[serde(default = "default_optimization")]
    pub optimization: u8,

    /// Whether to output verbose logs
    #[serde(default)]
    pub verbose: bool,

    /// Directory for circuit files
    #[serde(default = "default_dir_circuits")]
    pub dir_circuits: PathBuf,

    /// Directory for input files
    #[serde(default = "default_dir_inputs")]
    pub dir_inputs: PathBuf,

    /// Directory for build artifacts
    #[serde(default = "default_dir_build")]
    pub dir_build: PathBuf,

    /// Directory for PTAU files
    #[serde(default = "default_dir_ptau")]
    pub dir_ptau: PathBuf,

    /// Path to circuits configuration file
    #[serde(default = "default_circuits_file")]
    pub circuits: PathBuf,

    /// Include paths for circom compiler
    #[serde(default)]
    pub include: Vec<PathBuf>,

    /// Custom circom compiler path
    #[serde(default)]
    pub circom_path: Option<PathBuf>,

    /// Custom snarkjs path
    #[serde(default)]
    pub snarkjs_path: Option<PathBuf>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_optimization() -> u8 {
    1
}

fn default_dir_circuits() -> PathBuf {
    PathBuf::from("circuits")
}

fn default_dir_inputs() -> PathBuf {
    PathBuf::from("inputs")
}

fn default_dir_build() -> PathBuf {
    PathBuf::from("build")
}

fn default_dir_ptau() -> PathBuf {
    PathBuf::from("ptau")
}

fn default_circuits_file() -> PathBuf {
    PathBuf::from("circuits.json")
}

impl Default for CircomkitConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            protocol: Protocol::default(),
            prime: Prime::default(),
            optimization: default_optimization(),
            verbose: false,
            dir_circuits: default_dir_circuits(),
            dir_inputs: default_dir_inputs(),
            dir_build: default_dir_build(),
            dir_ptau: default_dir_ptau(),
            circuits: default_circuits_file(),
            include: Vec::new(),
            circom_path: None,
            snarkjs_path: None,
        }
    }
}

impl CircomkitConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from the default file (circomkit.json)
    pub fn from_default_file() -> Result<Self> {
        let path = PathBuf::from("circomkit.json");
        if path.exists() {
            Self::from_file(path)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to a file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Set the protocol
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Set the prime field
    pub fn with_prime(mut self, prime: Prime) -> Self {
        self.prime = prime;
        self
    }

    /// Set the optimization level
    pub fn with_optimization(mut self, level: u8) -> Self {
        self.optimization = level.min(2);
        self
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set the circuits directory
    pub fn with_circuits_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir_circuits = dir.into();
        self
    }

    /// Set the inputs directory
    pub fn with_inputs_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir_inputs = dir.into();
        self
    }

    /// Set the build directory
    pub fn with_build_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir_build = dir.into();
        self
    }

    /// Set the PTAU directory
    pub fn with_ptau_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir_ptau = dir.into();
        self
    }

    /// Add an include path
    pub fn with_include(mut self, path: impl Into<PathBuf>) -> Self {
        self.include.push(path.into());
        self
    }

    /// Set custom circom compiler path
    pub fn with_circom_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.circom_path = Some(path.into());
        self
    }

    /// Set custom snarkjs path
    pub fn with_snarkjs_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.snarkjs_path = Some(path.into());
        self
    }

    /// Get the path to a circuit file
    pub fn circuit_path(&self, file: &str) -> PathBuf {
        self.dir_circuits.join(file)
    }

    /// Get the path to an input file
    pub fn input_path(&self, circuit: &str, input: &str) -> PathBuf {
        self.dir_inputs.join(circuit).join(format!("{}.json", input))
    }

    /// Get the build directory for a circuit
    pub fn build_path(&self, circuit: &str) -> PathBuf {
        self.dir_build.join(circuit)
    }

    /// Get the path to a PTAU file
    pub fn ptau_path(&self, filename: &str) -> PathBuf {
        self.dir_ptau.join(filename)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.optimization > 2 {
            return Err(CircomkitError::InvalidConfig(
                "Optimization level must be 0, 1, or 2".to_string(),
            ));
        }
        Ok(())
    }

    /// Get the circom compiler command
    pub fn circom_command(&self) -> String {
        self.circom_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "circom".to_string())
    }

    /// Get the snarkjs command
    pub fn snarkjs_command(&self) -> String {
        self.snarkjs_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "snarkjs".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CircomkitConfig::default();
        assert_eq!(config.protocol, Protocol::Groth16);
        assert_eq!(config.prime, Prime::Bn128);
        assert_eq!(config.optimization, 1);
        assert!(!config.verbose);
    }

    #[test]
    fn test_config_builder() {
        let config = CircomkitConfig::new()
            .with_protocol(Protocol::Plonk)
            .with_optimization(2)
            .with_verbose(true);

        assert_eq!(config.protocol, Protocol::Plonk);
        assert_eq!(config.optimization, 2);
        assert!(config.verbose);
    }

    #[test]
    fn test_config_paths() {
        let config = CircomkitConfig::new();
        
        assert_eq!(
            config.circuit_path("multiplier.circom"),
            PathBuf::from("circuits/multiplier.circom")
        );
        
        assert_eq!(
            config.input_path("multiplier", "default"),
            PathBuf::from("inputs/multiplier/default.json")
        );
        
        assert_eq!(
            config.build_path("multiplier"),
            PathBuf::from("build/multiplier")
        );
    }
}
