use crate::{cheatcodes::EnhancedHintError, CheatedState};
use cairo_felt::Felt252;

impl CheatedState {
    pub fn expect_events(&mut self, events: Vec<Felt252>) -> Result<(), EnhancedHintError> {
        self.expected_events = events;
        Ok(())
    }
}
