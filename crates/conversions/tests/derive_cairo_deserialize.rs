use conversions::serde::deserialize::{BufferReader, CairoDeserialize};
use starknet_types_core::felt::Felt as Felt252;

macro_rules! from_felts {
    ($($exprs:expr),*) => {
        CairoDeserialize::deserialize(&mut BufferReader::new(&[
            $(
                Felt252::from($exprs)
            ),*
        ])).unwrap()
    };
}

#[test]
fn work_on_struct() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    struct Foo {
        a: Felt252,
    }

    let value: Foo = from_felts!(123);

    assert_eq!(
        value,
        Foo {
            a: Felt252::from(123)
        }
    );
}

#[test]
fn work_on_empty_struct() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    struct Foo {}

    let value: Foo = from_felts!();

    assert_eq!(value, Foo {});
}

#[test]
fn work_on_tuple_struct() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    struct Foo(Felt252);

    let value: Foo = from_felts!(123);

    assert_eq!(value, Foo(Felt252::from(123)));
}

#[test]
fn work_on_empty_tuple_struct() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    struct Foo();

    let value: Foo = from_felts!();

    assert_eq!(value, Foo());
}

#[test]
fn work_on_unit_struct() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    struct Foo;

    let value: Foo = from_felts!();

    assert_eq!(value, Foo);
}

#[test]
fn work_on_enum() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    enum Foo {
        A,
        B(Felt252),
        C { a: Felt252 },
    }

    let value: Foo = from_felts!(0);
    assert_eq!(value, Foo::A);

    let value: Foo = from_felts!(1, 123);
    assert_eq!(value, Foo::B(Felt252::from(123)));

    let value: Foo = from_felts!(2, 123);
    assert_eq!(
        value,
        Foo::C {
            a: Felt252::from(123)
        }
    );
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseFailed")]
fn fail_on_empty_enum() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    enum Foo {}

    let _: Foo = from_felts!(0);
}

#[test]
fn work_with_nested() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    enum Foo {
        A,
        B(Felt252),
        C { a: Bar },
    }

    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    struct Bar {
        a: Felt252,
    }

    let value: Foo = from_felts!(2, 123);

    assert_eq!(
        value,
        Foo::C {
            a: Bar {
                a: Felt252::from(123)
            }
        }
    );
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: EndOfBuffer")]
fn fail_on_too_short_data() {
    #[derive(CairoDeserialize, Debug, PartialEq, Eq)]
    struct Foo {
        a: Felt252,
        b: Felt252,
    }

    let _: Foo = from_felts!(123);
}
