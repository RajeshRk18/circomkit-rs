//! Main Circomkit implementation

use crate::core::CircomkitConfig;
use crate::error::{CircomkitError, Result};
use crate::types::{
    CircuitArtifacts, CircuitConfig, CircuitInfo, CircuitSignals, Proof, PublicSignals,
    VerificationKey, Witness,
};
use log::{debug, info};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Main Circomkit instance for circuit testing and development
#[derive(Debug)]
pub struct Circomkit {
    /// Configuration
    config: CircomkitConfig,
    /// Loaded circuit configurations
    circuits: HashMap<String, CircuitConfig>,
}

impl Circomkit {
    /// Create a new Circomkit instance with the given configuration
    pub fn new(config: CircomkitConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            circuits: HashMap::new(),
        })
    }

    /// Create a new Circomkit instance with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(CircomkitConfig::default())
    }

    /// Create a new Circomkit instance loading config from the default file
    pub fn from_config_file() -> Result<Self> {
        let config = CircomkitConfig::from_default_file()?;
        Self::new(config)
    }

    /// Get the current configuration
    pub fn config(&self) -> &CircomkitConfig {
        &self.config
    }

    /// Load circuit configurations from the circuits.json file
    pub async fn load_circuits(&mut self) -> Result<()> {
        let path = &self.config.circuits;
        if path.exists() {
            let content = fs::read_to_string(path).await?;
            self.circuits = serde_json::from_str(&content)?;
            info!("Loaded {} circuit configurations", self.circuits.len());
        }
        Ok(())
    }

    /// Add a circuit configuration
    pub fn add_circuit(&mut self, config: CircuitConfig) {
        self.circuits.insert(config.name.clone(), config);
    }

    /// Get a circuit configuration by name
    pub fn get_circuit(&self, name: &str) -> Option<&CircuitConfig> {
        self.circuits.get(name)
    }

    /// Compile a circuit
    pub async fn compile(&self, circuit: &CircuitConfig) -> Result<CircuitArtifacts> {
        info!("Compiling circuit: {}", circuit.name);

        // Ensure build directory exists
        let build_dir = self.config.build_path(&circuit.name);
        fs::create_dir_all(&build_dir).await?;

        // Generate main component if needed
        let main_path = self.generate_main_component(circuit).await?;

        // Build circom command
        let circom = self.config.circom_command();
        let mut cmd = Command::new(&circom);

        cmd.arg(&main_path)
            .arg("--r1cs")
            .arg("--wasm")
            .arg("--sym")
            .arg("-o")
            .arg(&build_dir)
            .arg("-p")
            .arg(self.config.prime.to_string())
            .arg(format!("--O{}", self.config.optimization));

        // Add include paths
        for include in &self.config.include {
            cmd.arg("-l").arg(include);
        }

        debug!("Running: {:?}", cmd);

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CircomkitError::tool_not_found(&circom)
            } else {
                CircomkitError::Io(e)
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::CommandFailed {
                command: circom,
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        info!("Circuit compiled successfully: {}", circuit.name);

        Ok(CircuitArtifacts {
            r1cs: build_dir.join(format!("{}.r1cs", circuit.name)),
            wasm: build_dir
                .join(format!("{}_js", circuit.name))
                .join(format!("{}.wasm", circuit.name)),
            sym: build_dir.join(format!("{}.sym", circuit.name)),
            pkey: None,
            vkey: None,
        })
    }

    /// Generate a main component file for the circuit
    ///
    /// The main component is generated in `build/main/` directory.
    /// If the circuit has an absolute file path, it uses that directly.
    /// Otherwise, it uses the relative path from the circuits directory.
    async fn generate_main_component(&self, circuit: &CircuitConfig) -> Result<PathBuf> {
        // Put main components in build/main/ directory
        let main_dir = self.config.dir_build.join("main");
        fs::create_dir_all(&main_dir).await?;

        let main_path = main_dir.join(format!("{}.circom", circuit.name));

        // Generate the main component
        let params = if circuit.params.is_empty() {
            String::new()
        } else {
            circuit
                .params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let public_signals = if circuit.public.is_empty() {
            String::new()
        } else {
            format!(" {{public [{}]}}", circuit.public.join(", "))
        };

        // Determine the include path
        let include_path = if let Some(abs_path) = &circuit.absolute_file {
            // Use absolute path directly
            abs_path.to_string_lossy().to_string()
        } else {
            // Use relative path from build/main to circuits directory
            // build/main -> ../../circuits/file.circom
            format!(
                "../../{}/{}",
                self.config.dir_circuits.display(),
                circuit.file
            )
        };

        // circom 2.1.9
        let content = format!(
            r#"pragma circom 2.1.9;

include "{}";

component main{} = {}({});
"#,
            include_path, public_signals, circuit.template, params
        );

        fs::write(&main_path, content).await?;
        debug!("Generated main component: {:?}", main_path);

        Ok(main_path)
    }

    /// Generate a witness for the given inputs
    pub async fn generate_witness(
        &self,
        circuit: &CircuitConfig,
        inputs: &CircuitSignals,
    ) -> Result<Witness> {
        info!("Generating witness for: {}", circuit.name);

        let build_dir = self.config.build_path(&circuit.name);
        let wasm_dir = build_dir.join(format!("{}_js", circuit.name));
        let witness_calc = wasm_dir.join("generate_witness.js");
        let wasm_file = wasm_dir.join(format!("{}.wasm", circuit.name));

        // Check if circuit is compiled
        if !wasm_file.exists() {
            return Err(CircomkitError::CircuitNotFound(wasm_file));
        }

        // Write inputs to temp file
        let input_path = build_dir.join("input.json");
        let input_json = serde_json::to_string_pretty(inputs)?;
        fs::write(&input_path, input_json).await?;

        // Generate witness
        let witness_path = build_dir.join("witness.wtns");

        let output = Command::new("node")
            .arg(&witness_calc)
            .arg(&wasm_file)
            .arg(&input_path)
            .arg(&witness_path)
            .output()
            .map_err(|e| CircomkitError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::witness_failed(stderr.to_string()));
        }

        info!("Witness generated successfully");

        Ok(Witness {
            path: witness_path,
            num_signals: 0, // TODO: Parse from witness file
        })
    }

    /// Set up the proving and verification keys
    pub async fn setup(
        &self,
        circuit: &CircuitConfig,
        ptau_path: &Path,
    ) -> Result<CircuitArtifacts> {
        info!("Setting up keys for: {}", circuit.name);

        let build_dir = self.config.build_path(&circuit.name);
        let r1cs_path = build_dir.join(format!("{}.r1cs", circuit.name));

        if !r1cs_path.exists() {
            return Err(CircomkitError::CircuitNotFound(r1cs_path));
        }

        if !ptau_path.exists() {
            return Err(CircomkitError::PtauNotFound(ptau_path.to_path_buf()));
        }

        let snarkjs = self.config.snarkjs_command();
        let protocol = self.config.protocol.to_string();

        // Generate zkey
        let zkey_path = build_dir.join(format!("{}_pkey.zkey", protocol));

        let output = Command::new(&snarkjs)
            .arg(&protocol)
            .arg("setup")
            .arg(&r1cs_path)
            .arg(ptau_path)
            .arg(&zkey_path)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CircomkitError::tool_not_found(&snarkjs)
                } else {
                    CircomkitError::Io(e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::CommandFailed {
                command: snarkjs.clone(),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        // Export verification key
        let vkey_path = build_dir.join(format!("{}_vkey.json", protocol));

        let output = Command::new(&snarkjs)
            .arg("zkey")
            .arg("export")
            .arg("verificationkey")
            .arg(&zkey_path)
            .arg(&vkey_path)
            .output()
            .map_err(|e| CircomkitError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::CommandFailed {
                command: snarkjs,
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        info!("Setup completed successfully");

        Ok(CircuitArtifacts {
            r1cs: r1cs_path,
            wasm: build_dir
                .join(format!("{}_js", circuit.name))
                .join(format!("{}.wasm", circuit.name)),
            sym: build_dir.join(format!("{}.sym", circuit.name)),
            pkey: Some(zkey_path),
            vkey: Some(vkey_path),
        })
    }

    /// Generate a proof
    pub async fn prove(
        &self,
        circuit: &CircuitConfig,
        inputs: &CircuitSignals,
    ) -> Result<(Proof, PublicSignals)> {
        info!("Generating proof for: {}", circuit.name);

        // First generate the witness
        let witness = self.generate_witness(circuit, inputs).await?;

        let build_dir = self.config.build_path(&circuit.name);
        let protocol = self.config.protocol.to_string();
        let zkey_path = build_dir.join(format!("{}_pkey.zkey", protocol));

        if !zkey_path.exists() {
            return Err(CircomkitError::proof_failed(
                "Proving key not found. Run setup first.",
            ));
        }

        let proof_path = build_dir.join(format!("{}_proof.json", protocol));
        let public_path = build_dir.join("public.json");

        let snarkjs = self.config.snarkjs_command();

        let output = Command::new(&snarkjs)
            .arg(&protocol)
            .arg("prove")
            .arg(&zkey_path)
            .arg(&witness.path)
            .arg(&proof_path)
            .arg(&public_path)
            .output()
            .map_err(|e| CircomkitError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::proof_failed(stderr.to_string()));
        }

        // Read proof and public signals
        let proof_content = fs::read_to_string(&proof_path).await?;
        let proof_data: serde_json::Value = serde_json::from_str(&proof_content)?;

        let public_content = fs::read_to_string(&public_path).await?;
        let public_signals: Vec<String> = serde_json::from_str(&public_content)?;

        info!("Proof generated successfully");

        Ok((
            Proof {
                protocol: self.config.protocol,
                data: proof_data,
            },
            PublicSignals::new(public_signals),
        ))
    }

    /// Verify a proof
    pub async fn verify(
        &self,
        circuit: &CircuitConfig,
        proof: &Proof,
        public_signals: &PublicSignals,
    ) -> Result<bool> {
        info!("Verifying proof for: {}", circuit.name);

        let build_dir = self.config.build_path(&circuit.name);
        let protocol = self.config.protocol.to_string();
        let vkey_path = build_dir.join(format!("{}_vkey.json", protocol));

        if !vkey_path.exists() {
            return Err(CircomkitError::verification_failed(
                "Verification key not found. Run setup first.",
            ));
        }

        // Write proof and public signals to temp files
        let proof_path = build_dir.join("temp_proof.json");
        let public_path = build_dir.join("temp_public.json");

        fs::write(&proof_path, serde_json::to_string(&proof.data)?).await?;
        fs::write(&public_path, serde_json::to_string(&public_signals.0)?).await?;

        let snarkjs = self.config.snarkjs_command();

        let output = Command::new(&snarkjs)
            .arg(&protocol)
            .arg("verify")
            .arg(&vkey_path)
            .arg(&public_path)
            .arg(&proof_path)
            .output()
            .map_err(|e| CircomkitError::Io(e))?;

        // Clean up temp files
        let _ = fs::remove_file(&proof_path).await;
        let _ = fs::remove_file(&public_path).await;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Invalid proof") || stderr.contains("INVALID") {
                return Ok(false);
            }
            return Err(CircomkitError::verification_failed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let is_valid = stdout.contains("OK") || stdout.contains("valid");

        info!("Proof verification result: {}", is_valid);

        Ok(is_valid)
    }

    /// Export a Solidity verifier contract
    pub async fn export_verifier(&self, circuit: &CircuitConfig) -> Result<PathBuf> {
        info!("Exporting Solidity verifier for: {}", circuit.name);

        let build_dir = self.config.build_path(&circuit.name);
        let protocol = self.config.protocol.to_string();
        let zkey_path = build_dir.join(format!("{}_pkey.zkey", protocol));

        if !zkey_path.exists() {
            return Err(CircomkitError::proof_failed(
                "Proving key not found. Run setup first.",
            ));
        }

        let verifier_path = build_dir.join(format!("{}_verifier.sol", protocol));

        let snarkjs = self.config.snarkjs_command();

        let output = Command::new(&snarkjs)
            .arg("zkey")
            .arg("export")
            .arg("solidityverifier")
            .arg(&zkey_path)
            .arg(&verifier_path)
            .output()
            .map_err(|e| CircomkitError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::CommandFailed {
                command: snarkjs,
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        info!("Verifier exported: {:?}", verifier_path);

        Ok(verifier_path)
    }

    /// Get information about a compiled circuit
    pub async fn info(&self, circuit: &CircuitConfig) -> Result<CircuitInfo> {
        let build_dir = self.config.build_path(&circuit.name);
        let r1cs_path = build_dir.join(format!("{}.r1cs", circuit.name));

        if !r1cs_path.exists() {
            return Err(CircomkitError::CircuitNotFound(r1cs_path));
        }

        let snarkjs = self.config.snarkjs_command();

        let output = Command::new(&snarkjs)
            .arg("r1cs")
            .arg("info")
            .arg(&r1cs_path)
            .arg("--json")
            .output()
            .map_err(|e| CircomkitError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::CommandFailed {
                command: snarkjs,
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse the output (snarkjs outputs human-readable format)
        // This is a simplified parser
        let mut info = CircuitInfo {
            constraints: 0,
            private_inputs: 0,
            public_inputs: 0,
            public_outputs: 0,
            labels: 0,
        };

        for line in stdout.lines() {
            if line.contains("Constraints:") {
                if let Some(n) = line.split(':').nth(1) {
                    info.constraints = n.trim().parse().unwrap_or(0);
                }
            } else if line.contains("Private Inputs:") {
                if let Some(n) = line.split(':').nth(1) {
                    info.private_inputs = n.trim().parse().unwrap_or(0);
                }
            } else if line.contains("Public Inputs:") {
                if let Some(n) = line.split(':').nth(1) {
                    info.public_inputs = n.trim().parse().unwrap_or(0);
                }
            } else if line.contains("Outputs:") {
                if let Some(n) = line.split(':').nth(1) {
                    info.public_outputs = n.trim().parse().unwrap_or(0);
                }
            } else if line.contains("Labels:") {
                if let Some(n) = line.split(':').nth(1) {
                    info.labels = n.trim().parse().unwrap_or(0);
                }
            }
        }

        Ok(info)
    }

    /// Clean build artifacts for a circuit
    pub async fn clean(&self, circuit: &CircuitConfig) -> Result<()> {
        let build_dir = self.config.build_path(&circuit.name);
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).await?;
            info!("Cleaned build directory: {:?}", build_dir);
        }
        Ok(())
    }

    /// Clean all build artifacts
    pub async fn clean_all(&self) -> Result<()> {
        if self.config.dir_build.exists() {
            fs::remove_dir_all(&self.config.dir_build).await?;
            info!("Cleaned all build artifacts");
        }
        Ok(())
    }

    /// Read input signals from a JSON file
    pub async fn read_inputs(&self, circuit: &str, input_name: &str) -> Result<CircuitSignals> {
        let path = self.config.input_path(circuit, input_name);
        let content = fs::read_to_string(&path).await.map_err(|_| {
            CircomkitError::InvalidSignals(format!("Input file not found: {:?}", path))
        })?;
        let signals: CircuitSignals = serde_json::from_str(&content)?;
        Ok(signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_circomkit() {
        let config = CircomkitConfig::default();
        let circomkit = Circomkit::new(config);
        assert!(circomkit.is_ok());
    }

    #[test]
    fn test_add_circuit() {
        let config = CircomkitConfig::default();
        let mut circomkit = Circomkit::new(config).unwrap();

        let circuit = CircuitConfig::new("test")
            .with_template("TestCircuit")
            .with_params(vec![10]);

        circomkit.add_circuit(circuit);

        assert!(circomkit.get_circuit("test").is_some());
    }
}
