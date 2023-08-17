use crate::cheatcodes::EnhancedHintError;
use crate::CheatnetState;
use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::MemBuffer;
use starknet_api::core::ContractAddress;

pub struct Event {
    pub from: ContractAddress,
    pub name: Felt252,
    pub keys: Vec<Felt252>,
    pub data: Vec<Felt252>,
}

impl CheatnetState {
    pub fn spy_events(&mut self) -> Result<(), EnhancedHintError> {
        self.cheatcode_state.spy_events = true;
        Ok(())
    }

    pub fn fetch_events(&mut self, buffer: &mut MemBuffer) -> Result<(), EnhancedHintError> {
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

        buffer
            .write(Felt252::from(self.cheatcode_state.emitted_events.len()))
            .expect("Failed to insert serialized events length");
        for felt in serialized_events.concat() {
            buffer
                .write(felt)
                .expect("Failed to insert serialized events");
        }

        self.cheatcode_state.emitted_events = vec![];

        Ok(())
    }
}
