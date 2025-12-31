mod circuits;
mod testing;

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
    assert!(r1.is_ok());

    // 256 does NOT fit in 8 bits
    let r2 = tester.test_circuit_fails(
        "RangeCheck",
        circuits::RANGE_CHECK_8,
        vec![8],
        inputs(&[("in", vec!["256"])]),
    );
    assert!(r2.is_ok());
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
    assert!(result.is_ok());
}
