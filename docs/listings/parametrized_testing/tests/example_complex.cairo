#[derive(Copy, Drop)]
struct User {
    pub name: felt252,
    pub age: u8,
}

#[generate_trait]
impl UserImpl of UserTrait {
    fn is_adult(self: @User) -> bool {
        return *self.age >= 18_u8;
    }
}

#[test]
#[test_case(User { name: 'Alice', age: 20 }, true)]
#[test_case(User { name: 'Bob', age: 14 }, false)]
#[test_case(User { name: 'Josh', age: 18 }, true)]
fn test_is_adult(user: User, expected: bool) {
    assert_eq!(user.is_adult(), expected);
}
