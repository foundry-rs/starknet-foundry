use sncast::helpers::interactive::update_config;
use toml_edit::DocumentMut;

#[test]
fn test_update_config() {
    let original = r#"
    [snfoundry]
    key = 2137

    [sncast.default]
    account = "mainnet"
    url = "https://localhost:5050"

    # comment

    [sncast.testnet]
    account = "testnet-account"        # comment
    url = "https://swmansion.com/"
    "#;

    let expected = r#"
    [snfoundry]
    key = 2137

    [sncast.default]
    account = "testnet"
    url = "https://localhost:5050"

    # comment

    [sncast.testnet]
    account = "testnet-account"        # comment
    url = "https://swmansion.com/"
    "#;

    let mut toml_doc = original.parse::<DocumentMut>().unwrap();

    update_config(&mut toml_doc, "default", "account", "testnet");

    assert_eq!(toml_doc.to_string(), expected);
}
