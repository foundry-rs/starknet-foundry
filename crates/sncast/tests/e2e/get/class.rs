use crate::helpers::{
    constants::{
        DEVNET_OZ_ADDRESS_CAIRO_0, DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        URL,
    },
    runner::runner,
};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use sncast::helpers::constants::MAP_CONTRACT_ADDRESS_SEPOLIA;

#[derive(Copy, Clone, Debug)]
enum ContractVariant {
    Sierra,
    Legacy,
}

impl ContractVariant {
    fn class_hash(self) -> &'static str {
        match self {
            ContractVariant::Sierra => MAP_CONTRACT_CLASS_HASH_SEPOLIA,
            ContractVariant::Legacy => DEVNET_OZ_CLASS_HASH_CAIRO_0,
        }
    }

    fn class_address(self) -> &'static str {
        match self {
            ContractVariant::Sierra => MAP_CONTRACT_ADDRESS_SEPOLIA,
            ContractVariant::Legacy => DEVNET_OZ_ADDRESS_CAIRO_0,
        }
    }
}

fn class_output(output: &str) -> &str {
    output
        .split_once("\nTo see class details, visit:\n")
        .map_or(output, |(class_output, _)| class_output)
}

#[tokio::test]
async fn test_invalid_block_id() {
    let args = vec![
        "get",
        "class",
        "-g",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--block-id",
        "invalid_block",
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get class
        Error: Incorrect value passed for block_id = invalid_block. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)
        "},
    );
}

#[test]
fn test_class_hash_or_address_required() {
    let args = vec!["get", "class", "--url", URL];
    let output = runner(&args).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          <--hash <HASH>|--address <ADDRESS>>

        Usage: sncast get class --url <URL> <--hash <HASH>|--address <ADDRESS>>

        For more information, try '--help'.
        "},
    );
}

mod hash {
    use indoc::indoc;
    use serde::Deserialize;
    use shared::test_utils::output_assert::{
        AsOutput, assert_stderr_contains, assert_stdout_contains,
    };
    use starknet_rust::core::types::ContractClass;

    use super::{ContractVariant, class_output};
    use crate::helpers::constants::{MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL};
    use crate::helpers::runner::runner;
    use test_case::test_case;

