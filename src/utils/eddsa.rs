//! EdDSA Poseidon signature utilities for testing circomlib circuits
//!
//! This module provides EdDSA signature generation using the Baby JubJub curve
//! and Poseidon hash function, compatible with circomlib's EdDSAPoseidonVerifier.

use babyjubjub_rs::{new_key, Point, PrivateKey, Signature};
use num_bigint::BigInt;

/// EdDSA test inputs for circomlib's EdDSAPoseidonVerifier circuit
#[derive(Debug, Clone)]
pub struct EdDSATestInputs {
    /// Enable flag (1 = enabled, 0 = disabled)
    pub enabled: String,
    /// Public key X coordinate
    pub ax: String,
    /// Public key Y coordinate
    pub ay: String,
    /// Signature R8 point X coordinate
    pub r8x: String,
    /// Signature R8 point Y coordinate
    pub r8y: String,
    /// Signature S scalar
    pub s: String,
    /// Message
    pub m: String,
}

/// Generate a new EdDSA private key
pub fn generate_private_key() -> PrivateKey {
    new_key()
}

/// Create a private key from a 32-byte seed
pub fn private_key_from_seed(seed: &[u8; 32]) -> PrivateKey {
    PrivateKey { key: *seed }
}

/// Sign a message with the given private key and return test inputs
pub fn sign_poseidon(private_key: &PrivateKey, message: i64) -> EdDSATestInputs {
    let msg = BigInt::from(message);
    let signature = private_key.sign(msg.clone()).expect("Failed to sign message");
    let public_key = private_key.public();

    EdDSATestInputs {
        enabled: "1".to_string(),
        ax: point_x_to_string(&public_key),
        ay: point_y_to_string(&public_key),
        r8x: point_x_to_string(&signature.r_b8),
        r8y: point_y_to_string(&signature.r_b8),
        s: signature.s.to_string(),
        m: message.to_string(),
    }
}

/// Sign a BigInt message with the given private key and return test inputs
pub fn sign_poseidon_bigint(private_key: &PrivateKey, message: &BigInt) -> EdDSATestInputs {
    let signature = private_key
        .sign(message.clone())
        .expect("Failed to sign message");
    let public_key = private_key.public();

    EdDSATestInputs {
        enabled: "1".to_string(),
        ax: point_x_to_string(&public_key),
        ay: point_y_to_string(&public_key),
        r8x: point_x_to_string(&signature.r_b8),
        r8y: point_y_to_string(&signature.r_b8),
        s: signature.s.to_string(),
        m: message.to_string(),
    }
}

/// Verify an EdDSA Poseidon signature
pub fn verify_poseidon(public_key: &Point, signature: &Signature, message: &BigInt) -> bool {
    babyjubjub_rs::verify(public_key.clone(), signature.clone(), message.clone())
}

/// Convert a Point's X coordinate to a decimal string
fn point_x_to_string(point: &Point) -> String {
    // Use ff_ce's into_repr() to get the internal representation
    use ff_ce::PrimeField;
    let repr = point.x.into_repr();
    // into_repr returns a 4 element array of u64s (256 bits total)
    let mut bytes = [0u8; 32];
    for (i, limb) in repr.0.iter().enumerate() {
        let limb_bytes = limb.to_le_bytes();
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb_bytes);
    }
    BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes).to_string()
}

/// Convert a Point's Y coordinate to a decimal string
fn point_y_to_string(point: &Point) -> String {
    use ff_ce::PrimeField;
    let repr = point.y.into_repr();
    let mut bytes = [0u8; 32];
    for (i, limb) in repr.0.iter().enumerate() {
        let limb_bytes = limb.to_le_bytes();
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb_bytes);
    }
    BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let private_key = generate_private_key();
        let message = BigInt::from(1234);

        let signature = private_key.sign(message.clone()).expect("Failed to sign");
        let public_key = private_key.public();

        assert!(verify_poseidon(&public_key, &signature, &message));
    }

    #[test]
    fn test_sign_poseidon_generates_inputs() {
        let seed: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x00, 0x01,
        ];
        let private_key = private_key_from_seed(&seed);
        let inputs = sign_poseidon(&private_key, 1234);

        assert_eq!(inputs.enabled, "1");
        assert_eq!(inputs.m, "1234");
        assert!(!inputs.ax.is_empty());
        assert!(!inputs.ay.is_empty());
        assert!(!inputs.r8x.is_empty());
        assert!(!inputs.r8y.is_empty());
        assert!(!inputs.s.is_empty());
    }

    #[test]
    fn test_deterministic_key_from_seed() {
        let seed: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x00, 0x01, 0x02, 0x03,
            0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x00, 0x01,
        ];
        let key1 = private_key_from_seed(&seed);
        let key2 = private_key_from_seed(&seed);

        let inputs1 = sign_poseidon(&key1, 1234);
        let inputs2 = sign_poseidon(&key2, 1234);

        // Same seed should produce same public key
        assert_eq!(inputs1.ax, inputs2.ax);
        assert_eq!(inputs1.ay, inputs2.ay);
    }
}
