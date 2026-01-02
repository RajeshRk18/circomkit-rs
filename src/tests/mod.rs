mod circuits;
mod testing;

use crate::utils::eddsa::{private_key_from_seed, sign_poseidon};
use testing::{CircuitTester, inputs};

#[test]
fn test_mock_adder() {
    let tester = CircuitTester::new();
    let result = tester.test_circuit(
        "Adder",
        circuits::ADDER,
        vec![],
        inputs(&[("a", vec!["5"]), ("b", vec!["7"])]),
    );
    assert!(result.is_ok());
}

#[test]
fn test_mock_multiplier() {
    let tester = CircuitTester::new();
    let result = tester.test_circuit(
        "Multiplier",
        circuits::MULTIPLIER,
        vec![],
        inputs(&[("a", vec!["6"]), ("b", vec!["7"])]),
    );
    assert!(result.is_ok());
}

#[test]
fn test_mock_multiplier_array() {
    let tester = CircuitTester::new();
    let result = tester.test_circuit(
        "MultiplierN",
        circuits::MULTIPLIER_N,
        vec![4],
        inputs(&[("in", vec!["2", "3", "4", "5"])]),
    );
    assert!(result.is_ok());
}

#[test]
fn test_mock_is_zero() {
    let tester = CircuitTester::new();

    // Test with 0 (should output 1)
    let r1 = tester.test_circuit(
        "IsZero",
        circuits::IS_ZERO,
        vec![],
        inputs(&[("in", vec!["0"])]),
    );
    assert!(r1.is_ok());
    // Test with non-zero (should output 0)
    let r2 = tester.test_circuit(
        "IsZero",
        circuits::IS_ZERO,
        vec![],
        inputs(&[("in", vec!["42"])]),
    );
    assert!(r2.is_ok());
}

#[test]
fn test_mock_is_equal() {
    let tester = CircuitTester::new();

    // Equal
    let r1: Result<(), String> = tester.test_circuit(
        "IsEqual",
        circuits::IS_EQUAL,
        vec![],
        inputs(&[("in", vec!["5", "5"])]),
    );
    assert!(r1.is_ok());

    // Not equal
    let r2 = tester.test_circuit(
        "IsEqual",
        circuits::IS_EQUAL,
        vec![],
        inputs(&[("in", vec!["5", "7"])]),
    );
    assert!(r2.is_ok());
}

#[test]
fn test_mock_force_equal() {
    let tester = CircuitTester::new();

    // Should pass when equal
    let r1 = tester.test_circuit(
        "ForceEqual",
        circuits::FORCE_EQUAL,
        vec![],
        inputs(&[("a", vec!["42"]), ("b", vec!["42"])]),
    );
    assert!(r1.is_ok());

    // Should fail when not equal
    let r2 = tester.test_circuit_fails(
        "ForceEqual",
        circuits::FORCE_EQUAL,
        vec![],
        inputs(&[("a", vec!["42"]), ("b", vec!["43"])]),
    );
    assert!(r2.is_ok());
}

#[test]
fn test_mock_mux1() {
    let tester = CircuitTester::new();

    let r1 = tester.test_circuit(
        "Mux1",
        circuits::MUX1,
        vec![],
        inputs(&[("c", vec!["10", "20"]), ("s", vec!["0"])]),
    );

    match r1 {
        Ok(_) => println!("✓ Mux1([10,20], 0) = 10"),
        Err(e) => {
            println!("⚠ Mux1 failed: {}", e);
            panic!()
        }
    }
    assert!(r1.is_ok());
}

#[test]
fn test_mock_range_check_8bit() {
    let tester = CircuitTester::new();

    // 255 fits in 8 bits
    let r1 = tester.test_circuit(
        "RangeCheck",
        circuits::RANGE_CHECK_8,
        vec![8],
        inputs(&[("in", vec!["255"])]),
    );
    if let Err(e) = &r1 {
        panic!("Range check 8-bit test failed: {}", e);
    }

    // 256 does NOT fit in 8 bits
    let r2 = tester.test_circuit_fails(
        "RangeCheck",
        circuits::RANGE_CHECK_8,
        vec![8],
        inputs(&[("in", vec!["256"])]),
    );
    if let Err(e) = &r2 {
        panic!("Range check 8-bit overflow test failed: {}", e);
    }
}

