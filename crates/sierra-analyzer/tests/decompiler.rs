use cairo_lang_starknet_classes::contract_class::ContractClass;
use sierra_analyzer::sierra_program::SierraProgram;
use std::fs;
use test_case::test_case;

#[test_case("account__account.sierra")]
#[test_case("corelib_usage.sierra")]
#[test_case("enum_flow.sierra")]
#[test_case("erc20__erc_20.sierra")]
#[test_case("fib_array.sierra")]
#[test_case("fib_box.sierra")]
#[test_case("fib_counter.sierra")]
#[test_case("fib_gas.sierra")]
#[test_case("fib_jumps.sierra")]
#[test_case("fib_local.sierra")]
#[test_case("fib_loop.sierra")]
#[test_case("fib_match.sierra")]
#[test_case("fib_no_gas.sierra")]
#[test_case("fib_struct.sierra")]
#[test_case("fib_u128_checked.sierra")]
#[test_case("fib_u128.sierra")]
#[test_case("fib_unary.sierra")]
#[test_case("fib.sierra")]
#[test_case("hash_chain_gas.sierra")]
#[test_case("hash_chain.sierra")]
#[test_case("hello_starknet__hello_starknet.sierra")]
#[test_case("hello_starknet.sierra")]
#[test_case("match_or.sierra")]
#[test_case("minimal_contract__minimal_contract.sierra")]
#[test_case("minimal_contract.sierra")]
#[test_case("mintable__mintable_erc20_ownable.sierra")]
#[test_case("multi_component__contract_with_4_components.sierra")]
#[test_case("new_syntax_test_contract__counter_contract.sierra")]
#[test_case("new_syntax_test_contract.sierra")]
#[test_case("ownable_erc20__ownable_erc20_contract.sierra")]
#[test_case("pedersen_test.sierra")]
#[test_case("symbolic_execution_test.sierra")]
#[test_case("test_contract__test_contract.sierra")]
#[test_case("testing.sierra")]
#[test_case("token_bridge__token_bridge.sierra")]
#[test_case("upgradable_counter__counter_contract.sierra")]
#[test_case("upgradable_counter.sierra")]
#[test_case("with_erc20__erc20_contract.sierra")]
#[test_case("with_ownable__ownable_balance.sierra")]
#[test_case("with_ownable.sierra")]
fn test_decompiler_sierra_no_error(file_name: &str) {
    // Construct the file path
    let file_path = format!("./examples/sierra/{}", file_name);

    // Read file content
    let content = fs::read_to_string(file_path).expect("Unable to read file");

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    // Check that the decompiler output is not empty
    assert!(!decompiler_output.is_empty());
}

#[test_case("account__account.contract_class.json")]
#[test_case("erc20.contract_class.json")]
#[test_case("erc20__erc_20.contract_class.json")]
#[test_case("hello_starknet__hello_starknet.contract_class.json")]
#[test_case("minimal_contract.contract_class.json")]
#[test_case("minimal_contract__minimal_contract.contract_class.json")]
#[test_case("mintable.contract_class.json")]
#[test_case("mintable__mintable_erc20_ownable.contract_class.json")]
#[test_case("multi_component__contract_with_4_components.contract_class.json")]
#[test_case("new_syntax_test_contract.contract_class.json")]
#[test_case("new_syntax_test_contract__counter_contract.contract_class.json")]
#[test_case("ownable_erc20.contract_class.json")]
#[test_case("ownable_erc20__ownable_erc20_contract.contract_class.json")]
#[test_case("storage_accesses__storage_accesses.contract_class.json")]
#[test_case("test_contract.contract_class.json")]
#[test_case("test_contract__test_contract.contract_class.json")]
#[test_case("token_bridge.contract_class.json")]
#[test_case("token_bridge__token_bridge.contract_class.json")]
#[test_case("upgradable_counter.contract_class.json")]
#[test_case("upgradable_counter__counter_contract.contract_class.json")]
#[test_case("with_erc20.contract_class.json")]
#[test_case("with_erc20__erc20_contract.contract_class.json")]
#[test_case("with_erc20_mini__erc20_mini_contract.contract_class.json")]
#[test_case("with_ownable.contract_class.json")]
#[test_case("with_ownable_mini__ownable_mini_contract.contract_class.json")]
#[test_case("with_ownable__ownable_balance.contract_class.json")]
fn test_decompiler_starknet_no_error(file_name: &str) {
    // Construct the file path
    let file_path = format!("./examples/starknet/{}", file_name);

    // Read file content
    let content = fs::read_to_string(file_path).expect("Unable to read file");

    // Deserialize JSON into a ContractClass
    let program_string = serde_json::from_str::<ContractClass>(&content)
        .ok()
        .and_then(|prog| prog.extract_sierra_program().ok())
        .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string());

    // Init a new SierraProgram with the .contract_class.json file content
    let program = SierraProgram::new(program_string);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the contract_class program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    // Check that the decompiler output is not empty
    assert!(!decompiler_output.is_empty());
}

