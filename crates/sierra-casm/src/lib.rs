use anyhow::Context;
use cairo_lang_utils::bigint::BigUintAsHex;
use num_bigint::BigUint;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error;
use starknet::core::types::FlattenedSierraClass;

use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_starknet_v1_0_0_alpha6::casm_contract_class::CasmContractClass as CasmContractClassV1Alpha;
use cairo_lang_starknet_v1_0_0_alpha6::contract_class::ContractClass as ContractClassV1Alpha;
use cairo_lang_starknet_v1_0_0_rc0::casm_contract_class::CasmContractClass as CasmContractClassV1Rc0;
use cairo_lang_starknet_v1_0_0_rc0::contract_class::ContractClass as ContractClassV1Rc0;
use cairo_lang_starknet_v1_1_1::casm_contract_class::CasmContractClass as CasmContractClassV1_1;
use cairo_lang_starknet_v1_1_1::contract_class::ContractClass as ContractClassV1_1;

pub fn compile(definition: &FlattenedSierraClass) -> anyhow::Result<CasmContractClass> {
    macro_rules! compile_contract {
        ($sierra_type:ty, $casm_type:ty) => {{
            let sierra_class = flattened_sierra_to_contract_class::<$sierra_type>(definition)?;
            let maybe_casm_class =
                <$casm_type>::from_contract_class(sierra_class, true).context("Compiling to CASM");
            if let Ok(casm_class) = maybe_casm_class {
                return casm_to_main_casm::<$casm_type>(&casm_class);
            }
        }};
    }

    compile_contract!(ContractClass, CasmContractClass);
    compile_contract!(ContractClassV1_1, CasmContractClassV1_1);
    compile_contract!(ContractClassV1Rc0, CasmContractClassV1Rc0);
    compile_contract!(ContractClassV1Alpha, CasmContractClassV1Alpha);

    unreachable!()
}

fn flattened_sierra_to_contract_class<T: DeserializeOwned>(
    value: &FlattenedSierraClass,
) -> Result<T, Error> {
    let converted_sierra_program: Vec<BigUintAsHex> = value
        .sierra_program
        .iter()
        .map(|field_element| BigUintAsHex {
            value: BigUint::from_bytes_be(&field_element.to_bytes_be()),
        })
        .collect();
    let converted_entry_points: ContractEntryPoints =
        serde_json::from_str(&serde_json::to_string(&value.entry_points_by_type)?)?;

    let json = serde_json::json!({
        "sierra_program": converted_sierra_program,
        "contract_class_version": value.contract_class_version,
        "entry_points_by_type": converted_entry_points,
    });
    serde_json::from_value::<T>(json)
}

fn casm_to_main_casm<T>(value: &T) -> anyhow::Result<CasmContractClass>
where
    T: Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_value(value)?;
    Ok(serde_json::from_value::<CasmContractClass>(serialized)?)
}
