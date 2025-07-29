use camino::Utf8PathBuf;
use packages_validation::check_and_lint;
use test_utils::use_snforge_std_deprecated;

#[test]
fn validate_snforge_std() {
    let package_path = Utf8PathBuf::from("../../snforge_std")
        .canonicalize()
        .unwrap()
        .try_into()
        .unwrap();
    if !use_snforge_std_deprecated() {
        check_and_lint(&package_path);
    }
}
