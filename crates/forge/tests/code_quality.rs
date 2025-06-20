use camino::Utf8PathBuf;
use packages_validation::check_and_lint;

#[test]
fn validate_snforge_std() {
    let package_path = Utf8PathBuf::from("../../snforge_std")
        .canonicalize()
        .unwrap()
        .try_into()
        .unwrap();
    check_and_lint(&package_path);
}
