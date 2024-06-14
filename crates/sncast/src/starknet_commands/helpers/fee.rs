use clap::Args;
use starknet::core::types::FieldElement;

#[derive(Args, Debug)]
pub struct Fee {
    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,
}
