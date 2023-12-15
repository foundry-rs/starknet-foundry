use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use cairo_lang_starknet_sierra_0_1_0::casm_contract_class::CasmContractClass as CasmContractClassSierraV0;
use cairo_lang_starknet_sierra_0_1_0::contract_class::ContractClass as ContractClassSierraV0;
use cairo_lang_starknet_sierra_1_0_0::casm_contract_class::CasmContractClass as CasmContractClassSierraV1;
use cairo_lang_starknet_sierra_1_0_0::contract_class::ContractClass as ContractClassSierraV1;
use cairo_lang_starknet_sierra_1_1_0::casm_contract_class::CasmContractClass as CasmContractClassSierraV1_1;
use cairo_lang_starknet_sierra_1_1_0::contract_class::ContractClass as ContractClassSierraV1_1;

/// `sierra_json` should be a json containing `sierra_program` and `entry_points_by_type`
pub fn compile(mut sierra_json: Value) -> Result<CasmContractClass, String> {
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

    let major = parse_sierra_version(1, &sierra_json)?;
    if major.as_str() == "0" {
        compile_contract!(ContractClassSierraV0, CasmContractClassSierraV0)
    }

    let sierra_version = parse_sierra_version(3, &sierra_json)?;
    match sierra_version.as_str() {
        "1.4.0" | "1.3.0" | "1.2.0" => compile_contract!(ContractClass, CasmContractClass),
        "1.1.0" => compile_contract!(ContractClassSierraV1, CasmContractClassSierraV1),
        "1.0.0" => compile_contract!(ContractClassSierraV1_1, CasmContractClassSierraV1_1),
        _ => {}
    };

    Err(
        "Unable to compile Sierra to Casm. No matching ContractClass or CasmContractClass found"
            .to_string(),
    )
}

/// converts `CasmContractClass` from the old `cairo_lang_starknet` library version
/// to the `CasmContractClass` from the newest version
fn old_casm_to_newest_casm<T>(value: &T) -> CasmContractClass
where
    T: Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_value(value).unwrap();
    serde_json::from_value::<CasmContractClass>(serialized).unwrap()
}

fn parse_sierra_version(slice_length: usize, sierra_json: &Value) -> Result<String, String> {
    let parsed_values: Vec<String> = sierra_json["sierra_program"]
        .as_array()
        .ok_or("Unable to read sierra_program. Make sure it is an array of felts")?
        .iter()
        .take(slice_length)
        .map(|x| {
            u8::from_str_radix(&x.as_str().unwrap()[2..], 16)
                .unwrap_or_default()
                .to_string()
        })
        .collect();

    Ok(parsed_values.join("."))
}
