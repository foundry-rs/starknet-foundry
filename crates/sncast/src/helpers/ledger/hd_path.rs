/// Adapted from starkli: <https://github.com/xJonathanLEI/starkli/blob/1c1040ece107e8b371f0342b05c81e3fccf09086/src/hd_path.rs>
use std::str::FromStr;

use crate::response::ui::UI;
use anyhow::{Result, anyhow, bail};
use clap::{Arg, Command, Error, builder::TypedValueParser, error::ErrorKind};
use foundry_ui::components::warning::WarningMessage;
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
struct Eip2645Path {
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

impl Eip2645Path {
    fn from_str_with_warnings(s: &str) -> Result<(Self, Vec<String>)> {
        let path = s.parse::<Self>()?;
        let mut warnings = Vec::new();

        // These are allowed but we should serve a warning.
        match &path.eth_address_1 {
            Eip2645Level::Hash(_) => {
                warnings.push(
                    "using a non-numerical value for \"eth_address_1\" might make \
                    automatic key discovery difficult or impossible. Learn more at \
                    https://foundry-rs.github.io/starknet-foundry/starknet/eip-2645-hd-paths.html"
                        .to_string(),
                );
            }
            Eip2645Level::Raw(raw) => {
                if raw & 0x7fff_ffff != 0 {
                    warnings.push(
                        "using any value other than `0'` for \"eth_address_1\" might \
                        make automatic key discovery difficult or impossible. Learn more at \
                        https://foundry-rs.github.io/starknet-foundry/starknet/eip-2645-hd-paths.html"
                            .to_string(),
                    );
                }
            }
        }
        match &path.eth_address_2 {
            Eip2645Level::Hash(_) => {
                warnings.push(
                    "using a non-numerical value for \"eth_address_2\" might make \
                    automatic key discovery difficult or impossible. Learn more at \
                    https://foundry-rs.github.io/starknet-foundry/starknet/eip-2645-hd-paths.html"
                        .to_string(),
                );
            }
            Eip2645Level::Raw(raw) => {
                if raw & 0x7fff_ffff > 100 {
                    warnings.push(
                        "using a large value for \"eth_address_2\" might make \
                        automatic key discovery difficult. Learn more at \
                        https://foundry-rs.github.io/starknet-foundry/starknet/eip-2645-hd-paths.html"
                            .to_string(),
                    );
                }
            }
        }
        if path.index.is_hardened() {
            warnings.push(
                "hardening \"index\" is non-standard and it might make \
                automatic key discovery difficult or impossible. Learn more at \
                https://foundry-rs.github.io/starknet-foundry/starknet/eip-2645-hd-paths.html"
                    .to_string(),
            );
        }
        if u32::from(&path.index) & 0x7fff_ffff > 100 {
            warnings.push(
                "using a large value for \"index\" might make \
                automatic key discovery difficult. Learn more at \
                https://foundry-rs.github.io/starknet-foundry/starknet/eip-2645-hd-paths.html"
                    .to_string(),
            );
        }

        Ok((path, warnings))
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

        // These are not enforced by Ledger (for now) but are nice to have security properties
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

        // In the future, certain wallets might utilize sequential `index` values for key discovery,
        // so it might be a good idea for us to disallow using hash-based values for `index` here.
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

pub fn parse_derivation_path(path: &str, ui: &UI) -> Result<DerivationPath> {
    let (parsed, warnings) = Eip2645Path::from_str_with_warnings(path)?;

    for warning in warnings {
        ui.print_warning(WarningMessage::new(warning));
    }

    Ok(parsed.into())
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

    #[test]
    fn test_path_warnings() {
        let warnings_for = |path: &str| {
            Eip2645Path::from_str_with_warnings(path)
                .expect("path should be valid")
                .1
        };

        // Canonical path — no warnings expected
        assert!(warnings_for("m//starknet'/sncast'/0'/0'/0").is_empty());

        // eth_address_1: hash segment warns
        assert!(
            warnings_for("m//starknet'/sncast'/myapp'/0'/0")
                .iter()
                .any(|w| w.contains("eth_address_1"))
        );

        // eth_address_1: any non-zero raw value warns
        assert!(
            warnings_for("m//starknet'/sncast'/1'/0'/0")
                .iter()
                .any(|w| w.contains("eth_address_1"))
        );

        // eth_address_1: 0' is standard — no warning
        assert!(
            !warnings_for("m//starknet'/sncast'/0'/0'/0")
                .iter()
                .any(|w| w.contains("eth_address_1"))
        );

        // eth_address_2: hash segment warns
        assert!(
            warnings_for("m//starknet'/sncast'/0'/wallet'/0")
                .iter()
                .any(|w| w.contains("eth_address_2"))
        );

        // eth_address_2: value > 100 warns
        assert!(
            warnings_for("m//starknet'/sncast'/0'/101'/0")
                .iter()
                .any(|w| w.contains("eth_address_2"))
        );

        // eth_address_2: value <= 100 — no warning
        assert!(
            !warnings_for("m//starknet'/sncast'/0'/1'/0")
                .iter()
                .any(|w| w.contains("eth_address_2"))
        );

        // index: value > 100 warns
        assert!(
            warnings_for("m//starknet'/sncast'/0'/0'/101")
                .iter()
                .any(|w| w.contains("index"))
        );

        // index: value <= 100 — no warning
        assert!(
            !warnings_for("m//starknet'/sncast'/0'/0'/5")
                .iter()
                .any(|w| w.contains("index"))
        );
    }
}
