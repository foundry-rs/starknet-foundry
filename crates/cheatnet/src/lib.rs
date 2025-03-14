use state::CheatnetState;

pub mod constants;
pub mod forking;
pub mod runtime_extensions;
pub mod state;

const CHEAT_MAGIC: usize = usize::MAX;
