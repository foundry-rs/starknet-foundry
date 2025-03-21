const TEST_RESOURCE_BOUNDS_FLAGS: [&str; 12] = [
    "--l1-gas",
    "100000",
    "--l1-gas-price",
    "10000000000000",
    "--l2-gas",
    "1000000000",
    "--l2-gas-price",
    "100000000000000000000",
    "--l1-data-gas",
    "100000",
    "--l1-data-gas-price",
    "10000000000000",
];

#[must_use]
pub fn apply_test_resource_bounds_flags(mut args: Vec<&str>) -> Vec<&str> {
    args.extend(TEST_RESOURCE_BOUNDS_FLAGS);
    args
}
