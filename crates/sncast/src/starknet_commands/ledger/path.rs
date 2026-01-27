use std::str::FromStr;

use anyhow::{Result, anyhow, bail};
use clap::{Arg, Command, Error, builder::TypedValueParser, error::ErrorKind};
use sha2::{Digest, Sha256};
use starknet_rust::signers::DerivationPath;

const EIP2645_LENGTH: usize = 6;

/// BIP-32 encoding of `2645'`
const EIP_2645_PURPOSE: u32 = 0x8000_0a55;

#[derive(Clone)]
pub struct DerivationPathParser;

/// An EIP-2645 HD path, required by the Starknet Ledger app. This type allows users to write
/// hash-based segments in text, instead of manually finding out the lowest 31 bits of the hash.
///
/// The Ledger app requires that the path:
/// - starts with `2645'`; and
/// - has 6 levels
///
/// Supports shorthand `m//` to omit `2645'`:
///   `m//starknet'/sncast'/0'/0'/0`
#[derive(Debug, Clone)]
pub struct Eip2645Path {
    layer: Eip2645Level,
    application: Eip2645Level,
    eth_address_1: Eip2645Level,
    eth_address_2: Eip2645Level,
    index: Eip2645Level,
}

#[derive(Debug, Clone)]
enum Eip2645Level {
    Hash(HashLevel),
    Raw(u32),
}

#[derive(Debug, Clone)]
struct HashLevel {
    text: String,
    hardened: bool,
}

impl TypedValueParser for DerivationPathParser {
    type Value = DerivationPath;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        if value.is_empty() {
            return Err(cmd
                .clone()
                .error(ErrorKind::InvalidValue, "empty Ledger derivation path"));
        }
        match value.to_str() {
            Some(value) => match Eip2645Path::from_str(value) {
                Ok(path) => Ok(path.into()),
                Err(err) => Err(cmd.clone().error(
                    ErrorKind::InvalidValue,
                    format!("invalid Ledger derivation path: {err}"),
                )),
            },
            None => Err(cmd.clone().error(
                ErrorKind::InvalidValue,
                "invalid Ledger derivation path: not UTF-8",
            )),
        }
    }
}

impl FromStr for Eip2645Path {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Handle m// prefix (omitted 2645')
        let s = if s.starts_with("m//") {
            s.replacen("m//", "m/2645'/", 1)
        } else {
            s.to_string()
        };

        let segments: Vec<&str> = s.split('/').collect();
        if segments.len() != EIP2645_LENGTH + 1 {
            bail!("EIP-2645 paths must have {EIP2645_LENGTH} levels");
        }
        if segments[0] != "m" {
            bail!("HD wallet paths must start with \"m/\"");
        }

        if !segments[1].is_empty() {
            let prefix: Eip2645Level = segments[1].parse()?;
            if Into::<u32>::into(&prefix) != EIP_2645_PURPOSE {
                bail!("EIP-2645 paths must start with \"m/2645'/\"");
            }
        }

        let path = Self {
            layer: segments[2].parse()?,
            application: segments[3].parse()?,
            eth_address_1: segments[4].parse()?,
            eth_address_2: segments[5].parse()?,
            index: segments[6].parse()?,
        };

        if !path.layer.is_hardened() {
            bail!("the \"layer\" level of an EIP-2645 path must be hardened");
        }
        if !path.application.is_hardened() {
            bail!("the \"application\" level of an EIP-2645 path must be hardened");
        }
        if !path.eth_address_1.is_hardened() {
            bail!("the \"eth_address_1\" level of an EIP-2645 path must be hardened");
        }
        if !path.eth_address_2.is_hardened() {
            bail!("the \"eth_address_2\" level of an EIP-2645 path must be hardened");
        }

        if matches!(path.index, Eip2645Level::Hash(_)) {
            bail!("the \"index\" level must be a number");
        }

        Ok(path)
    }
}

impl Eip2645Level {
    fn is_hardened(&self) -> bool {
        match self {
            Self::Hash(hash) => hash.hardened,
            Self::Raw(raw) => raw & 0x8000_0000 > 0,
        }
    }
}

