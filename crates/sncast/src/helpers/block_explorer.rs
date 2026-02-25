use crate::{Network, response::explorer_link::ExplorerError};
use conversions::padded_felt::PaddedFelt;
use serde::{Deserialize, Serialize};

const VOYAGER: &str = "voyager.online";
const VIEWBLOCK: &str = "https://viewblock.io/starknet";
const OKLINK: &str = "https://www.oklink.com/starknet";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Service {
    #[default]
    Voyager,
    StarkScan,
    ViewBlock,
    OkLink,
}

impl Service {
    pub fn validate_for_config(block_explorer: Option<Self>) -> anyhow::Result<()> {
        if block_explorer.unwrap_or_default() == Self::StarkScan {
            anyhow::bail!(
                "starkscan.co was terminated and `'StarkScan'` is no longer available. Please set `block-explorer` to `'Voyager'` or other explorer of your choice."
            );
        }
        Ok(())
    }

    pub fn as_provider(&self, network: Network) -> Result<Box<dyn LinkProvider>, ExplorerError> {
        match (self, network) {
            (Service::StarkScan, _) => unreachable!("Should be caught by config validation"),
            (Service::Voyager, _) => Ok(Box::new(Voyager { network })),
            (Service::ViewBlock, Network::Mainnet) => Ok(Box::new(ViewBlock)),
            (Service::OkLink, Network::Mainnet) => Ok(Box::new(OkLink)),
            (_, Network::Sepolia) => Err(ExplorerError::SepoliaNotSupported),
            (_, Network::Devnet) => Err(ExplorerError::DevnetNotSupported),
        }
    }
}

pub trait LinkProvider {
    fn transaction(&self, hash: PaddedFelt) -> String;
    fn class(&self, hash: PaddedFelt) -> String;
    fn contract(&self, address: PaddedFelt) -> String;
}

const fn network_subdomain(network: Network) -> &'static str {
    match network {
        Network::Sepolia => "sepolia.",
        Network::Mainnet | Network::Devnet => "",
    }
}

pub struct Voyager {
    network: Network,
}

impl LinkProvider for Voyager {
    fn transaction(&self, hash: PaddedFelt) -> String {
        format!(
            "https://{}{VOYAGER}/tx/{hash:#x}",
            network_subdomain(self.network)
        )
    }

    fn class(&self, hash: PaddedFelt) -> String {
        format!(
            "https://{}{VOYAGER}/class/{hash:#x}",
            network_subdomain(self.network)
        )
    }

    fn contract(&self, address: PaddedFelt) -> String {
        format!(
            "https://{}{VOYAGER}/contract/{address:#x}",
            network_subdomain(self.network)
        )
    }
}

pub struct ViewBlock;

impl LinkProvider for ViewBlock {
    fn transaction(&self, hash: PaddedFelt) -> String {
        format!("{VIEWBLOCK}/tx/{hash:#x}")
    }

    fn class(&self, hash: PaddedFelt) -> String {
        format!("{VIEWBLOCK}/class/{hash:#x}")
    }

    fn contract(&self, address: PaddedFelt) -> String {
        format!("{VIEWBLOCK}/contract/{address:#x}")
    }
}

pub struct OkLink;

impl LinkProvider for OkLink {
    fn transaction(&self, hash: PaddedFelt) -> String {
        format!("{OKLINK}/tx/{hash:#x}")
    }

    fn class(&self, hash: PaddedFelt) -> String {
        format!("{OKLINK}/class/{hash:#x}")
    }

    fn contract(&self, address: PaddedFelt) -> String {
        format!("{OKLINK}/contract/{address:#x}")
    }
}

#[cfg(test)]
mod tests {
    use crate::response::deploy::DeployResponse;
    use crate::{
        Network,
        helpers::block_explorer::Service,
        response::{deploy::StandardDeployResponse, explorer_link::OutputLink},
    };
    use conversions::padded_felt::PaddedFelt;
    use regex::Regex;
    use starknet_rust::macros::felt;
    use test_case::test_case;

    const MAINNET_RESPONSE: DeployResponse = DeployResponse::Standard(StandardDeployResponse {
        contract_address: PaddedFelt(felt!(
            "0x03241d40a2af53a34274dd411e090ccac1ea80e0380a0303fe76d71b046cfecb"
        )),
        transaction_hash: PaddedFelt(felt!(
            "0x7605291e593e0c6ad85681d09e27a601befb85033bdf1805aabf5d84617cf68"
        )),
    });

    const SEPOLIA_RESPONSE: DeployResponse = DeployResponse::Standard(StandardDeployResponse {
        contract_address: PaddedFelt(felt!(
            "0x0716b5f1e3bd760c489272fd6530462a09578109049e26e3f4c70492676eae17"
        )),
        transaction_hash: PaddedFelt(felt!(
            "0x1cde70aae10f79d2d1289c923a1eeca7b81a2a6691c32551ec540fa2cb29c33"
        )),
    });

    async fn assert_valid_links(input: &str) {
        let pattern = Regex::new(r"transaction: |contract: |class: ").unwrap();
        let links = pattern.replace_all(input, "");
        let mut links = links.split('\n');

        let contract = links.next().unwrap();
        let transaction = links.next().unwrap();

        let (contract_response, transaction_response) =
            tokio::join!(reqwest::get(contract), reqwest::get(transaction));

        assert!(contract_response.is_ok());
        assert!(transaction_response.is_ok());
    }

    #[test_case(Network::Mainnet, &MAINNET_RESPONSE; "mainnet")]
    #[test_case(Network::Sepolia, &SEPOLIA_RESPONSE; "sepolia")]
    #[tokio::test]
    async fn test_happy_case_voyager(network: Network, response: &DeployResponse) {
        let provider = Service::Voyager.as_provider(network).unwrap();
        let result = response.format_links(provider);
        assert_valid_links(&result).await;
    }

    #[test_case(Service::ViewBlock; "viewblock")]
    #[test_case(Service::OkLink; "oklink")]
    #[tokio::test]
    async fn test_happy_case_mainnet(explorer: Service) {
        let result = MAINNET_RESPONSE.format_links(explorer.as_provider(Network::Mainnet).unwrap());
        assert_valid_links(&result).await;
    }

    #[test_case(Service::ViewBlock; "viewblock")]
    #[test_case(Service::OkLink; "oklink")]
    #[tokio::test]
    async fn test_fail_on_sepolia(explorer: Service) {
        assert!(explorer.as_provider(Network::Sepolia).is_err());
    }
}
