use using_corelib_names::bits::bits;
use using_corelib_names::math::math;
use using_corelib_names::test::test;
use using_corelib_names::types::types;

#[test]
fn test_names() {
    assert(bits() == 1, '');
    assert(math() == 2, '');
    assert(test() == 3, '');
    assert(types() == 4, '');
}
