use crate::helpers::{
    constants::{
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA, CONTRACTS_DIR,
        MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
    },
    fixtures::{create_and_deploy_oz_account, duplicate_contract_directory_with_salt},
    runner::runner,
};
use indoc::indoc;
use serde_json::Value;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

fn extract_contract_address(output: &[u8]) -> String {
    let stdout = std::str::from_utf8(output).expect("stdout is not utf8");
    for line in stdout.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line)
            && let Some(addr) = v.get("contract_address").and_then(|a| a.as_str())
        {
            return addr.to_string();
        }
    }
    panic!("Could not find contract_address in output:\n{stdout}");
}

#[test]
fn test_happy_case_class_hash() {
    let args = vec![
        "utils",
        "contract-address",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x1",
    ];
    let output = runner(&args).assert().success();
    assert_stdout_contains(output, indoc! {r"Contract Address: 0x0[..]"});
}

#[test]
fn test_happy_case_class_hash_json() {
    let args = vec![
        "--json",
        "utils",
        "contract-address",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x1",
    ];
    let output = runner(&args).assert().success();
    assert_stdout_contains(
        output,
        indoc! {r#"{"command":"contract-address","contract_address":"0x0[..]","type":"response"}"#},
    );
}

#[test]
fn test_happy_case_no_salt() {
    // When --salt is omitted, a random salt is generated and a valid address is still returned.
    let args = vec![
        "utils",
        "contract-address",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
    ];
    let output = runner(&args).assert().success();
    assert_stdout_contains(output, indoc! {r"Contract Address: 0x0[..]"});
}

#[test]
fn test_happy_case_unique() {
    let args = vec![
        "utils",
        "contract-address",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x1",
        "--unique",
        "--deployer-address",
        "0x123",
    ];
    let output = runner(&args).assert().success();
    assert_stdout_contains(output, indoc! {r"Contract Address: 0x0[..]"});
}

#[test]
fn test_different_salts_produce_different_addresses() {
    let run = |salt: &str| {
        let args = vec![
            "--json",
            "utils",
            "contract-address",
            "--class-hash",
            MAP_CONTRACT_CLASS_HASH_SEPOLIA,
            "--salt",
            salt,
        ];
        let out = runner(&args).assert().success();
        extract_contract_address(&out.get_output().stdout)
    };

    let addr1 = run("0x1");
    let addr2 = run("0x2");
    assert_ne!(
        addr1, addr2,
        "Different salts must produce different addresses"
    );
}

#[test]
fn test_unique_vs_not_unique_produce_different_addresses() {
    let base = vec![
        "--json",
        "utils",
        "contract-address",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x1",
    ];

    let out_plain = runner(&base).assert().success();
    let addr_plain = extract_contract_address(&out_plain.get_output().stdout);

    let mut unique_args = base.clone();
    unique_args.extend(["--unique", "--deployer-address", "0x1"]);
    let out_unique = runner(&unique_args).assert().success();
    let addr_unique = extract_contract_address(&out_unique.get_output().stdout);

    assert_ne!(
        addr_plain, addr_unique,
        "--unique must change the resulting address"
    );
}

#[test]
fn test_happy_case_constructor_calldata() {
    let args = vec![
        "utils",
        "contract-address",
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
        "--constructor-calldata",
        "0x1",
        "0x2",
        "--salt",
        "0x1",
    ];
    let output = runner(&args).assert().success();
    assert_stdout_contains(output, indoc! {r"Contract Address: 0x0[..]"});
}

#[test]
fn test_calldata_affects_address() {
    let run = |calldata: &str| {
        let args = vec![
            "--json",
            "utils",
            "contract-address",
            "--class-hash",
            MAP_CONTRACT_CLASS_HASH_SEPOLIA,
            "--constructor-calldata",
            calldata,
            "--salt",
            "0x1",
        ];
        let out = runner(&args).assert().success();
        extract_contract_address(&out.get_output().stdout)
    };

    let addr1 = run("0x1");
    let addr2 = run("0x2");
    assert_ne!(
        addr1, addr2,
        "Different calldata must produce different addresses"
    );
}

#[test]
fn test_happy_case_arguments() {
    let args = vec![
        "utils",
        "contract-address",
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
        "--arguments",
        "1, 2",
        "--salt",
        "0x1",
        "--url",
        URL,
    ];
    let output = runner(&args).assert().success();
    assert_stdout_contains(output, indoc! {r"Contract Address: 0x0[..]"});
}

#[test]
fn test_happy_case_contract_name() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "contract_address",
        "contract_name",
    );

    let args = vec![
        "utils",
        "contract-address",
        "--contract-name",
        "Map",
        "--salt",
        "0x1",
    ];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .success();
    assert_stdout_contains(output, indoc! {r"Contract Address: 0x0[..]"});
}

