use crate::forking::state::ForkStateReader;
use crate::CheatnetState;

use starknet::core::types::BlockId;

impl CheatnetState {
    #[must_use]
    pub fn setup_fork(&mut self, url: &str, block_id: BlockId) -> &mut CheatnetState {
        let fork_state_reader = ForkStateReader::new(url, block_id);
        self.blockifier_state.state.fork_state_reader = Some(fork_state_reader);
        self
    }
}
