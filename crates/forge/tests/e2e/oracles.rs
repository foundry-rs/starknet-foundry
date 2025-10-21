use crate::e2e::common::runner::{
    BASE_FILE_PATTERNS, Package, setup_package_with_file_patterns, test_runner,
};
use indoc::indoc;
use scarb_api::ScarbCommand;

fn scarb_supports_oracles() -> bool {
    ScarbCommand::version().run().unwrap().scarb
        >= semver::Version::parse("2.12.3+nightly-2025-10-21").unwrap()
}

#[test]
fn wasm() {
    if !scarb_supports_oracles() {
        eprintln!("skipping because scarb does not fully support oracles");
        return;
    }

    let t = setup_package_with_file_patterns(
        Package::Name("wasm_oracles".to_string()),
        &[BASE_FILE_PATTERNS, &["*.wasm"]].concat(),
    );

    let expected = indoc! {r#"
        [..]Compiling[..]
    "#};

    test_runner(&t).assert().success().stdout_matches(expected);
}