    #[test_case(ContractVariant::Sierra)]
    #[test_case(ContractVariant::Legacy)]
    #[tokio::test]
    async fn test_pretty_output(variant: ContractVariant) {
        let args = vec!["get", "class", "-g", variant.class_hash(), "--url", URL];

        let output = runner(&args).assert().success();

        let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
        assert_stdout_contains(
            output,
            indoc! {"
            To see class details, visit:
            class: https://sepolia.voyager.online/class/[..]
        "},
        );

        let res: ContractClass =
            serde_json::from_str(class_output(&stdout)).expect("Failed to parse as ContractClass");

        assert!(
            matches!(
                (variant, res),
                (ContractVariant::Sierra, ContractClass::Sierra(_))
                    | (ContractVariant::Legacy, ContractClass::Legacy(_))
            ),
            "Expected {variant:?}"
        );

        insta::assert_snapshot!(format!("get-class-pretty-{variant:?}"), stdout);
    }

    #[test_case(ContractVariant::Sierra)]
    #[test_case(ContractVariant::Legacy)]
    #[tokio::test]
    async fn test_json_output(variant: ContractVariant) {
        #[derive(Deserialize)]
        struct Response {
            #[serde(flatten)]
            contract: ContractClass,
            command: String,
            #[serde(rename = "type")]
            res_type: String,
        }

        #[derive(Deserialize)]
        struct Link {
            links: String,
            title: String,
            #[serde(rename = "type")]
            res_type: String,
        }

        let args = vec![
            "--json",
            "get",
            "class",
            "-g",
            variant.class_hash(),
            "--url",
            URL,
        ];

        let output = runner(&args).assert().success();
        let mut lines = output.as_stdout().lines();
        let response = lines.next().unwrap();
        let link = lines.next().unwrap();

        let res: Response = serde_json::from_str(response)
            .expect("Failed to parse as ContractClass with additional data");

        let link_res: Link = serde_json::from_str(link).expect("Failed to parse as Explorer link");

        assert_eq!(&res.res_type, "response");
        assert_eq!(&res.command, "get class");

        assert!(
            matches!(
                (variant, res.contract),
                (ContractVariant::Sierra, ContractClass::Sierra(_))
                    | (ContractVariant::Legacy, ContractClass::Legacy(_))
            ),
            "Expected {variant:?}"
        );

        assert_eq!(link_res.title, "class");
        assert_eq!(link_res.res_type, "notification");
        assert!(
            link_res
                .links
                .starts_with("class: https://sepolia.voyager.online/class/"),
            "Link should be valid voyager URL"
        );
    }

    #[tokio::test]
    async fn test_nonexistent_class_hash() {
        let args = vec!["get", "class", "-g", "0x43", "--url", URL];

        let output = runner(&args).assert().failure();

        assert_stderr_contains(
            output,
            indoc! {r"
        Command: get class
        Error: Provided class hash does not exist
        "},
        );
    }

    #[tokio::test]
    async fn test_nonexistent_class_hash_json() {
        let args = vec!["--json", "get", "class", "-g", "0x43", "--url", URL];

        let output = runner(&args).assert().failure();

        let res: serde_json::Value = serde_json::from_str(output.as_stderr()).unwrap();
        let expected = serde_json::json!({
            "command": "get class",
            "type": "error",
            "error": "Provided class hash does not exist"
        });

        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_with_block_id() {
        let args = vec![
            "get",
            "class",
            "-g",
            MAP_CONTRACT_CLASS_HASH_SEPOLIA,
            "--block-id",
            "latest",
            "--url",
            URL,
        ];

        let output = runner(&args).assert().success();

        let _: ContractClass = serde_json::from_str(class_output(output.as_stdout()))
            .expect("Failed to parse as ContractClass");
    }
}

mod address {
    use crate::helpers::constants::URL;
    use crate::helpers::runner::runner;
    use configuration::test_utils::copy_config_to_tempdir;
    use indoc::indoc;
    use serde::Deserialize;
    use shared::test_utils::output_assert::{
        AsOutput, assert_stderr_contains, assert_stdout_contains,
    };
    use sncast::helpers::constants::MAP_CONTRACT_ADDRESS_SEPOLIA;
    use starknet_rust::core::types::ContractClass;

    use test_case::test_case;

    use super::{ContractVariant, class_output};

    #[test_case(ContractVariant::Sierra)]
    #[test_case(ContractVariant::Legacy)]
    #[tokio::test]
    async fn test_pretty_output(variant: ContractVariant) {
        let args = vec!["get", "class", "-d", variant.class_address(), "--url", URL];

        let output = runner(&args).assert().success();
        let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
        assert_stdout_contains(
            output,
            indoc! {"
            To see class details, visit:
            class: https://sepolia.voyager.online/class/[..]
        "},
        );

        let res: ContractClass =
            serde_json::from_str(class_output(&stdout)).expect("Failed to parse as ContractClass");

        match (variant, res) {
            (ContractVariant::Sierra, ContractClass::Sierra(_))
            | (ContractVariant::Legacy, ContractClass::Legacy(_)) => {}
            _ => panic!("Expected {variant:?}"),
        }

        insta::assert_snapshot!(format!("get-class-pretty-{variant:?}"), stdout);
    }

    #[test_case(ContractVariant::Sierra)]
    #[test_case(ContractVariant::Legacy)]
    #[tokio::test]
    async fn test_json_output(variant: ContractVariant) {
        #[derive(Deserialize)]
        struct Response {
            #[serde(flatten)]
            contract: ContractClass,
            command: String,
            #[serde(rename = "type")]
            res_type: String,
        }

        #[derive(Deserialize)]
        struct Link {
            links: String,
            title: String,
            #[serde(rename = "type")]
            res_type: String,
        }

        let args = vec![
            "--json",
            "get",
            "class",
            "-d",
            variant.class_address(),
            "--url",
            URL,
        ];

        let output = runner(&args).assert().success();

        let mut lines = output.as_stdout().lines();
        let response = lines.next().unwrap();
        let link = lines.next().unwrap();

        let res: Response = serde_json::from_str(response)
            .expect("Failed to parse as ContractClass with additional data");
        let link_res: Link = serde_json::from_str(link).expect("Failed to parse as Explorer link");

        assert_eq!(&res.res_type, "response");
        assert_eq!(&res.command, "get class");

        match (variant, res.contract) {
            (ContractVariant::Sierra, ContractClass::Sierra(_))
            | (ContractVariant::Legacy, ContractClass::Legacy(_)) => {}
            _ => panic!("Expected {variant:?}"),
        }

        assert_eq!(link_res.title, "class");
        assert_eq!(link_res.res_type, "notification");
        assert!(
            link_res
                .links
                .starts_with("class: https://sepolia.voyager.online/class/"),
            "Link should be valid voyager URL"
        );
    }

    #[tokio::test]
    async fn test_non_existent_address() {
        let args = vec!["get", "class", "-d", "0x43", "--url", URL];

        let output = runner(&args).assert().failure();

        assert_stderr_contains(
            output,
            indoc! {r"
        Command: get class
        Error: There is no contract at the specified address
        "},
        );
    }

    #[tokio::test]
    async fn test_non_existent_address_json() {
        let args = vec!["--json", "get", "class", "-d", "0x43", "--url", URL];

        let output = runner(&args).assert().failure();

        let res: serde_json::Value = serde_json::from_str(output.as_stderr()).unwrap();
        let expected = serde_json::json!({
            "command": "get class",
            "type": "error",
            "error": "There is no contract at the specified address"
        });

        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_with_block_id() {
        let args = vec![
            "get",
            "class",
            "-d",
            MAP_CONTRACT_ADDRESS_SEPOLIA,
            "--block-id",
            "latest",
            "--url",
            URL,
        ];

        let output = runner(&args).assert().success();

        let _: ContractClass = serde_json::from_str(class_output(output.as_stdout()))
            .expect("Failed to parse as ContractClass");
    }

    #[tokio::test]
    async fn test_invalid_block_id() {
        let args = vec![
            "get",
            "class",
            "-d",
            MAP_CONTRACT_ADDRESS_SEPOLIA,
            "--block-id",
            "invalid_block",
            "--url",
            URL,
        ];

        let output = runner(&args).assert().failure();

        assert_stderr_contains(
            output,
            indoc! {r"
        Command: get class
        Error: Incorrect value passed for block_id = invalid_block. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)
        "},
        );
    }

    #[tokio::test]
    async fn test_with_alias() {
        let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
        let args = vec!["get", "class", "-d", "@map", "--url", URL];

        let output = runner(&args).current_dir(tempdir.path()).assert().success();

        let _: ContractClass = serde_json::from_str(class_output(output.as_stdout()))
            .expect("Failed to parse as ContractClass");
    }

    #[tokio::test]
    async fn test_with_unknown_alias() {
        let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
        let args = vec!["get", "class", "-d", "@unknown", "--url", URL];

        let output = runner(&args).current_dir(tempdir.path()).assert().failure();

        assert_stderr_contains(
            output,
            indoc! {r"
            Command: get class
            Error: Invalid contract address

            Caused by:
                Alias `unknown` not found in config
        "},
        );
    }
}
