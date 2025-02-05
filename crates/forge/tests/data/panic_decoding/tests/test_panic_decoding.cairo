use core::array::ArrayTrait;

#[test]
fn test_simple() {
    assert(1 == 1, 'simple check');
}

#[test]
fn test_panic_decoding() {
    let max_felt = 3618502788666131213697322783095070105623107215331596699973092056135872020480;

    let mut arr = ArrayTrait::new();
    arr.append(123);
    arr.append('aaa');
    arr.append(max_felt);
    arr.append(152);
    arr.append(124);
    arr.append(149);
    panic(arr);
}

#[test]
fn test_panic_decoding2() {
    assert(1 == 2, 128);
}

#[test]
fn test_simple2() {
    assert(2 == 2, 'simple check');
}

#[test]
fn test_assert_eq() {
    let x = 5;
    let y = 6;
    assert_eq!(x, y);
}

#[test]
fn test_assert_eq_message() {
    let x = 5;
    let y = 6;
    assert_eq!(x, y, "An identifiable and meaningful error message");
}

#[test]
fn test_assert() {
    let x = false;
    assert!(x);
}

#[test]
fn test_assert_message() {
    let x = false;
    assert!(x, "Another identifiable and meaningful error message");
}
