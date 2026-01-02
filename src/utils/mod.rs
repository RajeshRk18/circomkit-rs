//! Utility functions for Circomkit

pub mod eddsa;
mod ptau;
mod signals;

pub use eddsa::{
    generate_private_key, private_key_from_seed, sign_poseidon, sign_poseidon_bigint,
    verify_poseidon, EdDSATestInputs,
};
pub use ptau::{PtauInfo, download_ptau, get_recommended_ptau};
pub use signals::{signal_array, signals};
