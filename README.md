# Circomkit-RS

A Rust library for Circom circuit testing and development.

## Overview

Circomkit-RS provides a comprehensive toolkit for working with Circom circuits in Rust:

- **Circuit Configuration**: Manage circuit parameters and build settings
- **Witness Generation**: Generate and test circuit witnesses  
- **Proof Generation**: Create and verify zero-knowledge proofs
- **Testing Utilities**: Convenient testing helpers for circuit development

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Circom](https://docs.circom.io/getting-started/installation/) (2.0+)
- [SnarkJS](https://github.com/iden3/snarkjs) (0.7+)
- Node.js (for witness generation)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
circomkit = { git = "https://github.com/RajeshRk18/circomkit-rs" }
```

## Quick Start

### Basic Usage

```rust
use circomkit::{Circomkit, CircomkitConfig, CircuitConfig};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Circomkit instance
    let config = CircomkitConfig::default();
    let circomkit = Circomkit::new(config)?;

    // Configure a circuit
    let circuit = CircuitConfig::new("multiplier")
        .with_file("multiplier.circom")
        .with_template("Multiplier")
        .with_params(vec![2]);

    // Compile the circuit
    let artifacts = circomkit.compile(&circuit).await?;
    println!("Compiled to: {:?}", artifacts.r1cs);

    Ok(())
}
```

### Witness Testing

```rust
use circomkit::{WitnessTester, CircuitConfig};
use circomkit::utils::signals;

#[tokio::test]
async fn test_multiplier() -> Result<(), Box<dyn std::error::Error>> {
    let circuit = CircuitConfig::new("multiplier")
        .with_template("Multiplier")
        .with_params(vec![2]);

    let mut tester = WitnessTester::new(circuit).await?;

    // Test valid inputs
    let inputs = signals([
        ("a", 3.into()),
        ("b", 5.into()),
    ]);
    
    let outputs = tester.expect_pass(inputs).await?;
    assert_eq!(outputs.get("out").unwrap().as_string(), "15");

    Ok(())
}
```

### Proof Generation & Verification

```rust
use circomkit::{ProofTester, CircuitConfig};
use circomkit::utils::signals;
use std::path::PathBuf;

#[tokio::test]
async fn test_proof() -> Result<(), Box<dyn std::error::Error>> {
    let circuit = CircuitConfig::new("multiplier")
        .with_template("Multiplier")
        .with_params(vec![2]);

    let ptau = PathBuf::from("ptau/powersOfTau28_hez_final_10.ptau");
    let mut tester = ProofTester::new(circuit, ptau).await?;

    let inputs = signals([
        ("a", 3.into()),
        ("b", 5.into()),
    ]);

    // Generate and verify proof
    tester.expect_valid_proof(inputs).await?;

    Ok(())
}
```

## Configuration

Create a `circomkit.json` file in your project root:

```json
{
  "version": "0.1.0",
  "protocol": "groth16",
  "prime": "bn128",
  "optimization": 1,
  "verbose": false,
  "dirCircuits": "circuits",
  "dirInputs": "inputs",
  "dirBuild": "build",
  "dirPtau": "ptau"
}
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `protocol` | string | `"groth16"` | Proving system: `groth16`, `plonk`, or `fflonk` |
| `prime` | string | `"bn128"` | Prime field: `bn128`, `bls12381`, or `goldilocks` |
| `optimization` | number | `1` | Circom optimization level (0-2) |
| `verbose` | boolean | `false` | Enable verbose logging |
| `dirCircuits` | string | `"circuits"` | Directory for circuit files |
| `dirInputs` | string | `"inputs"` | Directory for input files |
| `dirBuild` | string | `"build"` | Directory for build artifacts |
| `dirPtau` | string | `"ptau"` | Directory for PTAU files |

## Project Structure

```
my-project/
├── circuits.json          # Circuit configurations
├── circomkit.json         # Circomkit configuration
├── circuits/
│   ├── main/              # Auto-generated main components
│   └── multiplier.circom  # Your circuits
├── inputs/
│   └── multiplier/
│       └── default.json   # Input signals
├── ptau/
│   └── *.ptau             # Powers of Tau files
└── build/                 # Build artifacts
    └── multiplier/
        ├── multiplier.r1cs
        ├── multiplier.sym
        └── multiplier_js/
```

## PTAU Files

Download Powers of Tau files from the Hermez ceremony:

```rust
use circomkit::utils::{get_recommended_ptau, download_ptau};
use std::path::Path;

async fn setup_ptau(num_constraints: usize) -> PathBuf {
    let info = get_recommended_ptau(num_constraints);
    download_ptau(&info, Path::new("ptau")).await.unwrap()
}
```

## API Reference

### Circomkit

Main interface for circuit operations:

- `compile(circuit)` - Compile a circuit
- `generate_witness(circuit, inputs)` - Generate a witness
- `setup(circuit, ptau_path)` - Set up proving/verification keys
- `prove(circuit, inputs)` - Generate a proof
- `verify(circuit, proof, public_signals)` - Verify a proof
- `export_verifier(circuit)` - Export Solidity verifier

### WitnessTester

Testing utilities for witnesses:

- `expect_pass(inputs)` - Test that witness generation succeeds
- `expect_fail(inputs)` - Test that witness generation fails
- `expect_output(inputs, expected)` - Test output values
- `expect_constraint_count(n)` - Verify constraint count

### ProofTester

Testing utilities for proofs:

- `prove_and_verify(inputs)` - Generate and verify a proof
- `expect_valid_proof(inputs)` - Test that a valid proof is generated
- `expect_tampered_fails(inputs, tamper_fn)` - Test that tampered proofs fail
- `export_solidity_verifier()` - Export Solidity verifier
- `get_calldata(inputs)` - Get calldata for on-chain verification

## Acknowledgement
https://github.com/erhant/circomkit

## License

MIT
