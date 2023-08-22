use crate::CheatnetState;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

/// Represents emitted event. It is used in the CheatnetState to keep track of events
/// emitted in the `cheatnet::src::rpc::call_contract`
pub struct Event {
    pub from: ContractAddress,
    pub name: Felt252,
    pub keys: Vec<Felt252>,
    pub data: Vec<Felt252>,
}

/// Specifies which contract are spied on.
pub enum SpyOn {
    All,
    One(ContractAddress),
    Multiple(Vec<ContractAddress>),
}

impl CheatnetState {
    pub fn spy_events(&mut self, spy_on: SpyOn) {
        self.cheatcode_state.spy_events = Some(spy_on);
    }

    pub fn fetch_events(&mut self) -> (usize, Vec<Felt252>) {
        let serialized_events: Vec<Vec<Felt252>> = self
            .cheatcode_state
            .emitted_events
            .iter()
            .map(|event| {
                let mut flattened: Vec<Felt252> = vec![
                    Felt252::from_bytes_be(event.from.0.key().bytes()),
                    event.name.clone(),
                    Felt252::from(event.keys.len()),
                ];
                flattened.append(&mut event.keys.clone());
                flattened.push(Felt252::from(event.data.len()));
                flattened.append(&mut event.data.clone());

                flattened
            })
            .collect();

        let emitted_events_len = self.cheatcode_state.emitted_events.len();
        self.cheatcode_state.emitted_events = vec![];

        (emitted_events_len, serialized_events.concat())
    }
}
