use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use cairo_lang_starknet_v1_0_0_alpha6::casm_contract_class::CasmContractClass as CasmContractClassV1Alpha;
use cairo_lang_starknet_v1_0_0_alpha6::contract_class::ContractClass as ContractClassV1Alpha;
use cairo_lang_starknet_v1_0_0_rc0::casm_contract_class::CasmContractClass as CasmContractClassV1Rc0;
use cairo_lang_starknet_v1_0_0_rc0::contract_class::ContractClass as ContractClassV1Rc0;
use cairo_lang_starknet_v1_1_1::casm_contract_class::CasmContractClass as CasmContractClassV1_1;
use cairo_lang_starknet_v1_1_1::contract_class::ContractClass as ContractClassV1_1;

/// `sierra_json` should be a json containing `sierra_program` and `entry_points_by_type`
pub fn compile(mut sierra_json: Value) -> Result<CasmContractClass, String> {
    sierra_json["abi"] = Value::Null;
    sierra_json["sierra_program_debug_info"] = Value::Null;
    sierra_json["contract_class_version"] = Value::String(String::new());

    macro_rules! compile_contract {
        ($sierra_type:ty, $casm_type:ty) => {{
            if let Ok(sierra_class) = serde_json::from_value::<$sierra_type>(sierra_json.clone()) {
                if let Ok(casm_class) = <$casm_type>::from_contract_class(sierra_class, true) {
                    return Ok(old_casm_to_newest_casm::<$casm_type>(&casm_class));
                }
            }
        }};
    }

    compile_contract!(ContractClass, CasmContractClass);
    compile_contract!(ContractClassV1_1, CasmContractClassV1_1);
    compile_contract!(ContractClassV1Rc0, CasmContractClassV1Rc0);
    compile_contract!(ContractClassV1Alpha, CasmContractClassV1Alpha);

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
