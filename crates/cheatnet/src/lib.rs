use blockifier::state::cached_state::CachedState;
use camino::Utf8PathBuf;
use constants::build_testing_state;
use state::{CheatcodeState, DictStateReader};

pub mod cheatcodes;
pub mod constants;
pub mod rpc;
pub mod state;

pub struct CheatnetState {
    cheatcode_state: CheatcodeState,
    blockifier_state: CachedState<DictStateReader>,
}

impl CheatnetState {
    #[must_use]
    pub fn new(predeployed_contracts: &Utf8PathBuf) -> Self {
        CheatnetState {
            cheatcode_state: CheatcodeState::new(),
            blockifier_state: build_testing_state(predeployed_contracts),
        }
    }
}
