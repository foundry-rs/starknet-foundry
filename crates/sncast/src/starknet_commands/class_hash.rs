use clap::Args;
use sncast::response::{class_hash::ClassHashResponse, errors::StarknetCommandError};

#[derive(Args)]
#[command(about = "Generate the class hash of a contract", long_about = None)]
struct ClassHash {
    /// Contract name
    #[arg(short = 'c', long = "contract-name")]
    pub contract: String,
}

pub async fn get_class_hash() -> Result<ClassHashResponse, StarknetCommandError> {
    
}
