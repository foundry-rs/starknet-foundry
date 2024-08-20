use conversions::serde::serialize::{CairoSerialize, SerializeToFeltVec};
use starknet_types_core::felt::Felt as Felt252;

macro_rules! from_felts {
    ($($exprs:expr),*) => {
        &[
            $(
                Felt252::from($exprs)
            ),*
        ]
    };
}

#[test]
fn work_on_struct() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    struct Foo {
        a: Felt252,
    }

    let value = from_felts!(123);

    assert_eq!(
        value,
        Foo {
            a: Felt252::from(123)
        }
        .serialize_to_vec()
        .as_slice()
    );
}

#[test]
fn work_on_empty_struct() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    struct Foo {}

    let value: &[Felt252] = from_felts!();

    assert_eq!(value, Foo {}.serialize_to_vec().as_slice());
}

#[test]
fn work_on_tuple_struct() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    struct Foo(Felt252);

    let value = from_felts!(123);

    assert_eq!(value, Foo(Felt252::from(123)).serialize_to_vec().as_slice());
}

#[test]
fn work_on_empty_tuple_struct() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    struct Foo();

    let value: &[Felt252] = from_felts!();

    assert_eq!(value, Foo().serialize_to_vec().as_slice());
}

#[test]
fn work_on_unit_struct() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    struct Foo;

    let value: &[Felt252] = from_felts!();

    assert_eq!(value, Foo.serialize_to_vec().as_slice());
}

#[test]
fn work_on_enum() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    enum Foo {
        A,
        B(Felt252),
        C { a: Felt252 },
    }

    let value = from_felts!(0);
    assert_eq!(value, Foo::A.serialize_to_vec().as_slice());

    let value = from_felts!(1, 123);
    assert_eq!(
        value,
        Foo::B(Felt252::from(123)).serialize_to_vec().as_slice()
    );

    let value = from_felts!(2, 123);
    assert_eq!(
        value,
        Foo::C {
            a: Felt252::from(123)
        }
        .serialize_to_vec()
        .as_slice()
    );
}

#[test]
fn work_on_empty_enum() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    #[allow(dead_code)]
    enum Foo {}
}

#[test]
fn work_with_nested() {
    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    #[allow(dead_code)]
    enum Foo {
        A,
        B(Felt252),
        C { a: Bar },
    }

    #[derive(CairoSerialize, Debug, PartialEq, Eq)]
    struct Bar {
        a: Felt252,
    }

    let value = from_felts!(2, 123);

    assert_eq!(
        value,
        Foo::C {
            a: Bar {
                a: Felt252::from(123)
            }
        }
        .serialize_to_vec()
        .as_slice()
    );
}
