//! # Circomkit-rs
//!
//! A Rust library for Circom circuit testing and development.
//!
//! ## Overview
//!
//! Circomkit-rs provides a comprehensive toolkit for working with Circom circuits:
//!
//! - **Circuit Configuration**: Manage circuit parameters and build settings
//! - **Witness Generation**: Generate and test circuit witnesses
//! - **Proof Generation**: Create and verify zero-knowledge proofs
//! - **Testing Utilities**: Convenient testing helpers for circuit development
//!
//! ## Example
//!
//! ```rust,ignore
//! use circomkit::{Circomkit, CircomkitConfig, CircuitConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new Circomkit instance
//!     let config = CircomkitConfig::default();
//!     let circomkit = Circomkit::new(config)?;
//!
//!     // Configure a circuit
//!     let circuit_config = CircuitConfig::new("multiplier")
//!         .with_file("circuits/multiplier.circom")
//!         .with_template("Multiplier")
//!         .with_params(vec![2]);
//!
//!     // Compile the circuit
//!     circomkit.compile(&circuit_config).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod error;
pub mod testers;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use core::{Circomkit, CircomkitConfig};
pub use error::{CircomkitError, Result};
pub use testers::{ProofTester, WitnessTester};
pub use types::{CircuitConfig, CircuitSignals, Proof, VerificationKey};