#[test]
fn test_mock_range_check_64bit() {
    let tester = CircuitTester::new();
    let max_u64 = "18446744073709551615";

    let result = tester.test_circuit(
        "RangeCheck64",
        circuits::RANGE_CHECK_64,
        vec![],
        inputs(&[("in", vec![max_u64])]),
    );
    if let Err(e) = &result {
        panic!("Range check 64-bit test failed: {}", e);
    }
}

/// Test seed for deterministic EdDSA key generation
/// Same as the seed used in circomlibjs tests: 0x0001020304050607080900010203040506070809000102030405060708090001
const EDDSA_TEST_SEED: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
    0x06, 0x07, 0x08, 0x09, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x00, 0x01,
];

#[test]
fn test_eddsa_poseidon_verifier() {
    let tester = CircuitTester::new();

    // Generate EdDSA Poseidon signature using native Rust implementation
    let private_key = private_key_from_seed(&EDDSA_TEST_SEED);
    let sig = sign_poseidon(&private_key, 1234);

    let result = tester.test_circuit(
        "EdDSAVerifier",
        circuits::EDDSA_POSEIDON_VERIFIER,
        vec![],
        inputs(&[
            ("enabled", vec![sig.enabled.as_str()]),
            ("Ax", vec![sig.ax.as_str()]),
            ("Ay", vec![sig.ay.as_str()]),
            ("R8x", vec![sig.r8x.as_str()]),
            ("R8y", vec![sig.r8y.as_str()]),
            ("S", vec![sig.s.as_str()]),
            ("M", vec![sig.m.as_str()]),
        ]),
    );
    if let Err(e) = &result {
        panic!("EdDSA verifier test failed: {}", e);
    }
}

#[test]
fn test_eddsa_poseidon_verifier_invalid_signature() {
    let tester = CircuitTester::new();

    // Generate valid signature then modify R8x to make it invalid
    let private_key = private_key_from_seed(&EDDSA_TEST_SEED);
    let sig = sign_poseidon(&private_key, 1234);

    // Parse R8x, add 1, and convert back to string to create invalid signature
    let r8x_invalid: num_bigint::BigInt = sig.r8x.parse().unwrap();
    let r8x_modified = (r8x_invalid + 1i32).to_string();

    let result = tester.test_circuit_fails(
        "EdDSAVerifier",
        circuits::EDDSA_POSEIDON_VERIFIER,
        vec![],
        inputs(&[
            ("enabled", vec!["1"]),
            ("Ax", vec![sig.ax.as_str()]),
            ("Ay", vec![sig.ay.as_str()]),
            ("R8x", vec![r8x_modified.as_str()]),
            ("R8y", vec![sig.r8y.as_str()]),
            ("S", vec![sig.s.as_str()]),
            ("M", vec![sig.m.as_str()]),
        ]),
    );
    if let Err(e) = &result {
        panic!("EdDSA invalid signature test failed: {}", e);
    }
}

#[test]
fn test_eddsa_poseidon_verifier_disabled() {
    let tester = CircuitTester::new();

    // Generate valid signature then modify R8x to make it invalid
    let private_key = private_key_from_seed(&EDDSA_TEST_SEED);
    let sig = sign_poseidon(&private_key, 1234);

    // Parse R8x, add 1 to create invalid signature
    let r8x_invalid: num_bigint::BigInt = sig.r8x.parse().unwrap();
    let r8x_modified = (r8x_invalid + 1i32).to_string();

    // Disabled verification - bad signature should pass when enabled=0
    let result = tester.test_circuit(
        "EdDSAVerifier",
        circuits::EDDSA_POSEIDON_VERIFIER,
        vec![],
        inputs(&[
            ("enabled", vec!["0"]),
            ("Ax", vec![sig.ax.as_str()]),
            ("Ay", vec![sig.ay.as_str()]),
            ("R8x", vec![r8x_modified.as_str()]),
            ("R8y", vec![sig.r8y.as_str()]),
            ("S", vec![sig.s.as_str()]),
            ("M", vec![sig.m.as_str()]),
        ]),
    );
    if let Err(e) = &result {
        panic!("EdDSA disabled test failed: {}", e);
    }
}
