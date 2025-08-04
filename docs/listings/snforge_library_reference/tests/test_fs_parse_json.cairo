use snforge_std::fs::{FileTrait, FileParser};
use core::array::ArrayTrait;
use core::option::OptionTrait;
use core::serde::Serde;

#[derive(Debug, Serde, Drop, PartialEq)]
struct Location {
    city: ByteArray,
    country: ByteArray,
}

#[derive(Debug, Serde, Drop, PartialEq)]
struct User {
    age: u32,
    job: ByteArray,
    location: Location,
    name: ByteArray,
    surname: ByteArray,
}


#[test]
fn parse_json_example() {
    // Create an instance of `File` to be used later
    let file = FileTrait::new("data/user.json");

    // Parse the JSON content from the file
    let content = FileParser::<User>::parse_json(@file).expect('Failed to parse JSON');

    // Serialize the content to an array for comparison
    let mut output_array = ArrayTrait::new();
    content.serialize(ref output_array);

    println!("{:?}", content);

    assert!(content.name == "John");
    assert!(content.location.country == "USA");
    assert!(content.age == 30);
}
