use core::result::ResultTrait;

#[test]
fn invalid_module_path() {
    declare!(nonexistent::MissingContract).unwrap();
}
