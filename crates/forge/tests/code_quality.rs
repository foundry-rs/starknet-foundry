use packages_validation::validate_cairo_lib;

#[test]
fn validate_snforge_std() {
    validate_cairo_lib("snforge_std");
}
