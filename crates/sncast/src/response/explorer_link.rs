use crate::Network;
use crate::helpers::{block_explorer::LinkProvider, configuration::CastConfig, devnet::detection};
use foundry_ui::Message;
use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use serde_json::Value;
use starknet_types_core::felt::Felt;

const SNCAST_FORCE_SHOW_EXPLORER_LINKS_ENV: &str = "SNCAST_FORCE_SHOW_EXPLORER_LINKS";

// TODO(#3391): This code should be refactored to either use common `Message` trait or be directly
//  included in `sncast` output messages.
pub trait OutputLink {
    const TITLE: &'static str;

    fn get_links(&self, provider: Box<dyn LinkProvider>) -> Vec<ExplorerLink>;

    fn is_dry_run(&self) -> bool {
        false
    }
}

pub struct ExplorerLinksMessage {
    title: String,
    links: Vec<ExplorerLink>,
}

impl Serialize for ExplorerLinksMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Default, Serialize)]
        struct ExplorerLinksS<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            account: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            class: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            contract: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            declare_transaction: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            deploy_transaction: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            transaction: Option<&'a str>,
        }

        #[derive(Serialize)]
        struct ExplorerLinksMessageS<'a> {
            title: &'a str,
            links: ExplorerLinksS<'a>,
        }

        let mut links = ExplorerLinksS::default();

        for ExplorerLink { link_type, link } in &self.links {
            match link_type {
                ExplorerLinkType::Account => links.account = Some(link.as_str()),
                ExplorerLinkType::Class => links.class = Some(link.as_str()),
                ExplorerLinkType::Contract => links.contract = Some(link.as_str()),
                ExplorerLinkType::DeclareTransaction => {
                    links.declare_transaction = Some(link.as_str());
                }
                ExplorerLinkType::DeployTransaction => {
                    links.deploy_transaction = Some(link.as_str());
                }
                ExplorerLinkType::Transaction => links.transaction = Some(link.as_str()),
            }
        }

        ExplorerLinksMessageS {
            title: &self.title,
            links,
        }
        .serialize(serializer)
    }
}

impl ExplorerLinksMessage {
    pub fn new<T>(response: &T, provider: Box<dyn LinkProvider>) -> Self
    where
        T: OutputLink,
    {
        Self {
            title: T::TITLE.to_string(),
            links: response.get_links(provider),
        }
    }
}

impl Message for ExplorerLinksMessage {
    fn text(&self) -> String {
        let builder = OutputBuilder::new()
            .blank_line()
            .text_field(&format!("To see {} details, visit:", self.title));

        let builder = self
            .links
            .iter()
            .fold(builder, |acc, ExplorerLink { link_type, link }| {
                acc.field(link_type.as_str(), link)
            });

        builder.build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExplorerError {
    #[error("The chosen block explorer service is not available for Sepolia Network")]
    SepoliaNotSupported,
    #[error("Custom network is not recognized by block explorer service")]
    UnrecognizedNetwork,
    #[error("Block explorer service is not available for Devnet Network")]
    DevnetNotSupported,
}

pub async fn block_explorer_link_if_allowed<T>(
    result: &anyhow::Result<T>,
    chain_id: Felt,
    config: &CastConfig,
) -> Option<ExplorerLinksMessage>
where
    T: OutputLink + Clone,
{
    let Ok(response) = result else {
        return None;
    };

    if response.is_dry_run() {
        return None;
    }

    let network = chain_id.try_into().ok()?;

    let is_devnet = matches!(network, Network::Devnet) || detection::is_devnet_running().await;

    if (!config.show_explorer_links || is_devnet) && !is_explorer_link_overridden() {
        return None;
    }

    config
        .block_explorer
        .unwrap_or_default()
        .as_provider(network)
        .ok()
        .map(|provider| ExplorerLinksMessage::new(response, provider))
}

#[must_use]
pub fn is_explorer_link_overridden() -> bool {
    std::env::var(SNCAST_FORCE_SHOW_EXPLORER_LINKS_ENV).is_ok_and(|value| value == "1")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExplorerLinkType {
    Account,
    Class,
    Contract,
    DeclareTransaction,
    DeployTransaction,
    Transaction,
}

impl ExplorerLinkType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Account => "Account",
            Self::Class => "Class",
            Self::Contract => "Contract",
            Self::DeclareTransaction => "Declare Transaction",
            Self::DeployTransaction => "Deploy Transaction",
            Self::Transaction => "Transaction",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExplorerLink {
    #[serde(rename = "type")]
    link_type: ExplorerLinkType,
    link: String,
}

#[derive(Debug, Default)]
pub struct ExplorerLinkBuilder {
    links: Vec<ExplorerLink>,
}

impl ExplorerLinkBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn build(self) -> Vec<ExplorerLink> {
        self.links
    }

    #[must_use]
    pub fn account(self, link: String) -> Self {
        self.link(ExplorerLinkType::Account, link)
    }

    #[must_use]
    pub fn class(self, link: String) -> Self {
        self.link(ExplorerLinkType::Class, link)
    }

    #[must_use]
    pub fn contract(self, link: String) -> Self {
        self.link(ExplorerLinkType::Contract, link)
    }

    #[must_use]
    pub fn declare_transaction(self, link: String) -> Self {
        self.link(ExplorerLinkType::DeclareTransaction, link)
    }

    #[must_use]
    pub fn deploy_transaction(self, link: String) -> Self {
        self.link(ExplorerLinkType::DeployTransaction, link)
    }

    #[must_use]
    pub fn transaction(self, link: String) -> Self {
        self.link(ExplorerLinkType::Transaction, link)
    }

    fn link(mut self, link_type: ExplorerLinkType, link: String) -> Self {
        self.links.push(ExplorerLink { link_type, link });

        self
    }
}
