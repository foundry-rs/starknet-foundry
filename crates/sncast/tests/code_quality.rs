use camino::Utf8PathBuf;
use packages_validation::check_and_lint;

#[test]
fn validate_sncast_std() {
    let package_path = Utf8PathBuf::from("../../sncast_std")
        .canonicalize()
        .unwrap()
        .try_into()
        .unwrap();
    check_and_lint(&package_path);
}
