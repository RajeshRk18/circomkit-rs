//! Proof testing utilities

use crate::core::{Circomkit, CircomkitConfig};
use crate::error::{CircomkitError, Result};
use crate::types::{CircuitConfig, CircuitSignals, Proof, ProofTestResult, PublicSignals};
use std::path::PathBuf;

/// Tester for circuit proofs
pub struct ProofTester {
    circomkit: Circomkit,
    circuit: CircuitConfig,
    ptau_path: PathBuf,
    setup_complete: bool,
}

impl ProofTester {
    /// Create a new proof tester for a circuit
    pub async fn new(circuit: CircuitConfig, ptau_path: PathBuf) -> Result<Self> {
        let config = CircomkitConfig::from_default_file()?;
        let circomkit = Circomkit::new(config)?;

        Ok(Self {
            circomkit,
            circuit,
            ptau_path,
            setup_complete: false,
        })
    }

    /// Create a new proof tester with custom configuration
    pub async fn with_config(
        circuit: CircuitConfig,
        ptau_path: PathBuf,
        config: CircomkitConfig,
    ) -> Result<Self> {
        let circomkit = Circomkit::new(config)?;

        Ok(Self {
            circomkit,
            circuit,
            ptau_path,
            setup_complete: false,
        })
    }

    /// Ensure the circuit is compiled and keys are set up
    pub async fn ensure_setup(&mut self) -> Result<()> {
        if !self.setup_complete {
            // Compile circuit
            self.circomkit.compile(&self.circuit).await?;

            // Set up proving/verification keys
            self.circomkit.setup(&self.circuit, &self.ptau_path).await?;

            self.setup_complete = true;
        }
        Ok(())
    }

    /// Generate and verify a proof
    pub async fn prove_and_verify(&mut self, inputs: CircuitSignals) -> Result<ProofTestResult> {
        self.ensure_setup().await?;

        // Generate proof
        let (proof, public_signals) = self.circomkit.prove(&self.circuit, &inputs).await?;

        // Verify proof
        let valid = self
            .circomkit
            .verify(&self.circuit, &proof, &public_signals)
            .await?;

        Ok(ProofTestResult {
            valid,
            proof: Some(proof),
            public_signals: Some(public_signals),
            error: None,
        })
    }

    /// Test that a valid proof can be generated and verified
    pub async fn expect_valid_proof(&mut self, inputs: CircuitSignals) -> Result<()> {
        let result = self.prove_and_verify(inputs).await?;

        if !result.valid {
            return Err(CircomkitError::verification_failed(
                "Proof was generated but verification failed",
            ));
        }

        Ok(())
    }

    /// Test that proof generation fails for invalid inputs
    pub async fn expect_invalid_inputs(&mut self, inputs: CircuitSignals) -> Result<()> {
        self.ensure_setup().await?;

        let result = self.circomkit.prove(&self.circuit, &inputs).await;

        match result {
            Ok(_) => Err(CircomkitError::Other(
                "Expected proof generation to fail for invalid inputs, but it succeeded"
                    .to_string(),
            )),
            Err(_) => Ok(()),
        }
    }

    /// Verify a proof with tampered public signals (should fail)
    pub async fn expect_tampered_fails(
        &mut self,
        inputs: CircuitSignals,
        tamper_fn: impl FnOnce(&mut PublicSignals),
    ) -> Result<()> {
        self.ensure_setup().await?;

        // Generate valid proof
        let (proof, mut public_signals) = self.circomkit.prove(&self.circuit, &inputs).await?;

        // Tamper with public signals
        tamper_fn(&mut public_signals);

        // Verify should fail
        let valid = self
            .circomkit
            .verify(&self.circuit, &proof, &public_signals)
            .await?;

        if valid {
            return Err(CircomkitError::Other(
                "Expected verification to fail for tampered signals, but it passed".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate a proof and return it
    pub async fn generate_proof(
        &mut self,
        inputs: CircuitSignals,
    ) -> Result<(Proof, PublicSignals)> {
        self.ensure_setup().await?;
        self.circomkit.prove(&self.circuit, &inputs).await
    }

    /// Verify an existing proof
    pub async fn verify_proof(
        &mut self,
        proof: &Proof,
        public_signals: &PublicSignals,
    ) -> Result<bool> {
        self.ensure_setup().await?;
        self.circomkit
            .verify(&self.circuit, proof, public_signals)
            .await
    }

    /// Export Solidity verifier contract
    pub async fn export_solidity_verifier(&mut self) -> Result<PathBuf> {
        self.ensure_setup().await?;
        self.circomkit.export_verifier(&self.circuit).await
    }

    /// Get the calldata for verifying a proof on-chain
    pub async fn get_calldata(&mut self, inputs: CircuitSignals) -> Result<String> {
        self.ensure_setup().await?;

        let (proof, public_signals) = self.circomkit.prove(&self.circuit, &inputs).await?;

        let build_dir = self.circomkit.config().build_path(&self.circuit.name);
        let protocol = self.circomkit.config().protocol.to_string();

        // Write proof and public signals to temp files
        let proof_path = build_dir.join("calldata_proof.json");
        let public_path = build_dir.join("calldata_public.json");

        tokio::fs::write(&proof_path, serde_json::to_string(&proof.data)?).await?;
        tokio::fs::write(&public_path, serde_json::to_string(&public_signals.0)?).await?;

        let snarkjs = self.circomkit.config().snarkjs_command();

        let output = std::process::Command::new(&snarkjs)
            .arg("zkey")
            .arg("export")
            .arg("soliditycalldata")
            .arg(&public_path)
            .arg(&proof_path)
            .output()
            .map_err(CircomkitError::Io)?;

        // Clean up temp files
        let _ = tokio::fs::remove_file(&proof_path).await;
        let _ = tokio::fs::remove_file(&public_path).await;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::CommandFailed {
                command: snarkjs,
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Macro for convenient proof testing
#[macro_export]
macro_rules! proof_test {
    ($circuit:expr, $ptau:expr, $inputs:expr) => {{
        let mut tester = ProofTester::new($circuit, $ptau).await?;
        tester.expect_valid_proof($inputs).await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require actual circom/snarkjs installation
    // These are placeholder tests for the structure

    #[test]
    fn test_proof_tester_creation() {
        // This would be an async test in practice
        let circuit = CircuitConfig::new("test");
        let ptau_path = PathBuf::from("test.ptau");

        // Just verify the types compile correctly
        assert_eq!(circuit.name, "test");
        assert_eq!(ptau_path.to_str().unwrap(), "test.ptau");
    }
}
