//! Witness testing utilities

use crate::core::{Circomkit, CircomkitConfig};
use crate::error::{CircomkitError, Result};
use crate::types::{CircuitConfig, CircuitSignals, SignalValue, WitnessTestResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Tester for circuit witnesses
pub struct WitnessTester {
    circomkit: Circomkit,
    circuit: CircuitConfig,
    compiled: bool,
}

impl WitnessTester {
    /// Create a new witness tester from a circuit template file
    ///
    /// # Arguments
    /// * `test_name` - Name for this test instance (used for build artifacts)
    /// * `file_path` - Path to the circuit file (can be absolute or relative)
    /// * `template` - Template name to instantiate
    /// * `params` - Template parameters
    /// * `public` - Public signal names
    ///
    /// # Example
    /// ```rust,ignore
    /// let tester = WitnessTester::new(
    ///     "my_test",
    ///     "./circuits/multiplier.circom",
    ///     "Multiplier",
    ///     vec![2],
    ///     vec![],
    /// ).await?;
    /// ```
    pub async fn new(
        test_name: impl Into<String>,
        file_path: impl Into<PathBuf>,
        template: impl Into<String>,
        params: Vec<i64>,
        public: Vec<String>,
    ) -> Result<Self> {
        let test_name = test_name.into();
        let file_path = file_path.into();
        let template = template.into();

        // Resolve to absolute path
        let abs_path = if file_path.is_absolute() {
            file_path
        } else {
            std::env::current_dir()
                .map_err(CircomkitError::Io)?
                .join(&file_path)
        };

        // Verify the file exists
        if !abs_path.exists() {
            return Err(CircomkitError::CircuitNotFound(abs_path));
        }

        let config = CircomkitConfig::from_default_file()?;

        let circuit = CircuitConfig::new(&test_name)
            .with_absolute_file(abs_path)
            .with_template(template)
            .with_params(params)
            .with_public(public);

        let circomkit = Circomkit::new(config)?;

        Ok(Self {
            circomkit,
            circuit,
            compiled: false,
        })
    }

    /// Create a new witness tester with custom configuration
    pub async fn with_config(
        test_name: impl Into<String>,
        file_path: impl Into<PathBuf>,
        template: impl Into<String>,
        params: Vec<i64>,
        public: Vec<String>,
        config: CircomkitConfig,
    ) -> Result<Self> {
        let test_name = test_name.into();
        let file_path = file_path.into();
        let template = template.into();

        // Resolve to absolute path
        let abs_path = if file_path.is_absolute() {
            file_path
        } else {
            std::env::current_dir()
                .map_err(CircomkitError::Io)?
                .join(&file_path)
        };

        if !abs_path.exists() {
            return Err(CircomkitError::CircuitNotFound(abs_path));
        }

        let circuit = CircuitConfig::new(&test_name)
            .with_absolute_file(abs_path)
            .with_template(template)
            .with_params(params)
            .with_public(public);

        let circomkit = Circomkit::new(config)?;

        Ok(Self {
            circomkit,
            circuit,
            compiled: false,
        })
    }

    /// Create a witness tester from a pre-configured CircuitConfig
    pub async fn from_circuit_config(circuit: CircuitConfig) -> Result<Self> {
        let config = CircomkitConfig::from_default_file()?;
        let circomkit = Circomkit::new(config)?;

        Ok(Self {
            circomkit,
            circuit,
            compiled: false,
        })
    }

    /// Create a witness tester from CircuitConfig with custom Circomkit config
    pub async fn from_circuit_config_with_settings(
        circuit: CircuitConfig,
        config: CircomkitConfig,
    ) -> Result<Self> {
        let circomkit = Circomkit::new(config)?;

        Ok(Self {
            circomkit,
            circuit,
            compiled: false,
        })
    }

    /// Compile the circuit if not already compiled
    pub async fn ensure_compiled(&mut self) -> Result<()> {
        if !self.compiled {
            self.circomkit.compile(&self.circuit).await?;
            self.compiled = true;
        }
        Ok(())
    }

    /// Test that a witness can be computed for the given inputs
    pub async fn expect_pass(&mut self, inputs: CircuitSignals) -> Result<CircuitSignals> {
        self.ensure_compiled().await?;

        let witness = self
            .circomkit
            .generate_witness(&self.circuit, &inputs)
            .await?;

        // Read the output signals from the witness
        let outputs = self.read_witness_outputs(&witness.path).await?;

        Ok(outputs)
    }

    /// Test that witness computation fails for the given inputs
    pub async fn expect_fail(&mut self, inputs: CircuitSignals) -> Result<()> {
        self.ensure_compiled().await?;

        let result = self
            .circomkit
            .generate_witness(&self.circuit, &inputs)
            .await;

        match result {
            Ok(_) => Err(CircomkitError::Other(
                "Expected witness generation to fail, but it succeeded".to_string(),
            )),
            Err(_) => Ok(()),
        }
    }

    /// Test that the outputs match expected values
    pub async fn expect_output(
        &mut self,
        inputs: CircuitSignals,
        expected: CircuitSignals,
    ) -> Result<WitnessTestResult> {
        self.ensure_compiled().await?;

        let witness = self
            .circomkit
            .generate_witness(&self.circuit, &inputs)
            .await?;
        let outputs = self.read_witness_outputs(&witness.path).await?;

        // Compare outputs with expected
        let mut passed = true;
        let mut errors = Vec::new();

        for (name, expected_value) in &expected {
            if let Some(actual_value) = outputs.get(name) {
                if !self.compare_signals(actual_value, expected_value) {
                    passed = false;
                    errors.push(format!(
                        "Signal '{}': expected {}, got {}",
                        name,
                        expected_value.as_string(),
                        actual_value.as_string()
                    ));
                }
            } else {
                passed = false;
                errors.push(format!("Signal '{}' not found in outputs", name));
            }
        }

        Ok(WitnessTestResult {
            passed,
            outputs,
            expected: Some(expected),
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        })
    }

    /// Check constraint count
    pub async fn expect_constraint_count(&mut self, expected: usize) -> Result<()> {
        self.ensure_compiled().await?;

        let info = self.circomkit.info(&self.circuit).await?;

        if info.constraints != expected {
            return Err(CircomkitError::ConstraintNotSatisfied {
                expected: expected.to_string(),
                actual: info.constraints.to_string(),
            });
        }

        Ok(())
    }

    /// Read output signals from a witness file
    async fn read_witness_outputs(&self, witness_path: &Path) -> Result<CircuitSignals> {
        let build_dir = self.circomkit.config().build_path(&self.circuit.name);
        let sym_path = build_dir.join(format!("{}.sym", self.circuit.name));

        if !sym_path.exists() {
            return Err(CircomkitError::CircuitNotFound(sym_path));
        }

        // Use snarkjs to export witness to json
        let output_path = build_dir.join("witness.json");
        let snarkjs = self.circomkit.config().snarkjs_command();

        let wasm_path = build_dir
            .join(format!("{}_js", self.circuit.name))
            .join(format!("{}.wasm", self.circuit.name));

        let output = Command::new(&snarkjs)
            .arg("wtns")
            .arg("export")
            .arg("json")
            .arg(witness_path)
            .arg(&output_path)
            .output()
            .map_err(CircomkitError::Io)?;

        if !output.status.success() {
            // If export fails, return empty map (some versions don't support this)
            return Ok(HashMap::new());
        }

        // Parse the witness JSON
        let content = fs::read_to_string(&output_path).await?;
        let witness_array: Vec<String> = serde_json::from_str(&content)?;

        // Read symbol file to map indices to signal names
        let sym_content = fs::read_to_string(&sym_path).await?;
        let mut signals = HashMap::new();

        for line in sym_content.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let idx: usize = parts[0].parse().unwrap_or(0);
                let name = parts[3].to_string();

                // Only include output signals (those starting with "main.")
                if name.starts_with("main.") && idx < witness_array.len() {
                    let signal_name = name.strip_prefix("main.").unwrap_or(&name).to_string();
                    signals.insert(signal_name, SignalValue::Single(witness_array[idx].clone()));
                }
            }
        }

        Ok(signals)
    }

    /// Compare two signal values for equality
    fn compare_signals(&self, actual: &SignalValue, expected: &SignalValue) -> bool {
        match (actual, expected) {
            (SignalValue::Single(a), SignalValue::Single(e)) => a == e,
            (SignalValue::Number(a), SignalValue::Number(e)) => a == e,
            (SignalValue::Single(a), SignalValue::Number(e)) => {
                a.parse::<i64>().map(|n| n == *e).unwrap_or(false)
            }
            (SignalValue::Number(a), SignalValue::Single(e)) => {
                e.parse::<i64>().map(|n| n == *a).unwrap_or(false)
            }
            (SignalValue::Array(a), SignalValue::Array(e)) => {
                a.len() == e.len()
                    && a.iter()
                        .zip(e.iter())
                        .all(|(av, ev)| self.compare_signals(av, ev))
            }
            _ => false,
        }
    }
}

/// Macro for convenient witness testing with file path
#[macro_export]
macro_rules! witness_test {
    ($name:expr, $file:expr, $template:expr, $params:expr, $public:expr, $inputs:expr) => {{
        let mut tester = WitnessTester::new($name, $file, $template, $params, $public).await?;
        tester.expect_pass($inputs).await
    }};
    ($name:expr, $file:expr, $template:expr, $params:expr, $public:expr, $inputs:expr, $expected:expr) => {{
        let mut tester = WitnessTester::new($name, $file, $template, $params, $public).await?;
        tester.expect_output($inputs, $expected).await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_comparison() {
        let tester = WitnessTester {
            circomkit: Circomkit::with_defaults().unwrap(),
            circuit: CircuitConfig::new("test"),
            compiled: false,
        };

        assert!(
            tester.compare_signals(&SignalValue::Single("42".into()), &SignalValue::Number(42))
        );
        assert!(
            tester.compare_signals(&SignalValue::Number(42), &SignalValue::Single("42".into()))
        );
        assert!(
            !tester.compare_signals(&SignalValue::Single("42".into()), &SignalValue::Number(43))
        );
    }
}
