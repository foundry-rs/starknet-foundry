use starknet::core::types::{BlockId, BlockTag, ContractClass};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;
use tokio::sync::OnceCell;
use url::Url;

mod identity;
mod reverse_transformer;
mod transformer;

// Class hash of the declared contract from /tests/data/data_transformer
const TEST_CLASS_HASH: Felt =
    Felt::from_hex_unchecked("0x02978c91b2c3d47cba2103d40280a3601b90ed93a59cebc2ad61c6d1dab5e10a");

static CLASS: OnceCell<ContractClass> = OnceCell::const_new();

async fn init_class() -> ContractClass {
    let client = JsonRpcClient::new(HttpTransport::new(
        Url::parse("http://188.34.188.184:7070/rpc/v0_8").unwrap(),
    ));

    client
        .get_class(BlockId::Tag(BlockTag::Latest), TEST_CLASS_HASH)
        .await
        .unwrap()
}