#[test]
fn test_decompiler_output() {
    // Read file content
    let content = include_str!("../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    let expected_output = r#"// Function 1
func examples::fib::fib (v0: felt252, v1: felt252, v2: felt252) -> (felt252) {
	v3 = v2
	if (v3 == 0) {		
		v5 = v1
		v6 = v0 + v5
		v7 = 1
		v8 = v2 - v7
		v9 = user@examples::fib::fib(v1, v6, v8)
		return (v9)
	} else {	
		return (v0)
	}
}"#;
    assert_eq!(decompiler_output, expected_output);
}

#[test]
fn test_decompiler_verbose_output() {
    // Read file content
    let content = include_str!("../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Use the verbose output
    let verbose_output = true;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    let expected_output = r#"type felt252
type Const<felt252, 1>
type NonZero<felt252>

libfunc disable_ap_tracking
libfunc dup<felt252>
libfunc felt252_is_zero
libfunc branch_align
libfunc drop<felt252>
libfunc store_temp<felt252>
libfunc drop<NonZero<felt252>>
libfunc felt252_add
libfunc const_as_immediate<Const<felt252, 1>>
libfunc felt252_sub
libfunc function_call<user@examples::fib::fib>

// Function 1
func examples::fib::fib (v0: felt252, v1: felt252, v2: felt252) -> (felt252) {
	disable_ap_tracking()
	v2, v3 = dup<felt252>(v2)
	if (felt252_is_zero(v3) == 0) {		
		branch_align()
		drop<NonZero<felt252>>(v4)
		v1, v5 = dup<felt252>(v1)
		v6 = felt252_add(v0, v5)
		v7 = const_as_immediate<Const<felt252, 1>>()
		v8 = felt252_sub(v2, v7)
		v1 = store_temp<felt252>(v1)
		v6 = store_temp<felt252>(v6)
		v8 = store_temp<felt252>(v8)
		v9 = user@examples::fib::fib(v1, v6, v8)
		return (v9)
	} else {	
		branch_align()
		drop<felt252>(v1)
		drop<felt252>(v2)
		v0 = store_temp<felt252>(v0)
		return (v0)
	}
}"#;
    assert_eq!(decompiler_output, expected_output);
}

#[test]
fn test_decompiler_array_output() {
    // Read file content
    let content = include_str!("../examples/sierra/fib_gas.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't Use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    let expected_output = r#"// Function 1
func examples::fib::fib (v0: RangeCheck, v1: GasBuiltin, v2: felt252, v3: felt252, v4: felt252) -> (RangeCheck, GasBuiltin, core::panics::PanicResult::<(core::felt252)>) {
	if (withdraw_gas(v0, v1) == 0) {		
		v20 = Array<felt252>::new()
		v21 = 375233589013918064796019 // "Out of gas"
		v22 = v20.append(v21)
		v23 = struct_construct<core::panics::Panic>()
		v24 = struct_construct<Tuple<core::panics::Panic, Array<felt252>>>(v23, v22)
		v25 = enum_init<core::panics::PanicResult::<(core::felt252)>, 1>(v24)
		return (v7, v8, v25)
	} else {	
		v9 = v4
		if (v9 == 0) {			
			v13 = v3
			v14 = v2 + v13
			v15 = 1
			v16 = v4 - v15
			v17, v18, v19 = user@examples::fib::fib(v5, v6, v3, v14, v16)
			return (v17, v18, v19)
		} else {		
			v11 = struct_construct<Tuple<felt252>>(v2)
			v12 = enum_init<core::panics::PanicResult::<(core::felt252)>, 0>(v11)
			return (v5, v6, v12)
		}
	}
}"#;
    assert_eq!(decompiler_output, expected_output);
}
