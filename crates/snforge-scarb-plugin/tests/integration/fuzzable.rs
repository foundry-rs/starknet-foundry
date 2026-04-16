use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{Diagnostic, Severity, quote};
use snforge_scarb_plugin::derives::fuzzable::fuzzable_derive;

#[test]
fn struct_no_fields() {
    let item = quote!(
        struct Empty {}
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableEmptyImpl of snforge_std::fuzzable::Fuzzable<Empty> {
                fn blank() -> Empty {
                    Empty {}
                }
                fn generate() -> Empty {
                    Empty {}
                }
            }
        ",
    );
}

#[test]
fn struct_with_fields() {
    let item = quote!(
        struct Point {
            x: u64,
            y: u64,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzablePointImpl of snforge_std::fuzzable::Fuzzable<Point> {
                fn blank() -> Point {
                    Point {
                        x: snforge_std::fuzzable::Fuzzable::<u64>::blank(),
                        y: snforge_std::fuzzable::Fuzzable::<u64>::blank()
                    }
                }
                fn generate() -> Point {
                    Point {
                        x: snforge_std::fuzzable::Fuzzable::<u64>::generate(),
                        y: snforge_std::fuzzable::Fuzzable::<u64>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn struct_with_generic_type_param() {
    let item = quote!(
        struct Container<T, +core::fmt::Debug<T>> {
            value: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableContainerImpl<T, +core::fmt::Debug<T>, +snforge_std::fuzzable::Fuzzable<T>> of snforge_std::fuzzable::Fuzzable<Container<T>> {
                fn blank() -> Container<T> {
                    Container {
                        value: snforge_std::fuzzable::Fuzzable::<T>::blank()
                    }
                }
                fn generate() -> Container<T> {
                    Container {
                        value: snforge_std::fuzzable::Fuzzable::<T>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_single_unit_variant() {
    let item = quote!(
        enum Single {
            Only,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableSingleImpl of snforge_std::fuzzable::Fuzzable<Single> {
                fn blank() -> Single {
                    Single::Only
                }
                fn generate() -> Single {
                    Single::Only
                }
            }
        ",
    );
}

#[test]
fn enum_multiple_unit_variants() {
    let item = quote!(
        enum Direction {
            North,
            South,
            East,
            West,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableDirectionImpl of snforge_std::fuzzable::Fuzzable<Direction> {
                fn blank() -> Direction {
                    Direction::North
                }
                fn generate() -> Direction {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 3);
                    if __snforge_fuzz_variant_idx == 0 {
                        Direction::North
                    } else if __snforge_fuzz_variant_idx == 1 {
                        Direction::South
                    } else if __snforge_fuzz_variant_idx == 2 {
                        Direction::East
                    } else {
                        Direction::West
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_with_data_variants() {
    let item = quote!(
        enum Color {
            Red: u8,
            Green: u8,
            Blue: u8,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableColorImpl of snforge_std::fuzzable::Fuzzable<Color> {
                fn blank() -> Color {
                    Color::Red(snforge_std::fuzzable::Fuzzable::<u8>::blank())
                }
                fn generate() -> Color {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 2);
                    if __snforge_fuzz_variant_idx == 0 {
                        Color::Red(snforge_std::fuzzable::Fuzzable::<u8>::generate())
                    } else if __snforge_fuzz_variant_idx == 1 {
                        Color::Green(snforge_std::fuzzable::Fuzzable::<u8>::generate())
                    } else {
                        Color::Blue(snforge_std::fuzzable::Fuzzable::<u8>::generate())
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_mixed_variants() {
    let item = quote!(
        enum Mixed {
            Unit,
            WithData: u64,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableMixedImpl of snforge_std::fuzzable::Fuzzable<Mixed> {
                fn blank() -> Mixed {
                    Mixed::Unit
                }
                fn generate() -> Mixed {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 1);
                    if __snforge_fuzz_variant_idx == 0 {
                        Mixed::Unit
                    } else {
                        Mixed::WithData(snforge_std::fuzzable::Fuzzable::<u64>::generate())
                    }
                }
            }
        ",
    );
}

#[test]
fn error_on_empty_enum() {
    let item = quote!(
        enum Empty {}
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(
        &result,
        &[Diagnostic::new(
            Severity::Error,
            "#[derive(Fuzzable)] cannot be used on an enum with no variants".to_string(),
        )],
    );
}

#[test]
fn error_on_non_struct_enum() {
    let item = quote!(
        fn some_function() {}
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(
        &result,
        &[Diagnostic::new(
            Severity::Error,
            "#[derive(Fuzzable)] can only be used on structs and enums".to_string(),
        )],
    );
}

#[test]
fn error_on_trait_item() {
    let item = quote!(
        trait Fooable {
            fn foo() -> u64;
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(
        &result,
        &[Diagnostic::new(
            Severity::Error,
            "#[derive(Fuzzable)] can only be used on structs and enums".to_string(),
        )],
    );
}

#[test]
fn error_on_type_alias() {
    let item = quote!(
        type MyU64 = u64;
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(
        &result,
        &[Diagnostic::new(
            Severity::Error,
            "#[derive(Fuzzable)] can only be used on structs and enums".to_string(),
        )],
    );
}

#[test]
fn error_on_impl_block() {
    let item = quote!(
        impl SomeTrait of OtherTrait {}
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(
        &result,
        &[Diagnostic::new(
            Severity::Error,
            "#[derive(Fuzzable)] can only be used on structs and enums".to_string(),
        )],
    );
}

#[test]
fn error_on_use_statement() {
    let item = quote!(
        use core::integer;
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(
        &result,
        &[Diagnostic::new(
            Severity::Error,
            "#[derive(Fuzzable)] can only be used on structs and enums".to_string(),
        )],
    );
}

#[test]
fn struct_single_field() {
    let item = quote!(
        struct Wrapper {
            value: u32,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableWrapperImpl of snforge_std::fuzzable::Fuzzable<Wrapper> {
                fn blank() -> Wrapper {
                    Wrapper {
                        value: snforge_std::fuzzable::Fuzzable::<u32>::blank()
                    }
                }
                fn generate() -> Wrapper {
                    Wrapper {
                        value: snforge_std::fuzzable::Fuzzable::<u32>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn struct_multiple_generic_type_params() {
    let item = quote!(
        struct Pair<T, U> {
            first: T,
            second: U,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzablePairImpl<T, U, +snforge_std::fuzzable::Fuzzable<T>, +snforge_std::fuzzable::Fuzzable<U>> of snforge_std::fuzzable::Fuzzable<Pair<T, U>> {
                fn blank() -> Pair<T, U> {
                    Pair {
                        first: snforge_std::fuzzable::Fuzzable::<T>::blank(),
                        second: snforge_std::fuzzable::Fuzzable::<U>::blank()
                    }
                }
                fn generate() -> Pair<T, U> {
                    Pair {
                        first: snforge_std::fuzzable::Fuzzable::<T>::generate(),
                        second: snforge_std::fuzzable::Fuzzable::<U>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn struct_with_multiple_bounds_on_same_type_param() {
    let item = quote!(
        struct Fancy<T, +core::fmt::Debug<T>, +Drop<T>> {
            value: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableFancyImpl<T, +core::fmt::Debug<T>, +Drop<T>, +snforge_std::fuzzable::Fuzzable<T>> of snforge_std::fuzzable::Fuzzable<Fancy<T>> {
                fn blank() -> Fancy<T> {
                    Fancy {
                        value: snforge_std::fuzzable::Fuzzable::<T>::blank()
                    }
                }
                fn generate() -> Fancy<T> {
                    Fancy {
                        value: snforge_std::fuzzable::Fuzzable::<T>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn struct_with_nested_generic_field() {
    let item = quote!(
        struct Container {
            inner: Option<u64>,
            count: u32,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableContainerImpl of snforge_std::fuzzable::Fuzzable<Container> {
                fn blank() -> Container {
                    Container {
                        inner: snforge_std::fuzzable::Fuzzable::<Option<u64>>::blank(),
                        count: snforge_std::fuzzable::Fuzzable::<u32>::blank()
                    }
                }
                fn generate() -> Container {
                    Container {
                        inner: snforge_std::fuzzable::Fuzzable::<Option<u64>>::generate(),
                        count: snforge_std::fuzzable::Fuzzable::<u32>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_single_data_variant() {
    let item = quote!(
        enum Wrapped {
            Value: u64,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableWrappedImpl of snforge_std::fuzzable::Fuzzable<Wrapped> {
                fn blank() -> Wrapped {
                    Wrapped::Value(snforge_std::fuzzable::Fuzzable::<u64>::blank())
                }
                fn generate() -> Wrapped {
                    Wrapped::Value(snforge_std::fuzzable::Fuzzable::<u64>::generate())
                }
            }
        ",
    );
}

#[test]
fn enum_two_variants() {
    let item = quote!(
        enum Bool {
            False,
            True,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableBoolImpl of snforge_std::fuzzable::Fuzzable<Bool> {
                fn blank() -> Bool {
                    Bool::False
                }
                fn generate() -> Bool {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 1);
                    if __snforge_fuzz_variant_idx == 0 {
                        Bool::False
                    } else {
                        Bool::True
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_three_variants_with_data() {
    let item = quote!(
        enum Shape {
            Circle: u64,
            Square: u64,
            Triangle: u64,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableShapeImpl of snforge_std::fuzzable::Fuzzable<Shape> {
                fn blank() -> Shape {
                    Shape::Circle(snforge_std::fuzzable::Fuzzable::<u64>::blank())
                }
                fn generate() -> Shape {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 2);
                    if __snforge_fuzz_variant_idx == 0 {
                        Shape::Circle(snforge_std::fuzzable::Fuzzable::<u64>::generate())
                    } else if __snforge_fuzz_variant_idx == 1 {
                        Shape::Square(snforge_std::fuzzable::Fuzzable::<u64>::generate())
                    } else {
                        Shape::Triangle(snforge_std::fuzzable::Fuzzable::<u64>::generate())
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_with_generic_params() {
    let item = quote!(
        enum Maybe<T, +core::fmt::Debug<T>> {
            Nothing,
            Just: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableMaybeImpl<T, +core::fmt::Debug<T>, +snforge_std::fuzzable::Fuzzable<T>> of snforge_std::fuzzable::Fuzzable<Maybe<T>> {
                fn blank() -> Maybe<T> {
                    Maybe::Nothing
                }
                fn generate() -> Maybe<T> {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 1);
                    if __snforge_fuzz_variant_idx == 0 {
                        Maybe::Nothing
                    } else {
                        Maybe::Just(snforge_std::fuzzable::Fuzzable::<T>::generate())
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_with_nested_generic_data_variant() {
    let item = quote!(
        enum Nested {
            Empty,
            Inner: Option<u64>,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableNestedImpl of snforge_std::fuzzable::Fuzzable<Nested> {
                fn blank() -> Nested {
                    Nested::Empty
                }
                fn generate() -> Nested {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 1);
                    if __snforge_fuzz_variant_idx == 0 {
                        Nested::Empty
                    } else {
                        Nested::Inner(snforge_std::fuzzable::Fuzzable::<Option<u64>>::generate())
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_with_empty_variant_list() {
    let item = quote!(
        enum Empty {}
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(
        &result,
        &[Diagnostic::new(
            Severity::Error,
            "#[derive(Fuzzable)] cannot be used on an enum with no variants".to_string(),
        )],
    );
}

#[test]
fn struct_with_only_impl_bound_generic() {
    let item = quote!(
        struct Frozen<+core::fmt::Debug<u64>> {
            value: u64,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableFrozenImpl<+core::fmt::Debug<u64>> of snforge_std::fuzzable::Fuzzable<Frozen> {
                fn blank() -> Frozen {
                    Frozen {
                        value: snforge_std::fuzzable::Fuzzable::<u64>::blank()
                    }
                }
                fn generate() -> Frozen {
                    Frozen {
                        value: snforge_std::fuzzable::Fuzzable::<u64>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn struct_with_named_impl_param() {
    let item = quote!(
        struct Wrapper<T, impl D: core::fmt::Debug<T>> {
            value: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableWrapperImpl<T, impl D: core::fmt::Debug<T>, +snforge_std::fuzzable::Fuzzable<T>> of snforge_std::fuzzable::Fuzzable<Wrapper<T>> {
                fn blank() -> Wrapper<T> {
                    Wrapper {
                        value: snforge_std::fuzzable::Fuzzable::<T>::blank()
                    }
                }
                fn generate() -> Wrapper<T> {
                    Wrapper {
                        value: snforge_std::fuzzable::Fuzzable::<T>::generate()
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_with_multiple_type_params() {
    let item = quote!(
        enum Either<L, R> {
            Left: L,
            Right: R,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableEitherImpl<L, R, +snforge_std::fuzzable::Fuzzable<L>, +snforge_std::fuzzable::Fuzzable<R>> of snforge_std::fuzzable::Fuzzable<Either<L, R>> {
                fn blank() -> Either<L, R> {
                    Either::Left(snforge_std::fuzzable::Fuzzable::<L>::blank())
                }
                fn generate() -> Either<L, R> {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 1);
                    if __snforge_fuzz_variant_idx == 0 {
                        Either::Left(snforge_std::fuzzable::Fuzzable::<L>::generate())
                    } else {
                        Either::Right(snforge_std::fuzzable::Fuzzable::<R>::generate())
                    }
                }
            }
        ",
    );
}

#[test]
fn enum_with_multiple_bounds_on_type_param() {
    let item = quote!(
        enum Bounded<T, +core::fmt::Debug<T>, +Drop<T>> {
            Value: T,
            Empty,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            impl FuzzableBoundedImpl<T, +core::fmt::Debug<T>, +Drop<T>, +snforge_std::fuzzable::Fuzzable<T>> of snforge_std::fuzzable::Fuzzable<Bounded<T>> {
                fn blank() -> Bounded<T> {
                    Bounded::Value(snforge_std::fuzzable::Fuzzable::<T>::blank())
                }
                fn generate() -> Bounded<T> {
                    let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, 1);
                    if __snforge_fuzz_variant_idx == 0 {
                        Bounded::Value(snforge_std::fuzzable::Fuzzable::<T>::generate())
                    } else {
                        Bounded::Empty
                    }
                }
            }
        ",
    );
}
