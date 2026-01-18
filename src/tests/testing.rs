//! Circuit testing utilities

use crate::core::{Circomkit, CircomkitConfig};
use crate::testers::WitnessTester;
use crate::types::{CircuitConfig, CircuitSignals, SignalValue};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Directory for test circuits
pub const TEST_CIRCUITS_DIR: &str = "test_circuits";
/// Directory for test build artifacts
pub const TEST_BUILD_DIR: &str = "test_build";

/// Circuit tester that uses the circomkit library
pub struct CircuitTester {
    /// Circomkit instance
    circomkit: Circomkit,
    /// Directory for circuit source files
    pub circuits_dir: PathBuf,
}

impl Default for CircuitTester {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitTester {
    /// Create a new circuit tester with default directories
    pub fn new() -> Self {
        fs::create_dir_all(TEST_CIRCUITS_DIR).ok();
        fs::create_dir_all(TEST_BUILD_DIR).ok();

        let config = CircomkitConfig::new()
            .with_circuits_dir(TEST_CIRCUITS_DIR)
            .with_build_dir(TEST_BUILD_DIR)
            .with_optimization(2); // Opt level 2

        let circomkit = Circomkit::new(config).expect("Failed to create Circomkit");

        Self {
            circomkit,
            circuits_dir: PathBuf::from(TEST_CIRCUITS_DIR),
        }
    }

    /// Create a circuit tester with custom directories
    pub fn with_dirs(circuits_dir: &str, build_dir: &str) -> Self {
        fs::create_dir_all(circuits_dir).ok();
        fs::create_dir_all(build_dir).ok();

        let config = CircomkitConfig::new()
            .with_circuits_dir(circuits_dir)
            .with_build_dir(build_dir)
            .with_optimization(1);

        let circomkit = Circomkit::new(config).expect("Failed to create Circomkit");

        Self {
            circomkit,
            circuits_dir: PathBuf::from(circuits_dir),
        }
    }

    /// Write a circuit file to the circuits directory
    pub fn write_circuit(&self, name: &str, content: &str) -> PathBuf {
        let path = self.circuits_dir.join(format!("{}.circom", name));
        fs::write(&path, content).expect("Failed to write circuit");
        path
    }

    /// Create a CircuitConfig for a circuit
    pub fn circuit_config(&self, name: &str) -> CircuitConfig {
        CircuitConfig::new(name)
            .with_file(&format!("{}.circom", name))
            .with_template("main")
    }

    /// Compile and test a circuit with given inputs (expects success)
    pub fn test_circuit(
        &self,
        name: &str,
        code: &str,
        params: Vec<i64>,
        inputs: HashMap<String, Vec<String>>,
    ) -> std::result::Result<(), String> {
        // Write the circuit code
        self.write_circuit(name, code);

        // Create circuit config pointing to the file directly
        let circuit = CircuitConfig::new(name)
            .with_file(&format!("{}.circom", name))
            .with_params(params);

        // Use tokio runtime for async operations
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            // Compile
            self.circomkit
                .compile(&circuit)
                .await
                .map_err(|e| format!("Compilation failed: {}", e))?;

            // Convert inputs to CircuitSignals
            let signals = convert_inputs(&inputs);

            // Generate witness
            self.circomkit
                .generate_witness(&circuit, &signals)
                .await
                .map_err(|e| format!("Witness generation failed: {}", e))?;

            Ok(())
        })
    }

    /// Test that a circuit FAILS with given inputs (expects constraint failure)
    pub fn test_circuit_fails(
        &self,
        name: &str,
        code: &str,
        params: Vec<i64>,
        inputs: HashMap<String, Vec<String>>,
    ) -> std::result::Result<(), String> {
        // Write the circuit code
        self.write_circuit(name, code);

        let circuit = CircuitConfig::new(name)
            .with_file(&format!("{}.circom", name))
            .with_params(params);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            // Compile
            self.circomkit
                .compile(&circuit)
                .await
                .map_err(|e| format!("Compilation failed: {}", e))?;

            // Convert inputs
            let signals = convert_inputs(&inputs);

            // Generate witness - expect this to fail
            match self.circomkit.generate_witness(&circuit, &signals).await {
                Ok(_) => Err("Expected circuit to fail but it passed".to_string()),
                Err(_) => Ok(()), // Expected failure
            }
        })
    }

    /// Test circuit with expected outputs
    pub fn test_circuit_output(
        &self,
        name: &str,
        code: &str,
        params: Vec<i64>,
        inputs: HashMap<String, Vec<String>>,
        expected_outputs: HashMap<String, Vec<String>>,
    ) -> std::result::Result<(), String> {
        self.write_circuit(name, code);

        let circuit = CircuitConfig::new(name)
            .with_file(&format!("{}.circom", name))
            .with_params(params);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            // Create WitnessTester
            let config = CircomkitConfig::new()
                .with_circuits_dir(&self.circuits_dir)
                .with_build_dir(TEST_BUILD_DIR)
                .with_optimization(1);

            let mut tester = WitnessTester::from_circuit_config_with_settings(circuit, config)
                .await
                .map_err(|e| format!("Failed to create tester: {}", e))?;

            let input_signals = convert_inputs(&inputs);
            let expected_signals = convert_inputs(&expected_outputs);

            let result = tester
                .expect_output(input_signals, expected_signals)
                .await
                .map_err(|e| format!("Test failed: {}", e))?;

            if result.passed {
                Ok(())
            } else {
                Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
            }
        })
    }

    /// Get the underlying Circomkit instance
    pub fn circomkit(&self) -> &Circomkit {
        &self.circomkit
    }
}

/// Convert HashMap<String, Vec<String>> to CircuitSignals
fn convert_inputs(inputs: &HashMap<String, Vec<String>>) -> CircuitSignals {
    inputs
        .iter()
        .map(|(k, v)| {
            let value = if v.len() == 1 {
                SignalValue::Single(v[0].clone())
            } else {
                SignalValue::Array(v.iter().map(|s| SignalValue::Single(s.clone())).collect())
            };
            (k.clone(), value)
        })
        .collect()
}

/// Helper function to create inputs map from slice of pairs
pub fn inputs(pairs: &[(&str, Vec<&str>)]) -> HashMap<String, Vec<String>> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inputs_helper() {
        let inp = inputs(&[("a", vec!["1", "2"]), ("b", vec!["3"])]);

        assert_eq!(
            inp.get("a").unwrap(),
            &vec!["1".to_string(), "2".to_string()]
        );
        assert_eq!(inp.get("b").unwrap(), &vec!["3".to_string()]);
    }

    #[test]
    fn test_convert_inputs() {
        let inp = inputs(&[("x", vec!["42"])]);
        let signals = convert_inputs(&inp);

        assert!(matches!(signals.get("x").unwrap(), SignalValue::Single(s) if s == "42"));
    }

    #[test]
    fn test_convert_inputs_array() {
        let inp = inputs(&[("arr", vec!["1", "2", "3"])]);
        let signals = convert_inputs(&inp);

        assert!(matches!(signals.get("arr").unwrap(), SignalValue::Array(a) if a.len() == 3));
    }
}
