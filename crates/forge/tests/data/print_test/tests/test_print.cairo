use array::ArrayTrait;
use forge_print::PrintTrait;

#[test]
fn test_print() {
    123.print();
    'aaa'.print();
    3618502788666131213697322783095070105623107215331596699973092056135872020480.print();

    let mut arr = ArrayTrait::new();
    arr.append(152);
    arr.append(124);
    arr.append(149);
    arr.print();

    (1 == 5).print();
    assert(1 == 1, 'simple check');
}
