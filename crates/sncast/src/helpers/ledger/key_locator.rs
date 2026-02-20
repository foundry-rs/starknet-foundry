use clap::Args;
use starknet_rust::signers::DerivationPath;

use crate::response::ui::UI;

use super::hd_path::{DerivationPathParser, ParsedDerivationPath, account_id_to_derivation_path};

#[derive(Args, Debug)]
#[group(multiple = false, required = true)]
pub struct LedgerKeyLocator {
    /// Ledger derivation path in EIP-2645 format
    #[arg(long, value_parser = DerivationPathParser)]
    pub path: Option<ParsedDerivationPath>,

    /// Account index, expands to "m//starknet'/sncast'/0'/<account-id>'/0"
    #[arg(long)]
    pub account_id: Option<u32>,
}

impl LedgerKeyLocator {
    #[must_use]
    pub fn resolve(&self, ui: &UI) -> DerivationPath {
        // This expect is unreachable - clap's `required = true` group guarantees
        // that at least one of `--path` or `--account-id` is provided
        resolve_key_locator(self.path.as_ref(), self.account_id, ui)
            .expect("clap requires one of --path or --account-id")
    }
}

fn resolve_key_locator(
    path: Option<&ParsedDerivationPath>,
    account_id: Option<u32>,
    ui: &UI,
) -> Option<DerivationPath> {
    if let Some(parsed) = path {
        parsed.print_warnings(ui);
        Some(parsed.path.clone())
    } else {
        account_id.map(account_id_to_derivation_path)
    }
}
