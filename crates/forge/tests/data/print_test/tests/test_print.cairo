use array::ArrayTrait;
use cheatcodes::PrintTrait;

#[test]
fn test_print() {
    123.print();
    'aaa'.print();
    3618502788666131213697322783095070105623107215331596699973092056135872020480.print();

    let u32: u32 = 123456;
    u32.print();

    let u64: u64 = 1233456789;
    u64.print();

    let u128: u128 = 123345678910;
    u128.print();

    let u256: u256 = 3618502788666131213697322783095070105623107215331596699973092056135872020480;
    u256.print();

    let usize: usize = 1;
    usize.print();

    let i32: i32 = 123456;
    i32.print();

    let i64: i64 = 123456789;
    i64.print();

    let i128: i128 = 12345612342;
    i128.print();

    let mut arr = ArrayTrait::new();
    arr.append(152);
    arr.append(124);
    arr.append(149);
    arr.print();

    (1 == 5).print();
    assert(1 == 1, 'simple check');
}