impl FromStr for Eip2645Level {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim() != s || s.split_whitespace().count() != 1 {
            bail!("path must not contain whitespaces");
        }

        let (body, harden_notation) = if s.ends_with('\'') {
            (&s[0..(s.len() - 1)], true)
        } else {
            (s, false)
        };

        if body.chars().all(|char| char.is_ascii_digit()) {
            let raw_node = body
                .parse::<u32>()
                .map_err(|err| anyhow!("invalid path level \"{body}\": {err}"))?;

            if harden_notation {
                if raw_node & 0x8000_0000 > 0 {
                    bail!("`'` appended to an already-hardened value of {raw_node}");
                }
                Ok(Self::Raw(raw_node | 0x8000_0000))
            } else {
                Ok(Self::Raw(raw_node))
            }
        } else {
            Ok(Self::Hash(HashLevel {
                text: body.to_owned(),
                hardened: harden_notation,
            }))
        }
    }
}

impl From<Eip2645Path> for DerivationPath {
    fn from(value: Eip2645Path) -> Self {
        vec![
            EIP_2645_PURPOSE,
            (&value.layer).into(),
            (&value.application).into(),
            (&value.eth_address_1).into(),
            (&value.eth_address_2).into(),
            (&value.index).into(),
        ]
        .into()
    }
}

impl From<&Eip2645Level> for u32 {
    fn from(value: &Eip2645Level) -> Self {
        match value {
            Eip2645Level::Hash(level) => {
                let mut hasher = Sha256::new();
                hasher.update(level.text.as_bytes());
                let hash = hasher.finalize();

                // Safe to unwrap: SHA256 output is fixed size
                let node = u32::from_be_bytes(hash.as_slice()[28..].try_into().unwrap());

                if level.hardened {
                    node | 0x8000_0000
                } else {
                    node & 0x7fff_ffff
                }
            }
            Eip2645Level::Raw(raw) => *raw,
        }
    }
}

/// Parse a derivation path string into a `DerivationPath`.
/// Accepts EIP-2645 paths with string segments and the `m//` shorthand.
pub fn parse_derivation_path(path: &str) -> Result<DerivationPath> {
    Ok(Eip2645Path::from_str(path)?.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_segment(s: &str) -> u32 {
        let level = Eip2645Level::Hash(HashLevel {
            text: s.to_owned(),
            hardened: true,
        });
        u32::from(&level) & 0x7fff_ffff
    }

    #[test]
    fn test_hash_starknet() {
        assert_eq!(hash_segment("starknet"), 1_195_502_025);
    }

    #[test]
    fn test_hash_bounded() {
        assert!(hash_segment("sncast") < (1 << 31));
    }

    #[test]
    fn test_path_with_omitted_2645() {
        let path = Eip2645Path::from_str("m//starknet'/sncast'/0'/0'/0").unwrap();
        let dp: DerivationPath = path.into();
        let s = dp.derivation_string();
        assert!(s.contains("2645'"));
    }

    #[test]
    fn test_path_with_explicit_2645() {
        let input = "m/2645'/starknet'/sncast'/0'/0'/0";
        let path = Eip2645Path::from_str(input).unwrap();
        let dp: DerivationPath = path.into();
        let s = dp.derivation_string();
        assert!(s.contains("2645'"));
        assert!(s.contains("1195502025'"));
    }

    #[test]
    fn test_numeric_path() {
        let input = "m/2645'/1195502025'/355113700'/0'/0'/0";
        let path = Eip2645Path::from_str(input).unwrap();
        let dp: DerivationPath = path.into();
        let s = dp.derivation_string();
        assert!(s.contains("1195502025'"));
        assert!(s.contains("355113700'"));
    }

    #[test]
    fn test_wrong_level_count_errors() {
        assert!(Eip2645Path::from_str("m/2645'/starknet'/sncast'/0'/0'").is_err());
    }

    #[test]
    fn test_unhardened_layer_errors() {
        assert!(Eip2645Path::from_str("m/2645'/1195502025/355113700'/0'/0'/0").is_err());
    }

    #[test]
    fn test_hash_index_errors() {
        assert!(Eip2645Path::from_str("m/2645'/starknet'/sncast'/0'/0'/myindex").is_err());
    }
}
