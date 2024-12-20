use coverage_project::{increase_by_one, increase_by_two};


#[test]
fn my_test() {
    assert(increase_by_two(1) == 3, ''); // inlines
    assert(increase_by_one(1) == 2, ''); // inlines
}