#[test]
fn test_contract_name_matches_class_hash() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "contract_address",
        "name_vs_hash",
    );

    // Compute class hash via utils class-hash
    let json_args = vec!["--json", "utils", "class-hash", "--contract-name", "Map"];
    let class_hash_out = runner(&json_args)
        .current_dir(contract_path.path())
        .assert()
        .success();

    let stdout = std::str::from_utf8(&class_hash_out.get_output().stdout).unwrap();
    let class_hash = stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|v| v["class_hash"].as_str().map(str::to_string))
        .expect("No class_hash field in output");
    let class_hash = class_hash.as_str();

    // Address via --contract-name
    let name_args = vec![
        "--json",
        "utils",
        "contract-address",
        "--contract-name",
        "Map",
        "--salt",
        "0x1",
    ];
    let name_out = runner(&name_args)
        .current_dir(contract_path.path())
        .assert()
        .success();

    // Address via --class-hash
    let hash_args = vec![
        "--json",
        "utils",
        "contract-address",
        "--class-hash",
        class_hash,
        "--salt",
        "0x1",
    ];
    let hash_out = runner(&hash_args).assert().success();

    let addr_by_name = extract_contract_address(&name_out.get_output().stdout);
    let addr_by_hash = extract_contract_address(&hash_out.get_output().stdout);
    assert_eq!(
        addr_by_name, addr_by_hash,
        "Error: --contract-name and --class-hash must produce the same address for the same contract"
    );
}

#[tokio::test]
async fn test_precalculated_address_matches_deployed_address() {
    let tempdir = create_and_deploy_oz_account().await;

    // Read deployer address from accounts.json
    let accounts_content = std::fs::read_to_string(tempdir.path().join("accounts.json")).unwrap();
    let accounts: Value = serde_json::from_str(&accounts_content).unwrap();
    let account_address = accounts["alpha-sepolia"]["my_account"]["address"]
        .as_str()
        .expect("Failed to get account address");

    // Precalculate the expected address (pure local computation)
    let precalc_args = vec![
        "--json",
        "utils",
        "contract-address",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x42",
        "--unique",
        "--deployer-address",
        account_address,
    ];
    let precalc_out = runner(&precalc_args).assert().success();
    let precalculated = extract_contract_address(&precalc_out.get_output().stdout);

    // Actually deploy and read the address the network reports
    let deploy_args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--json",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x42",
        "--unique",
    ];
    let deploy_out = runner(&deploy_args)
        .current_dir(tempdir.path())
        .assert()
        .success();
    let deployed = extract_contract_address(&deploy_out.get_output().stdout);

    assert_eq!(
        precalculated, deployed,
        "Precalculated address must match the address assigned by the network on deploy"
    );
}

#[test]
fn test_unique_without_account_address() {
    // --unique without --deployer-address uses zero as the default
    let args = vec![
        "utils",
        "contract-address",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x1",
        "--unique",
    ];
    let output = runner(&args).assert().success();
    assert_stdout_contains(output, indoc! {r"Contract Address: 0x0[..]"});
}

#[test]
fn test_missing_required_arg() {
    // Neither --class-hash nor --contract-name — clap should reject this
    let args = vec!["utils", "contract-address", "--salt", "0x1"];
    runner(&args).assert().failure();
}

#[test]
fn test_contract_name_not_found_in_artifacts() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "contract_address",
        "not_found",
    );

    let args = vec![
        "utils",
        "contract-address",
        "--contract-name",
        "NonExistentContract",
        "--salt",
        "0x1",
    ];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .failure();
    assert_stderr_contains(output, indoc! {"[..]NonExistentContract[..]"});
}
