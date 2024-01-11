use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use clap::Args;
use sncast::get_nonce_for_tx;
use starknet::accounts::SingleOwnerAccount;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

#[derive(Args)]
#[command(about = "Show current configuration being used", long_about = None)]
pub struct ShowConfig {}

#[allow(clippy::ptr_arg)]
pub async fn wait_for_block(
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<()> {
    loop {
        let timeout: u8 = 30;
        let nonce_latest = get_nonce_for_tx(account, "latest", None).await;
        let nonce_pending = get_nonce_for_tx(account, "pending", None).await;
        if let Ok(nonce_latest) = nonce_latest {
            if let Ok(nonce_pending) = nonce_pending {
                if nonce_pending == nonce_latest {
                    break;
                }
            } else if let Err(message) = nonce_pending {
                println!("{message}");
            };
        } else if let Err(message) = nonce_latest {
            println!("{message}");
        }
        println!("Waiting...");
        sleep(Duration::from_secs(timeout.into()));
    }

    Ok(())
}
