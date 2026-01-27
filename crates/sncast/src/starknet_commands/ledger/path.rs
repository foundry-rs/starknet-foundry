use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use starknet_rust::signers::DerivationPath;

/// Hash a string to a 31-bit unsigned integer as per EIP-2645 standard
///
/// The EIP-2645 standard specifies that layer and application names should be
/// hashed using SHA256, and the result should be the 31 lowest bits of the hash.
/// This means we take the last 4 bytes (as they represent the lowest bits) and
/// mask to 31 bits.
fn hash_to_u31(s: &str) -> u32 {
    let hash = Sha256::digest(s.as_bytes());
    // Take the last 4 bytes (lowest bits) of the hash
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&hash[hash.len() - 4..]);
    // Interpret as big-endian u32 and mask to 31 bits
    let value = u32::from_be_bytes(bytes);
    value & ((1 << 31) - 1) // Take 31 lowest bits
}

/// Preprocess extended path format to standard EIP-2645 format
///
/// This function supports Starkli-style extensions:
/// 1. Omitting the `2645'` prefix: `m//...` becomes `m/2645'/...`
/// 2. Using string names: `starknet'` becomes `1195502025'` (SHA256 hash)
///
/// # Examples
///
/// ```
/// # use sncast::starknet_commands::ledger::path::preprocess_path;
/// let extended = "m//starknet'/sncast'/0'/0'/0'";
/// let standard = preprocess_path(extended).unwrap();
/// assert!(standard.starts_with("m/2645'/"));
/// ```
pub fn preprocess_path(path: &str) -> Result<String> {
    // Handle m// prefix (omitted 2645)
    let path = if path.starts_with("m//") {
        path.replacen("m//", "m/2645'/", 1)
    } else {
        path.to_string()
    };

    // Parse and convert string segments
    let parts: Vec<&str> = path.split('/').collect();
    let mut result = Vec::new();

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            // "m" prefix
            result.push((*part).to_string());
            continue;
        }

        // Check if hardened (ends with ')
        let (value, hardened) = if part.ends_with('\'') {
            (&part[..part.len() - 1], true)
        } else {
            (*part, false)
        };

        // Try to parse as number, otherwise hash the string
        let number = if let Ok(n) = value.parse::<u32>() {
            n
        } else if value.is_empty() {
            return Err(anyhow::anyhow!("Empty path segment in: {}", path));
        } else {
            hash_to_u31(value)
        };

        result.push(if hardened {
            format!("{number}'")
        } else {
            number.to_string()
        });
    }

    Ok(result.join("/"))
}

/// Parse derivation path with extension support
///
/// This function accepts both standard EIP-2645 paths and Starkli-style extensions.
///
/// # Examples
///
/// ```
/// # use sncast::starknet_commands::ledger::path::parse_derivation_path;
/// // Extended format
/// let path1 = parse_derivation_path("m//starknet'/sncast'/0'/0'/0'").unwrap();
///
/// // Standard format
/// let path2 = parse_derivation_path("m/2645'/1195502025'/1470455285'/0'/0'/0'").unwrap();
/// ```
pub fn parse_derivation_path(path: &str) -> Result<DerivationPath> {
    let canonical = preprocess_path(path)?;
    canonical
        .parse()
        .with_context(|| format!("Failed to parse derivation path '{path}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_to_u31_starknet() {
        // This value is from the EIP-2645 standard and starkli implementation
        assert_eq!(hash_to_u31("starknet"), 1195502025);
    }

    #[test]
    fn test_hash_to_u31_sncast() {
        // Calculate hash for "sncast"
        let hash = hash_to_u31("sncast");
        // Verify it's a 31-bit number
        assert!(hash < (1 << 31));
        println!("Hash of 'sncast': {hash}");
    }

    #[test]
    fn test_preprocess_path_with_omitted_2645() {
        let input = "m//starknet'/sncast'/0'/0'/0'";
        let output = preprocess_path(input).unwrap();
        assert!(output.starts_with("m/2645'/"));
        assert!(output.contains("1195502025'")); // starknet hash
    }

    #[test]
    fn test_preprocess_path_with_strings() {
        let input = "m/2645'/starknet'/sncast'/0'/0'/0'";
        let output = preprocess_path(input).unwrap();
        assert!(output.contains("1195502025'")); // starknet hash
        // sncast hash will be calculated
        assert!(!output.contains("sncast"));
    }

    #[test]
    fn test_numeric_path_unchanged() {
        let input = "m/2645'/1195502025'/1470455285'/0'/0'/0'";
        let output = preprocess_path(input).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_mixed_format() {
        let input = "m/2645'/starknet'/1470455285'/0'/0'/0'";
        let output = preprocess_path(input).unwrap();
        assert!(output.contains("1195502025'")); // starknet converted
        assert!(output.contains("1470455285'")); // numeric preserved
    }

    #[test]
    fn test_non_hardened_segments() {
        let input = "m/2645'/1195502025'/1470455285'/0/0/0";
        let output = preprocess_path(input).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_empty_segment_error() {
        let input = "m/2645'//0'/0'/0'";
        let result = preprocess_path(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_full_path_parsing() {
        // Test that the full parsing works
        let path = parse_derivation_path("m//starknet'/sncast'/0'/0'/0'");
        assert!(path.is_ok());
    }
}
