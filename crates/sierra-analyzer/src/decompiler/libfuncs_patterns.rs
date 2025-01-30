use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Those libfuncs id patterns are blacklisted from the regular decompiler output (not the verbose)
    /// to make it more readable
    ///
    /// We use lazy_static for performances issues

    // Variable drop
    pub static ref DROP_REGEX: Regex = Regex::new(r"drop(<.*>)?").unwrap();

    // Store temporary variable
    pub static ref STORE_TEMP_REGEX: Regex = Regex::new(r"store_temp(<.*>)?").unwrap();

    /// These are libfuncs id patterns whose representation in the decompiler output can be improved

    // User defined function call
    pub static ref FUNCTION_CALL_REGEX: Regex = Regex::new(r"function_call<(.*)>").unwrap();

    // Arithmetic operations
    pub static ref ADDITION_REGEX: Vec<Regex> = vec![
        Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_add").unwrap(),
        Regex::new(r"function_call<user@core::Felt(8|16|32|64|128|252)Add::add>").unwrap(),
    ];
    pub static ref SUBSTRACTION_REGEX: Vec<Regex> = vec![
        Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_sub").unwrap(),
        Regex::new(r"function_call<user@core::Felt(8|16|32|64|128|252)Sub::sub>").unwrap(),
     ];
    pub static ref MULTIPLICATION_REGEX: Vec<Regex> = vec![
        Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_mul").unwrap(),
        Regex::new(r"function_call<user@core::Felt(8|16|32|64|128|252)Mul::mul>").unwrap(),
    ];

    // Variable duplication
    pub static ref DUP_REGEX: Regex = Regex::new(r"dup(<.*>)?").unwrap();

    // Variable renaming
    pub static ref VARIABLE_ASSIGNMENT_REGEX: Vec<Regex> = vec![
        Regex::new(r"rename<.+>").unwrap(),
        Regex::new(r"store_temp<.+>").unwrap(),
        Regex::new(r"store_local<.+>").unwrap(),
        Regex::new(r"unbox<.+>").unwrap()
    ];

    // Check if an integer is 0
    pub static ref IS_ZERO_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)_is_zero").unwrap();

    // Consts declarations
    pub static ref CONST_REGEXES: Vec<Regex> = vec![
        Regex::new(r"const_as_immediate<Const<.*, (?P<const>-?[0-9]+)>>").unwrap(),
        Regex::new(r"storage_base_address_const<(?P<const>-?[0-9]+)>").unwrap(),
        Regex::new(r"(felt|u)_?(8|16|32|64|128|252)_const<(?P<const>-?[0-9]+)>").unwrap(),
    ];

    // User defined function
    pub static ref USER_DEFINED_FUNCTION_REGEX: Regex = Regex::new(r"(function_call|(\[[0-9]+\]))(::)?<user@(?P<function_id>.+)>").unwrap();

    // Array declarations & mutations
    pub static ref NEW_ARRAY_REGEX: Regex = Regex::new(r"array_new<(?P<array_type>.+)>").unwrap();
    pub static ref ARRAY_APPEND_REGEX: Regex = Regex::new(r"array_append<(.+)>").unwrap();

    // Regex of a type ID
    // Used to match and replace them in remote contracts
    pub static ref TYPE_ID_REGEX: Regex = Regex::new(r"(?<type_id>\[[0-9]+\])").unwrap();

    // User defined types IDs are the 250 first bits of the id name Keccak hash
    // https://github.com/starkware-libs/cairo/blob/b29f639c2090822914f52db6696d71748a8b93a6/crates/cairo-lang-sierra/src/ids.rs#L118
    pub static ref USER_DEFINED_TYPE_ID_REGEX: Regex = Regex::new(r"ut@\[(?<type_id>[0-9]+)\]").unwrap();

    /// Irrelevant callgraph functions regexes
    pub static ref IRRELEVANT_CALLGRAPH_FUNCTIONS_REGEXES: Vec<Regex> = {
        let mut regexes = vec![
            DROP_REGEX.clone(),
            STORE_TEMP_REGEX.clone(),
            NEW_ARRAY_REGEX.clone(),
            ARRAY_APPEND_REGEX.clone(),
            DUP_REGEX.clone(),
            IS_ZERO_REGEX.clone(),
            // Add the additional strings as regexes
            Regex::new(r"branch_align").unwrap(),
            Regex::new(r"disable_ap_tracking").unwrap(),
            Regex::new(r"enable_ap_tracking").unwrap(),
            Regex::new(r"finalize_locals").unwrap(),
            Regex::new(r"revoke_ap_tracking").unwrap(),
            Regex::new(r"get_builtin_costs").unwrap(),
        ];

        // Extend the vector with all the arithmetic operations regexes
        regexes.extend(ADDITION_REGEX.clone());
        regexes.extend(SUBSTRACTION_REGEX.clone());
        regexes.extend(MULTIPLICATION_REGEX.clone());

        // Extend the vector with all the variable assignment regexes
        regexes.extend(VARIABLE_ASSIGNMENT_REGEX.clone());
        regexes
    };
}
