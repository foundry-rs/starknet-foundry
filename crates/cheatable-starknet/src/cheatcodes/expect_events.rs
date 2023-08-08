use crate::{cheatcodes::EnhancedHintError, CheatedState};
use cairo_felt::Felt252;

pub struct Event {
    pub name: Felt252,
    pub keys: Vec<Felt252>,
    pub data: Vec<Felt252>,
}

impl CheatedState {
    pub fn expect_events(&mut self, events: Vec<Event>) -> Result<(), EnhancedHintError> {
        self.expected_events = events;
        Ok(())
    }
}
