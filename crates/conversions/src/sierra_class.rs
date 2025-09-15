use crate::TryFromConv;
use anyhow::Result;
use starknet::core::types::{
    FlattenedSierraClass,
    contract::{AbiEntry, SierraClass, SierraClassDebugInfo},
};

use std::vec;

impl TryFromConv<FlattenedSierraClass> for SierraClass {
    type Error = anyhow::Error;

    fn try_from_(class: FlattenedSierraClass) -> Result<Self, Self::Error> {
        Ok(SierraClass {
            sierra_program: class.sierra_program,
            sierra_program_debug_info: SierraClassDebugInfo {
                type_names: vec![],
                libfunc_names: vec![],
                user_func_names: vec![],
            },
            contract_class_version: class.contract_class_version,
            entry_points_by_type: class.entry_points_by_type,
            abi: serde_json::from_str::<Vec<AbiEntry>>(&class.abi)?,
        })
    }
}
