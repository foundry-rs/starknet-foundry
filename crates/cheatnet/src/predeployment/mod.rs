use anyhow::Result;
use flate2::read::GzDecoder;
use std::io::Read;

pub mod contracts_data;
pub mod erc20;
pub mod predeployed_contract;

pub fn load_gzipped_artifact(bytes: &[u8]) -> Result<String> {
    let mut decoder = GzDecoder::new(bytes);
    let mut artifact = String::new();
    decoder.read_to_string(&mut artifact)?;

    Ok(artifact)
}
