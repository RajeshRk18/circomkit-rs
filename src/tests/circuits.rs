//! Circom circuit templates for testing
//!
//! This module contains circuit code as string constants for testing.

/// TODO: Circuits with 2D array inputs to be added.

/// Simple adder circuit
pub const ADDER: &str = r#"
pragma circom 2.0.0;

template Adder() {
    signal input a;
    signal input b;
    signal output sum;
    sum <== a + b;
}
"#;

/// Simple multiplier circuit
pub const MULTIPLIER: &str = r#"
pragma circom 2.0.0;

template Multiplier() {
    signal input a;
    signal input b;
    signal output product;
    product <== a * b;
}
"#;

/// N-input multiplier circuit
pub const MULTIPLIER_N: &str = r#"
pragma circom 2.0.0;

template MultiplierN(n) {
    signal input in[n];
    signal output out;
    
    signal intermediate[n];
    intermediate[0] <== in[0];
    for (var i = 1; i < n; i++) {
        intermediate[i] <== intermediate[i-1] * in[i];
    }
    out <== intermediate[n-1];
}
"#;

/// IsZero circuit
pub const IS_ZERO: &str = r#"
pragma circom 2.0.0;

template IsZero() {
    signal input in;
    signal output out;
    signal inv;
    inv <-- in != 0 ? 1/in : 0;
    out <== -in * inv + 1;
    in * out === 0;
}
"#;

/// IsEqual circuit
pub const IS_EQUAL: &str = r#"
pragma circom 2.0.0;

template IsZero() {
    signal input in;
    signal output out;
    signal inv;
    inv <-- in != 0 ? 1/in : 0;
    out <== -in * inv + 1;
    in * out === 0;
}

template IsEqual() {
    signal input in[2];
    signal output out;
    component isz = IsZero();
    isz.in <== in[1] - in[0];
    out <== isz.out;
}
"#;

/// ForceEqual circuit
pub const FORCE_EQUAL: &str = r#"
pragma circom 2.0.0;

template ForceEqual() {
    signal input a;
    signal input b;
    a === b;
}
"#;

/// Mux1 circuit
pub const MUX1: &str = r#"
pragma circom 2.0.0;

template Mux1() {
    signal input c[2];
    signal input s;
    signal output out;
    out <== c[0] + s * (c[1] - c[0]);
}
"#;

/// range check circuit
pub const RANGE_CHECK_8: &str = r#"
pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/bitify.circom";

template RangeCheck(n) {
    signal input in;
    component bits = Num2Bits(n);
    bits.in <== in;
}
"#;

/// 64-bit range check circuit
pub const RANGE_CHECK_64: &str = r#"
pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/bitify.circom";

template RangeCheck64() {
    signal input in;
    component bits = Num2Bits(64);
    bits.in <== in;
}
"#;

/// EdDSA Poseidon verifier circuit (wrapper for circomlib's EdDSAPoseidonVerifier)
pub const EDDSA_POSEIDON_VERIFIER: &str = r#"
pragma circom 2.1.9;

include "../node_modules/circomlib/circuits/eddsaposeidon.circom";

template EdDSAVerifier() {
    signal input enabled;
    signal input Ax;
    signal input Ay;
    signal input R8x;
    signal input R8y;
    signal input S;
    signal input M;

    component verifier = EdDSAPoseidonVerifier();
    verifier.enabled <== enabled;
    verifier.Ax <== Ax;
    verifier.Ay <== Ay;
    verifier.R8x <== R8x;
    verifier.R8y <== R8y;
    verifier.S <== S;
    verifier.M <== M;
}
"#;
