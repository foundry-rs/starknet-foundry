use clap::Args;
use sncast::{
    helpers::rpc::RpcArgs,
    response::{errors::StarknetCommandError, serialize::SerializeResponse},
};
use starknet_types_core::felt::Felt;

#[derive(Args, Clone, Debug)]
#[command(about = "Serialize Cairo expressions into calldata")]
pub struct Serialize {
    /// Comma-separated string of Cairo expressions
    #[arg(long, allow_hyphen_values = true)]
    pub arguments: String,

    /// Address of contract which contains the function
    #[arg(short = 'd', long)]
    pub contract_address: Felt,

    /// Name of the function whose calldata should be serialized
    #[arg(short, long)]
    pub function: String,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[allow(clippy::unnecessary_wraps)]
pub fn serialize(calldata: Vec<Felt>) -> Result<SerializeResponse, StarknetCommandError> {
    Ok(SerializeResponse { calldata })
}
