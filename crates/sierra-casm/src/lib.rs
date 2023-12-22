use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use cairo_lang_starknet_sierra_0_1_0::casm_contract_class::CasmContractClass as CasmContractClassSierraV0;
use cairo_lang_starknet_sierra_0_1_0::contract_class::ContractClass as ContractClassSierraV0;
use cairo_lang_starknet_sierra_1_0_0::casm_contract_class::CasmContractClass as CasmContractClassSierraV1;
use cairo_lang_starknet_sierra_1_0_0::contract_class::ContractClass as ContractClassSierraV1;

/// `sierra_json` should be a json containing `sierra_program` and `entry_points_by_type`
pub fn compile(mut sierra_json: Value) -> Result<CasmContractClass> {
    sierra_json["abi"] = Value::Null;
    sierra_json["sierra_program_debug_info"] = Value::Null;
    sierra_json["contract_class_version"] = Value::String(String::new());

    macro_rules! compile_contract {
        ($sierra_type:ty, $casm_type:ty) => {{
            let sierra_class = serde_json::from_value::<$sierra_type>(sierra_json.clone()).unwrap();
            let casm_class = <$casm_type>::from_contract_class(sierra_class, true).unwrap();
            return Ok(old_casm_to_newest_casm::<$casm_type>(&casm_class));
        }};
    }

    let sierra_version = parse_sierra_version(&sierra_json)?;
    match sierra_version.as_slice() {
        [1, 2..=4, ..] => compile_contract!(ContractClass, CasmContractClass),
        [1, 0..=1, 0] => compile_contract!(ContractClassSierraV1, CasmContractClassSierraV1),
        [0, ..] => compile_contract!(ContractClassSierraV0, CasmContractClassSierraV0),
        _ => {
            anyhow::bail!(
                "Unable to compile Sierra to Casm. No matching ContractClass or CasmContractClass found for version "
                    .to_string() + &sierra_version.iter().map(|&num| num.to_string()).collect::<Vec<String>>().join("."),
            )
        }
    }
}

/// Converts `CasmContractClass` from the old `cairo_lang_starknet` library version
/// to the `CasmContractClass` from the newest version
fn old_casm_to_newest_casm<T>(value: &T) -> CasmContractClass
where
    T: Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_value(value).unwrap();
    serde_json::from_value::<CasmContractClass>(serialized).unwrap()
}

/// Extracts sierra version from the program
/// It will not be possible to convert sierra 0.1.0 version because it keeps its version only in the first felt252
/// (as a shortstring) while other versions keep it on the first 3 (major, minor, patch)
/// That's why it fallbacks to 0 when converting from Value to u8
fn parse_sierra_version(sierra_json: &Value) -> Result<Vec<u8>> {
    let parsed_values: Vec<u8> = sierra_json["sierra_program"]
        .as_array()
        .context("Unable to read sierra_program. Make sure it is an array of felts")?
        .iter()
        .take(3)
        .map(|x| u8::from_str_radix(&x.as_str().unwrap()[2..], 16).unwrap_or_default())
        .collect();

    Ok(parsed_values)
}
