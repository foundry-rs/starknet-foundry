use starknet::core::types::contract::AbiEntry;
use starknet::core::types::{BlockId, BlockTag, ContractClass};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;
use tokio::sync::OnceCell;
use url::Url;

mod identity;
mod reverse_transformer;
mod transformer;

/// Class hash of the declared `DataTransformer` contract from `/tests/data/data_transformer`
const TEST_CLASS_HASH: Felt =
    Felt::from_hex_unchecked("0x071b56ab58087fd00a0b4ddcdfecb727ae11d1674a4a0f5af7c30f9bb2f7150e");

/// Class hash of the declared `DataTransformerNoConstructor` contract from `/tests/data/data_transformer`
const NO_CONSTRUCTOR_CLASS_HASH: Felt =
    Felt::from_hex_unchecked("0x051d0347d3bfcd87eea5175994f55158a24b003370d8c83d2c430f663eceb08d");

static CLASS: OnceCell<ContractClass> = OnceCell::const_new();

async fn init_class(class_hash: Felt) -> ContractClass {
    let client = JsonRpcClient::new(HttpTransport::new(
        Url::parse("http://188.34.188.184:7070/rpc/v0_8").unwrap(),
    ));

    client
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
        .unwrap()
}

async fn get_abi() -> Vec<AbiEntry> {
    let class = CLASS.get_or_init(|| init_class(TEST_CLASS_HASH)).await;
    let ContractClass::Sierra(sierra_class) = class else {
        panic!("Expected Sierra class, but got legacy Sierra class")
    };

    serde_json::from_str(sierra_class.abi.as_str()).unwrap()
}
