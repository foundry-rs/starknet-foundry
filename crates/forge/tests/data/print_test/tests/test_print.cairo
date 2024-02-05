#[test]
fn test_print() {
    let felt252: felt252 = 123;
    println!("{}", felt252);

    let felt252_2: felt252 =
        3618502788666131213697322783095070105623107215331596699973092056135872020480;
    println!("{}", felt252_2);

    let string = 'aaa';
    println!("{}", string);

    let u8: u8 = 12;
    println!("{}", u8);

    let u16: u16 = 1234;
    println!("{}", u16);

    let u32: u32 = 123456;
    println!("{}", u32);

    let u64: u64 = 1233456789;
    println!("{}", u64);

    let u128: u128 = 123345678910;
    println!("{}", u128);

    let u256: u256 = 3618502788666131213697322783095070105623107215331596699973092056135872020480;
    println!("{}", u256);

    let usize: usize = 2;
    println!("{}", usize);

    let mut arr = ArrayTrait::new();
    arr.append(152);
    arr.append(124);
    arr.append(149);
    println!("{:?}", arr);

    let bool: bool = 1 == 5;
    println!("{}", bool);

    let esc = 27;
    println!("{}", esc);

    let dc1 = 17;
    println!("{}", dc1);

    let percent = 37;
    println!("{}", percent);

    let del = 127;
    println!("{}", del);

    let space = 32;
    println!("{}", space);

    let complex = ' % abc 123 !?>@';
    println!("{}", complex);

    let var1 = 5;
    let var2: ByteArray = "hello";
    let var3 = 5_u32;
    println!("{},{},{}", var1, var2, var3);
    print!("{var1}{var2}{var3}");
    println!("{var1:?}{var2:?}{var3:?}");

    assert(1 == 1, 'simple check');
}
