fn smallest_element<T, impl TPartialOrd: PartialOrd<T>, impl TCopy: Copy<T>, impl TDrop: Drop<T>>(
    list: @Array<T>,
) -> T {
    let mut smallest = *list[0];
    let mut index = 1;

    while index < list.len() {
        if *list[index] < smallest {
            smallest = *list[index];
        }
        index = index + 1;
    }

    smallest
}

struct MyStruct {
    pub value: felt252,
}

#[test]
#[fuzzer]
#[fork(url: "http://127.0.0.1:3030", block_tag: latest)]
#[ignore]
fn call_and_invoke(_a: felt252, b: u256) {
    let list: Array<MyStruct> = array![];

    // We need to specify that we are passing a snapshot of `list` as an argument
    let s = smallest_element(@list);
    assert!(s == 3);
}
