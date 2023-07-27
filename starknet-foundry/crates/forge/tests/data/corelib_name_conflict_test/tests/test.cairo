use corelib_name_conflict_test::bits::bits;
use corelib_name_conflict_test::math::math;
use corelib_name_conflict_test::test::test;
use corelib_name_conflict_test::types::types;

#[test]
fn test_names() {
    assert(bits() == 1, '');
    assert(math() == 2, '');
    assert(test() == 3, '');
    assert(types() == 4, '');
}
