use crate::helpers::constants::{BRAAVOS_BASE_ACCOUNT_CLASS_HASH, KEYSTORE_PASSWORD_ENV_VAR};
use anyhow::{anyhow, ensure, Context};
use camino::Utf8PathBuf;
use clap::ValueEnum;
use conversions::string::IntoHexStr;
use serde::{Deserialize, Serialize, Serializer};
use starknet::core::types::FieldElement;
use starknet::signers::SigningKey;
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct AccountKeystore {
    pub version: u64,
    pub variant: AccountVariant,
    pub deployment: DeploymentStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountVariant {
    OpenZeppelin(OzAccountConfig),
    Argent(ArgentAccountConfig),
    Braavos(BraavosAccountConfig),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeploymentStatus {
    Undeployed(UndeployedStatus),
    Deployed(DeployedStatus),
}

pub enum AccountVariantType {
    OpenZeppelinLegacy,
    ArgentLegacy,
    BraavosLegacy,
    Argent,
    Braavos,
    OpenZeppelin,
}

#[derive(Serialize, Deserialize)]
pub struct OzAccountConfig {
    pub version: u64,
    #[serde(serialize_with = "serialize_field_element")]
    pub public_key: FieldElement,
    #[serde(default = "default_legacy")]
    pub legacy: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ArgentAccountConfig {
    pub version: u64,
    #[serde(
        serialize_with = "serialize_field_element_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub implementation: Option<FieldElement>,
    #[serde(serialize_with = "serialize_field_element")]
    pub owner: FieldElement,
    #[serde(serialize_with = "serialize_field_element")]
    pub guardian: FieldElement,
}

#[derive(Serialize, Deserialize)]
pub struct BraavosAccountConfig {
    pub version: u64,
    #[serde(
        serialize_with = "serialize_field_element_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub implementation: Option<FieldElement>,
    pub multisig: BraavosMultisigConfig,
    pub signers: Vec<BraavosSigner>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BraavosMultisigConfig {
    On { num_signers: usize },
    Off,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BraavosSigner {
    Stark(BraavosStarkSigner),
}

#[derive(Serialize, Deserialize)]
pub struct BraavosStarkSigner {
    #[serde(serialize_with = "serialize_field_element")]
    pub public_key: FieldElement,
}

#[derive(Serialize, Deserialize)]
pub struct UndeployedStatus {
    #[serde(serialize_with = "serialize_field_element")]
    pub class_hash: FieldElement,
    #[serde(serialize_with = "serialize_field_element")]
    pub salt: FieldElement,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<DeploymentContext>,
}

#[derive(Serialize, Deserialize)]
pub struct DeployedStatus {
    #[serde(serialize_with = "serialize_field_element")]
    pub class_hash: FieldElement,
    #[serde(serialize_with = "serialize_field_element")]
    pub address: FieldElement,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "variant", rename_all = "snake_case")]
pub enum DeploymentContext {
    Braavos(BraavosDeploymentContext),
}

#[derive(Serialize, Deserialize)]
pub struct BraavosDeploymentContext {
    #[serde(serialize_with = "serialize_field_element")]
    pub base_account_class_hash: FieldElement,
}

impl TryFrom<&AccountData> for AccountKeystore {
    type Error = anyhow::Error;

    fn try_from(account_data: &AccountData) -> Result<Self, Self::Error> {
        let deployment = if account_data.deployed.unwrap_or(false) {
            DeploymentStatus::Deployed(DeployedStatus {
                class_hash: account_data
                    .class_hash
                    .context("Class hash must be known for conversion")?,
                address: account_data
                    .address
                    .context("Address must be known for conversion")?,
            })
        } else {
            DeploymentStatus::Undeployed(UndeployedStatus {
                class_hash: account_data
                    .class_hash
                    .context("Class hash must be known for conversion")?,
                salt: account_data
                    .salt
                    .context("Salt must be known for conversion")?,
                context: match account_data.account_type {
                    Some(AccountType::Braavos) => {
                        Some(DeploymentContext::Braavos(BraavosDeploymentContext {
                            base_account_class_hash: BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                        }))
                    }
                    _ => None,
                },
            })
        };

        let account = match account_data
            .account_type
            .clone()
            .context("Account type must be known for conversion")?
        {
            AccountType::Oz => AccountKeystore {
                version: 1,
                variant: AccountVariant::OpenZeppelin(OzAccountConfig {
                    version: 1,
                    public_key: account_data.public_key,
                    legacy: account_data.legacy.unwrap_or(true),
                }),
                deployment,
            },
            AccountType::Argent => AccountKeystore {
                version: 1,
                variant: AccountVariant::Argent(ArgentAccountConfig {
                    version: 1,
                    owner: account_data.public_key,
                    implementation: None,
                    guardian: FieldElement::ZERO,
                }),
                deployment,
            },
            AccountType::Braavos => AccountKeystore {
                version: 1,
                variant: AccountVariant::Braavos(BraavosAccountConfig {
                    version: 1,
                    implementation: None,
                    multisig: BraavosMultisigConfig::Off,
                    signers: vec![BraavosSigner::Stark(BraavosStarkSigner {
                        public_key: account_data.public_key,
                    })],
                }),
                deployment,
            },
        };
        Ok(account)
    }
}

#[allow(clippy::doc_markdown)]
#[derive(ValueEnum, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    #[serde(rename = "open_zeppelin")]
    /// OpenZeppelin account implementation
    Oz,
    /// Argent account implementation
    Argent,
    /// Braavos account implementation
    Braavos,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AccountType::Oz => write!(f, "open_zeppelin"),
            AccountType::Argent => write!(f, "argent"),
            AccountType::Braavos => write!(f, "braavos"),
        }
    }
}

impl FromStr for AccountType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s {
            "open_zeppelin" | "oz" => Ok(AccountType::Oz),
            "argent" => Ok(AccountType::Argent),
            "braavos" => Ok(AccountType::Braavos),
            account_type => Err(anyhow!("Invalid account type = {account_type}")),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccountData {
    #[serde(serialize_with = "serialize_field_element")]
    pub private_key: FieldElement,
    #[serde(serialize_with = "serialize_field_element")]
    pub public_key: FieldElement,
    #[serde(serialize_with = "serialize_field_element_option")]
    pub address: Option<FieldElement>,
    #[serde(
        serialize_with = "serialize_field_element_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub salt: Option<FieldElement>,
    pub deployed: Option<bool>,
    #[serde(serialize_with = "serialize_field_element_option")]
    pub class_hash: Option<FieldElement>,
    pub legacy: Option<bool>,

    #[serde(default, rename(serialize = "type", deserialize = "type"))]
    pub account_type: Option<AccountType>,
}

impl AccountData {
    pub fn from_keystore(
        keystore_path: &Utf8PathBuf,
        account: AccountKeystore,
    ) -> anyhow::Result<AccountData> {
        let private_key = SigningKey::from_keystore(
            keystore_path,
            crate::get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?.as_str(),
        )?
        .secret_scalar();

        let (address, class_hash, salt, deployed) = match account.deployment {
            DeploymentStatus::Undeployed(status) => (
                None,
                Some(status.class_hash),
                Some(status.salt),
                Some(false),
            ),
            DeploymentStatus::Deployed(status) => (
                Some(status.address),
                Some(status.class_hash),
                None,
                Some(true),
            ),
        };

        let (legacy, account_type, public_key) = match account.variant {
            AccountVariant::OpenZeppelin(config) => (
                Some(config.legacy),
                Some(AccountType::Oz),
                config.public_key,
            ),
            AccountVariant::Argent(config) => (None, Some(AccountType::Argent), config.owner),
            AccountVariant::Braavos(config) => (
                None,
                Some(AccountType::Braavos),
                get_braavos_account_public_key(&config)?,
            ),
        };

        Ok(AccountData {
            private_key,
            public_key,
            address,
            salt,
            deployed,
            class_hash,
            legacy,
            account_type,
        })
    }
}

fn get_braavos_account_public_key(config: &BraavosAccountConfig) -> anyhow::Result<FieldElement> {
    ensure!(
        matches!(config.multisig, BraavosMultisigConfig::Off),
        "Braavos accounts cannot be deployed with multisig on"
    );
    ensure!(
        config.signers.len() == 1,
        "Braavos accounts can only be deployed with one seed signer"
    );

    match config.signers.first().unwrap() {
        BraavosSigner::Stark(signer) => Ok(signer.public_key),
    }
}

fn default_legacy() -> bool {
    true
}

fn serialize_field_element<S>(value: &FieldElement, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.into_hex_string())
}

fn serialize_field_element_option<S>(
    value: &Option<FieldElement>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_str(&value.into_hex_string()),
        None => serializer.serialize_none(),
    }
}
