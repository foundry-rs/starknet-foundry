use crate::utils::{assert_diagnostics, format_output};
use cairo_lang_macro::{Diagnostic, Severity, quote};
use snforge_scarb_plugin::derives::fuzzable::fuzzable_derive;

#[test]
fn struct_no_fields() {
    let item = quote!(
        struct Empty {}
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn struct_with_generic_type_param() {
    let item = quote!(
        struct Container<T, +core::clone::Clone<T>> {
            value: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
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
fn error_on_non_struct_nor_enum() {
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
fn struct_single_field() {
    let item = quote!(
        struct Wrapper {
            value: u32,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn struct_with_impl_bound_between_type_params() {
    let item = quote!(
        struct MultiField<T, +core::clone::Clone<T>, A> {
            t: T,
            a: A,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn struct_with_multiple_bounds_on_same_type_param() {
    let item = quote!(
        struct Fancy<T, +core::clone::Clone<T>, +Drop<T>> {
            value: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn struct_with_const_generic_param() {
    let item = quote!(
        struct WithConst<T, const X: u8> {
            value: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn enum_multiple_variants() {
    let item = quote!(
        enum Shape {
            Circle: bool,
            Square: u8,
            Triangle: u64,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn enum_with_generic_params() {
    let item = quote!(
        enum Maybe<T, +core::clone::Clone<T>> {
            Nothing,
            Just: T,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
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

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn enum_with_multiple_bounds_on_type_param() {
    let item = quote!(
        enum Bounded<T, +core::clone::Clone<T>, +Drop<T>> {
            Value: T,
            Empty,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn enum_with_const_generic_param() {
    let item = quote!(
        enum WithConst<T, const X: u8> {
            Value: T,
            Empty,
        }
    );

    let result = fuzzable_derive(&item);

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}
