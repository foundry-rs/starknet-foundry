use camino::Utf8PathBuf;
use packages_validation::check_and_lint;
use project_root::get_project_root;
use test_case::test_case;

#[test_case("snforge_std")]
#[test_case("sncast_std")]
fn validate_cairo_libs(package_name: &str) {
    let package_path = get_project_root()
        .expect("Failed to get project root")
        .join(package_name);
    let package_path = package_path
        .canonicalize()
        .expect("Failed to canonicalize path");
    let package_path =
        Utf8PathBuf::from_path_buf(package_path).expect("Failed to convert to Utf8PathBuf");

    check_and_lint(package_path);
}
