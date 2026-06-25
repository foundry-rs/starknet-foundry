Contract Declaration Improvements in snforge_std
Context

Reference issue: #1531, #2626
At the moment,  declare in snforge_std is string-based:

let contract = declare("HelloWorld").unwrap();

This has two drawbacks:

1. It is not type-safe, hence passing a non-existent contract won't show diagnostics in code editor.
2. It is limited to contract-name lookup, while some workflows could benefit from declaring contracts by a more explicit reference, such as full module path or a file path (containing Sierra).

Other

1. There's also an issue (#2626) about making the current declare behavior deterministic when multiple contracts share the same name.

Scope

1. Type-safe contract declaration by full module path
2. Declaration from a Sierra file path
3. Fix for contracts with duplicated names

1. Type-safe contract declaration by full module path

Ideally, it would be good if the current contract-name-based API could simply become type-safe while preserving the existing UX. However, this requires obtaining contract artifacts, i.e. building them. This is technically possible, but the usage of declare would enforce the build, which definitely doesn't seem a good way to go.
However, we can introduce a new macro declare!(...) which takes a full module path.
Example usage:

#[test]
fn declare_with_full_module_path() {
    let contract = declare!(hello_starknet::HelloStarknet)
        .unwrap()
        .contract_class();
        
    // ...
}

The macro works in two steps.
First, it expands the provided contract path into ordinary Cairo code that forces semantic resolution of that path through ContractState, using assert_path_type::<...::ContractState()>. This does not perform any runtime logic, but it makes the compiler verify that the provided module path really resolves to a contract module.
Second, after that compile-time check, the macro emits the actual declaration call. In the current POC, this is done by lowering the validated path into a canonical string form and passing it to the runtime declaration layer. As a result, the macro itself is responsible for type-safe validation, while the runtime layer is responsible for the actual contract declaration.
So for the example above, this is the expansion result:

{
    snforge_std::_internals::assert_path_type::<declare_path::HelloStarknet::ContractState>();
    snforge_std::declare_module_path("declare_path::HelloStarknet")
}

Below is an example of diagnostic when an invalid path is passed:
The current diagnostic is still a regular semantic error such as Identifier not found. This is fine for POC, although the final UX could be improved further in CairoLS.

Action items:

foundry

* add declare!(...) macro

cairols

* no required changes for POC
* optional: improve diagnostic UX for invalid expanded paths

2. Declaration from a Sierra file path

Another improvement would be allowing contract declaration from an explicit file path containing Sierra.
Example usage:

#[test]
fn declare_from_sierra_path() {
    let contract = declare_path!("target/dev/hello_starknet_HelloStarknet.contract_class.json")
        .unwrap()
        .contract_class();

    // ...
}

The macro works as follows:
declare_path! verifies that the argument is a string literal,  that the file exists, and that it can be parsed as a valid Sierra contract class JSON. After that validation, it lowers to the runtime call to a function declare_from_path .
One important detail is path resolution. If declare_path! accepts relative paths, then both Scarb and CairoLS must resolve them consistently against the correct package context. This is likely fine for a POC, but it relies on having a well-defined resolution base in the editor as well as during normal build.
Runtime declaration layer reads the file, deserializes it as Sierra and then performs the actual declaration. The macro acts only as a compile-time validation layer.
This means the macro is responsible for editor/build-time path diagnostics, while the runtime layer is be responsible for the actual declaration.

Below is an example of diagnostic when an non-existent path is passed:
An en example diagnostic when when a file is not a valid Sierra:

Action items:

foundry

* add declare_from_path(...) in snforge_std
* add declare_path!(...) macro (the names are not final, can be improved of course)

cairols

* no required changes for POC
* optional: research and improve diagnostic mapping for path-related errors

3. Fix for contracts with duplicated names

At the moment snforge identifies contracts by contract_name. This breaks when multiple contracts have the same name, for example:

src/
  lib.cairo        <- contract named MyContract
tests/
  contracts.cairo  <- contract named MyContract

Scarb handles this correctly, because it distinguishes contracts by full module_path. snforge does not, so duplicated names collide in the name-based lookup and the selected contract is non-deterministic.
The fix is to stop treating contract_name as a unique identifier and use full module_path internally. Then declare should behave as follows:

* if a name resolves to exactly one contract, declare("MyContract") works
* if a name resolves to multiple contracts, declare("MyContract") fails
* in the ambiguous case, the user must declare by full module path instead

This keeps the current UX for unambiguous cases and makes duplicated-name cases deterministic.

Action items:

* Fix in foundry, as described above

Questions

1. Do we want to add declare!(full::module::path) ?
2. Do we want to add declare_path! ?
3. Is the fix for contracts with duplicated names clear?

