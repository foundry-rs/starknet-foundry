use crate::state::CheatnetState;
use blockifier::execution::call_info::OrderedL2ToL1Message;
use cairo_felt::Felt252;
use conversions::serde::serialize::CairoSerialize;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;
use starknet_api::hash::StarkFelt;

#[derive(CairoSerialize, Clone)]
pub struct MessageToL1 {
    from_address: ContractAddress,
    to_address: Felt252,
    payload: Vec<Felt252>,
}

impl MessageToL1 {
    #[must_use]
    pub fn from_ordered_message(
        ordered_message: &OrderedL2ToL1Message,
        from_address: ContractAddress,
    ) -> MessageToL1 {
        let to_address: StarkFelt = ordered_message.message.to_address.into();

        Self {
            from_address,
            to_address: to_address.into_(),
            payload: ordered_message
                .message
                .payload
                .clone()
                .0
                .into_iter()
                .map(conversions::IntoConv::into_)
                .collect(),
        }
    }
}

impl CheatnetState {
    #[must_use]
    pub fn get_messages_to_l1(&self, message_offset: usize) -> Vec<MessageToL1> {
        self.detected_messages_to_l1[message_offset..].to_vec()
    }
}
