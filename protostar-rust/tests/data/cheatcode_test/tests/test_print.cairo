use array::ArrayTrait;
use starknet::testing::print;

#[test]
fn test_print() {
    let mut arr = ArrayTrait::new();
    arr.append(152);
    arr.append(124);
    arr.append(149);

    print(arr.span());

    assert(1 == 1, 'simple check');
}
