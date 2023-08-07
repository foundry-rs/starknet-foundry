use array::ArrayTrait;
use cheatcodes::PrintTrait;

#[test]
fn test_print() {
    123.print();
    'aaa'.print();
    3618502788666131213697322783095070105623107215331596699973092056135872020480.print();
    let u32: u32 = 123;
    u32.print();

    let usize: usize = 1;
    usize.print();

    let mut arr = ArrayTrait::new();
    arr.append(152);
    arr.append(124);
    arr.append(149);
    arr.print();

    (1 == 5).print();
    assert(1 == 1, 'simple check');
}
