use serde::{Deserialize, Serialize};
use starknet_crypto::FieldElement;

const STARKSCAN: &str = "https://starkscan.co/search";
const VOYAGER: &str = "https://voyager.online";
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
    pub fn as_provider(&self) -> Box<dyn LinkProvider> {
        match self {
            Service::StarkScan => Box::new(StarkScan),
            Service::Voyager => Box::new(Voyager),
            Service::ViewBlock => Box::new(ViewBlock),
            Service::OkLink => Box::new(OkLink),
            Service::NftScan => Box::new(NftScan),
        }
    }
}

pub trait LinkProvider {
    fn transaction(&self, hash: FieldElement) -> String;
    fn class(&self, hash: FieldElement) -> String;
    fn contract(&self, address: FieldElement) -> String;
}

pub struct StarkScan;

impl LinkProvider for StarkScan {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{STARKSCAN}/{hash:x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{STARKSCAN}/{hash:x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{STARKSCAN}/{address:x}")
    }
}

pub struct Voyager;

impl LinkProvider for Voyager {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{VOYAGER}/tx/{hash:x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{VOYAGER}/class/{hash:x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{VOYAGER}/contract/{address:x}")
    }
}

pub struct ViewBlock;

impl LinkProvider for ViewBlock {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{VIEWBLOCK}/tx/{hash:x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{VIEWBLOCK}/class/{hash:x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{VIEWBLOCK}/contract/{address:x}")
    }
}

pub struct OkLink;

impl LinkProvider for OkLink {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{OKLINK}/tx/{hash:x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{OKLINK}/class/{hash:x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{OKLINK}/contract/{address:x}")
    }
}

pub struct NftScan;

impl LinkProvider for NftScan {
    fn transaction(&self, hash: FieldElement) -> String {
        format!("{NFTSCAN}/{hash:x}")
    }

    fn class(&self, hash: FieldElement) -> String {
        format!("{NFTSCAN}/{hash:x}")
    }

    fn contract(&self, address: FieldElement) -> String {
        format!("{NFTSCAN}/{address:x}")
    }
}
