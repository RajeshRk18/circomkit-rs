//! Signal creation utilities

use crate::types::{CircuitSignals, SignalValue};

/// Create a circuit signals map from key-value pairs
///
/// # Example
///
/// ```
/// use circomkit::utils::signals;
///
/// let inputs = signals([
///     ("a", 3.into()),
///     ("b", 5.into()),
/// ]);
/// ```
pub fn signals<const N: usize>(pairs: [(&str, SignalValue); N]) -> CircuitSignals {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// Create a signal array from a slice of values
///
/// # Example
///
/// ```
/// use circomkit::utils::signal_array;
///
/// let arr = signal_array(&[1, 2, 3, 4, 5]);
/// ```
pub fn signal_array<T: ToString>(values: &[T]) -> SignalValue {
    SignalValue::Array(
        values
            .iter()
            .map(|v| SignalValue::Single(v.to_string()))
            .collect(),
    )
}

/// Builder for creating circuit signals
#[derive(Debug, Default)]
pub struct SignalBuilder {
    signals: CircuitSignals,
}

impl SignalBuilder {
    /// Create a new signal builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single numeric signal
    pub fn add<T: ToString>(mut self, name: &str, value: T) -> Self {
        self.signals
            .insert(name.to_string(), SignalValue::Single(value.to_string()));
        self
    }

    /// Add an array signal
    pub fn add_array<T: ToString>(mut self, name: &str, values: &[T]) -> Self {
        self.signals.insert(name.to_string(), signal_array(values));
        self
    }

    /// Add a 2D array signal
    pub fn add_2d_array<T: ToString>(mut self, name: &str, values: &[Vec<T>]) -> Self {
        let arr = SignalValue::Array(
            values
                .iter()
                .map(|row| {
                    SignalValue::Array(
                        row.iter()
                            .map(|v| SignalValue::Single(v.to_string()))
                            .collect(),
                    )
                })
                .collect(),
        );
        self.signals.insert(name.to_string(), arr);
        self
    }

    /// Build the circuit signals
    pub fn build(self) -> CircuitSignals {
        self.signals
    }
}

/// Macro for creating circuit signals
///
/// # Example
///
/// ```ignore
/// use circomkit::signals;
///
/// let inputs = signals! {
///     "a" => 3,
///     "b" => 5,
///     "arr" => [1, 2, 3]
/// };
/// ```
#[macro_export]
macro_rules! signals {
    ($($name:expr => $value:expr),* $(,)?) => {{
        use $crate::types::SignalValue;
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($name.to_string(), SignalValue::from($value));
        )*
        map
    }};
}

/// Parse signals from a JSON string
pub fn parse_signals(json: &str) -> Result<CircuitSignals, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize signals to a JSON string
pub fn serialize_signals(signals: &CircuitSignals) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(signals)
}

/// Convert field element string to bytes (big-endian)
pub fn field_to_bytes(value: &str) -> Vec<u8> {
    // Handle hex strings
    if value.starts_with("0x") {
        return hex::decode(&value[2..]).unwrap_or_default();
    }

    // Handle decimal strings
    // This is a simplified implementation
    if let Ok(n) = value.parse::<u128>() {
        return n.to_be_bytes().to_vec();
    }

    // For larger numbers, we'd need a big integer library
    Vec::new()
}

/// Convert bytes to field element string
pub fn bytes_to_field(bytes: &[u8]) -> String {
    if bytes.len() <= 16 {
        // Can fit in u128
        let mut padded = [0u8; 16];
        padded[16 - bytes.len()..].copy_from_slice(bytes);
        u128::from_be_bytes(padded).to_string()
    } else {
        // Return as hex for larger values
        format!("0x{}", hex::encode(bytes))
    }
}

/// Hash a message and return as a field element string
pub fn hash_to_field(message: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(message);
    bytes_to_field(&hash[..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signals_macro() {
        let signals = signals! {
            "a" => 3_i64,
            "b" => 5_i64,
        };

        assert_eq!(signals.len(), 2);
        assert!(signals.contains_key("a"));
        assert!(signals.contains_key("b"));
    }

    #[test]
    fn test_signal_builder() {
        let signals = SignalBuilder::new()
            .add("a", 3)
            .add("b", 5)
            .add_array("arr", &[1, 2, 3])
            .build();

        assert_eq!(signals.len(), 3);
        assert!(signals.contains_key("a"));
        assert!(signals.contains_key("b"));
        assert!(signals.contains_key("arr"));
    }

    #[test]
    fn test_signal_array() {
        let arr = signal_array(&[1, 2, 3]);
        if let SignalValue::Array(values) = arr {
            assert_eq!(values.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_field_conversions() {
        let bytes = field_to_bytes("12345");
        let back = bytes_to_field(&bytes);
        assert_eq!(back, "12345");
    }
}
