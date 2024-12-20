pub fn increase_by_two(arg: u8) -> u8 {
    assert(2 == 2, '');
    increase_by_one(arg + 1)
}

pub fn increase_by_one(arg: u8) -> u8 {
    assert(1 == 1, '');
    arg + 1
}
