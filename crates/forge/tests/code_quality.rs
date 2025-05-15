use camino::Utf8PathBuf;
use packages_validation::check_and_lint;

#[test]
fn validate_snforge_std() {
    let package_path: Utf8PathBuf = std::env::current_dir()
        .expect("Failed to get current directory")
        .ancestors()
        .nth(2)
        .expect("Failed to get grandparent directory")
        .join("snforge_std")
        .canonicalize()
        .expect("Failed to canonicalize path")
        .try_into()
        .expect("Failed to convert to Utf8PathBuf");

    check_and_lint(&package_path);
}
