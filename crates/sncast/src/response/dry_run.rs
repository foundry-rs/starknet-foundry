use bigdecimal::BigDecimal;
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};
use starknet_rust::core::types::FeeEstimate;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Serialize, Deserialize, Debug, Clone, CairoSerialize, PartialEq, Eq)]
pub struct DryRunResponse {
    pub l1_gas_consumed: u64,
    pub l1_gas_price: u128,
    pub l2_gas_consumed: u64,
    pub l2_gas_price: u128,
    pub l1_data_gas_consumed: u64,
    pub l1_data_gas_price: u128,
    pub overall_fee: u128,
    pub detailed: bool,
}

impl DryRunResponse {
    #[must_use]
    pub fn new(fee_estimate: &FeeEstimate, detailed: bool) -> Self {
        Self {
            l1_gas_consumed: fee_estimate.l1_gas_consumed,
            l1_gas_price: fee_estimate.l1_gas_price,
            l2_gas_consumed: fee_estimate.l2_gas_consumed,
            l2_gas_price: fee_estimate.l2_gas_price,
            l1_data_gas_consumed: fee_estimate.l1_data_gas_consumed,
            l1_data_gas_price: fee_estimate.l1_data_gas_price,
            overall_fee: fee_estimate.overall_fee,
            detailed,
        }
    }
}

impl SncastCommandMessage for DryRunResponse {
    fn text(&self) -> String {
        let overall_fee_strk = BigDecimal::new(self.overall_fee.into(), 18.into());
        let builder = styling::OutputBuilder::new()
            .success_message("Dry run completed.")
            .blank_line()
            .field(
                "Overall Fee",
                &format!(
                    "{} Fri (~{} STRK)",
                    &overall_fee_strk.to_string(),
                    overall_fee_strk.round(4)
                ),
            );

        if self.detailed {
            builder
                .field("L1 Gas Consumed", &self.l1_gas_consumed.to_string())
                .field("L1 Gas Price", &self.l1_gas_price.to_string())
                .field("L2 Gas Consumed", &self.l2_gas_consumed.to_string())
                .field("L2 Gas Price", &self.l2_gas_price.to_string())
                .field(
                    "L1 Data Gas Consumed",
                    &self.l1_data_gas_consumed.to_string(),
                )
                .field("L1 Data Gas Price", &self.l1_data_gas_price.to_string())
                .build()
        } else {
            builder.build()
        }
    }
}
