//! Error types for Circomkit-RS

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using CircomkitError
pub type Result<T> = std::result::Result<T, CircomkitError>;

/// Errors that can occur when using Circomkit
#[derive(Error, Debug)]
pub enum CircomkitError {
    /// Circuit file not found
    #[error("Circuit file not found: {0}")]
    CircuitNotFound(PathBuf),

    /// Circuit compilation failed
    #[error("Circuit compilation failed: {message}")]
    CompilationFailed {
        message: String,
        #[source]
        source: Option<std::io::Error>,
    },

    /// Witness generation failed
    #[error("Witness generation failed: {message}")]
    WitnessGenerationFailed { message: String },

    /// Proof generation failed
    #[error("Proof generation failed: {message}")]
    ProofGenerationFailed { message: String },

    /// Proof verification failed
    #[error("Proof verification failed: {message}")]
    VerificationFailed { message: String },

    /// Invalid circuit configuration
    #[error("Invalid circuit configuration: {0}")]
    InvalidConfig(String),

    /// PTAU file not found
    #[error("PTAU file not found: {0}")]
    PtauNotFound(PathBuf),

    /// Invalid input signals
    #[error("Invalid input signals: {0}")]
    InvalidSignals(String),

    /// Constraint not satisfied
    #[error("Constraint not satisfied: expected {expected}, got {actual}")]
    ConstraintNotSatisfied { expected: String, actual: String },

    /// External tool not found
    #[error("External tool not found: {tool}. Please ensure it is installed and in PATH")]
    ToolNotFound { tool: String },

    /// External command failed
    #[error("Command '{command}' failed with exit code {exit_code}: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

impl CircomkitError {
    /// Create a new compilation error
    pub fn compilation_failed(message: impl Into<String>) -> Self {
        Self::CompilationFailed {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new witness generation error
    pub fn witness_failed(message: impl Into<String>) -> Self {
        Self::WitnessGenerationFailed {
            message: message.into(),
        }
    }

    /// Create a new proof generation error
    pub fn proof_failed(message: impl Into<String>) -> Self {
        Self::ProofGenerationFailed {
            message: message.into(),
        }
    }

    /// Create a new verification error
    pub fn verification_failed(message: impl Into<String>) -> Self {
        Self::VerificationFailed {
            message: message.into(),
        }
    }

    /// Create a tool not found error
    pub fn tool_not_found(tool: impl Into<String>) -> Self {
        Self::ToolNotFound { tool: tool.into() }
    }
}
