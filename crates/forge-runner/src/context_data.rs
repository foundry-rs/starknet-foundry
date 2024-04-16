use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use std::collections::HashMap;

pub struct ContextData {
    pub runtime_data: RuntimeData,
    pub workspace_root: Utf8PathBuf,
    pub test_artifacts_path: Utf8PathBuf,
}

pub struct RuntimeData {
    pub contracts_data: ContractsData,
    pub environment_variables: HashMap<String, String>,
}
