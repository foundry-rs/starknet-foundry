use serde::{Deserialize, Serialize};
use starknet_crypto::FieldElement;

use crate::{response::explorer_link::ExplorerError, Network};

const STARKSCAN: &str = "starkscan.co/search";
const VOYAGER: &str = "voyager.online";
const VIEWBLOCK: &str = "https://viewblock.io/starknet";
const OKLINK: &str = "https://www.oklink.com/starknet";
const NFTSCAN: &str = "https://starknet.nftscan.com";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Service {
    #[default]
    StarkScan,
    Voyager,
    ViewBlock,
    OkLink,
    NftScan,
}

impl Service {
    #[must_use]
    pub fn as_provider(&self, network: Network) -> Result<Box<dyn LinkProvider>, ExplorerError> {
        match (self, network) {
            (Service::StarkScan, _) => Ok(Box::new(StarkScan { network })),
            (Service::Voyager, _) => Ok(Box::new(Voyager { network })),
            (Service::ViewBlock, Network::Mainnet) => Ok(Box::new(ViewBlock)),
            (Service::OkLink, Network::Mainnet) => Ok(Box::new(OkLink)),
            (Service::NftScan, Network::Mainnet) => Ok(Box::new(NftScan)),
            (_, Network::Sepolia) => Err(ExplorerError::SepoliaNotSupported),
        }
    }
}

pub trait LinkProvider {
    fn transaction(&self, hash: FieldElement) -> String;
    fn class(&self, hash: FieldElement) -> String;
    fn contract(&self, address: FieldElement) -> String;
}

const fn network_mixin(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "",
        Network::Sepolia => "sepolia.",
    }
}

pub struct StarkScan {
    network: Network,
}

impl LinkProvider for StarkScan {
    fn transaction(&self, hash: FieldElement) -> String {
        format!(
            "https://{}{STARKSCAN}/{hash:#x}",
            network_mixin(self.network)
        )
    }

    fn class(&self, hash: FieldElement) -> String {
        format!(
            "https://{}{STARKSCAN}/{hash:#x}",
            network_mixin(self.network)
        )
    }

    fn contract(&self, address: FieldElement) -> String {
        format!(
            "https://{}{STARKSCAN}/{address:#x}",
            network_mixin(self.network)
        )
    }
}

pub struct Voyager {
    network: Network,
}

impl LinkProvider for Voyager {
    fn transaction(&self, hash: FieldElement) -> String {
        format!(
            "https://{}{VOYAGER}/tx/{hash:#x}",
            network_mixin(self.network)
        )
    }

    fn class(&self, hash: FieldElement) -> String {
        format!(
            "https://{}{VOYAGER}/class/{hash:#x}",
            network_mixin(self.network)
        )
    }

    fn contract(&self, address: FieldElement) -> String {
        format!(
            "https://{}{VOYAGER}/contract/{address:#x}",
            network_mixin(self.network)
        )
    }
}

pub struct ViewBlock;

impl LinkProvider for ViewBlock {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{VIEWBLOCK}/tx/{hash:#x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{VIEWBLOCK}/class/{hash:#x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{VIEWBLOCK}/contract/{address:#x}")
    }
}

pub struct OkLink;

impl LinkProvider for OkLink {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{OKLINK}/tx/{hash:#x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{OKLINK}/class/{hash:#x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{OKLINK}/contract/{address:#x}")
    }
}

pub struct NftScan;

impl LinkProvider for NftScan {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{NFTSCAN}/{hash:#x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{NFTSCAN}/{hash:#x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{NFTSCAN}/{address:#x}")
    }
}
