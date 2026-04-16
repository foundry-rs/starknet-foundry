use crate::format_ident;
use crate::utils::{SyntaxNodeUtils, create_single_token};
use cairo_lang_macro::{Diagnostic, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::SyntaxNode;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::{
    GenericParam, ItemEnum, ItemStruct, ModuleItem, OptionTypeClause,
    OptionWrappedGenericParamList, SyntaxFile, Variant,
};
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_utils::Upcast;

#[must_use]
pub fn fuzzable_derive(item: &TokenStream) -> ProcMacroResult {
    let db = SimpleParserDatabase::default();
    let (parsed, diagnostics) = db.parse_token_stream(item);

    if !diagnostics.is_empty() {
        return ProcMacroResult::new(TokenStream::empty())
            .with_diagnostics(Diagnostic::error("Failed because of invalid syntax").into());
    }

    let syntax_db = db.upcast();

    for module_item in SyntaxFile::from_syntax_node(syntax_db, parsed)
        .items(syntax_db)
        .elements(syntax_db)
    {
        match module_item {
            ModuleItem::Struct(struct_item) => {
                return generate_struct_impl(&db, &struct_item);
            }
            ModuleItem::Enum(enum_item) => {
                return generate_enum_impl(&db, &enum_item);
            }
            _ => {}
        }
    }

    ProcMacroResult::new(TokenStream::empty()).with_diagnostics(
        Diagnostic::error("#[derive(Fuzzable)] can only be used on structs and enums").into(),
    )
}

fn generate_struct_impl(db: &SimpleParserDatabase, struct_item: &ItemStruct) -> ProcMacroResult {
    let name_node = struct_item.name(db).as_syntax_node();
    let generic_params = struct_item.generic_params(db);

    let (impl_name_with_generics, name_with_type_args) =
        build_generic_parts(db, name_node, generic_params);

    // Struct constructors use just the name - Cairo infers the generic args
    let name_token = SyntaxNodeWithDb::new(&name_node, db);

    let blank_fields = build_struct_fields(db, struct_item, FuzzableMethod::Blank);
    let generate_fields = build_struct_fields(db, struct_item, FuzzableMethod::Generate);

    let impl_code = quote! {
        impl #impl_name_with_generics of snforge_std::fuzzable::Fuzzable<#name_with_type_args> {
            fn blank() -> #name_with_type_args {
                #name_token { #blank_fields }
            }
            fn generate() -> #name_with_type_args {
                #name_token { #generate_fields }
            }
        }
    };

    ProcMacroResult::new(impl_code)
}

fn build_struct_fields(
    db: &SimpleParserDatabase,
    struct_item: &ItemStruct,
    method: FuzzableMethod,
) -> TokenStream {
    let method_token = create_single_token(method.method_name());
    let mut result = TokenStream::empty();

    for (i, member) in struct_item.members(db).elements(db).enumerate() {
        if i > 0 {
            result.extend(quote!(,));
        }

        let name_syntax = member.name(db).as_syntax_node();
        let name = SyntaxNodeWithDb::new(&name_syntax, db);

        let type_syntax = member.type_clause(db).ty(db).as_syntax_node();
        let ty = SyntaxNodeWithDb::new(&type_syntax, db);

        result.extend(quote!(#name: snforge_std::fuzzable::Fuzzable::<#ty>::#method_token()));
    }

    result
}

fn generate_enum_impl(db: &SimpleParserDatabase, enum_item: &ItemEnum) -> ProcMacroResult {
    let name_node = enum_item.name(db).as_syntax_node();
    let generic_params = enum_item.generic_params(db);

    let variants: Vec<Variant> = enum_item.variants(db).elements(db).collect();

    if variants.is_empty() {
        return ProcMacroResult::new(TokenStream::empty()).with_diagnostics(
            Diagnostic::error("#[derive(Fuzzable)] cannot be used on an enum with no variants")
                .into(),
        );
    }

    let (impl_name_with_generics, name_with_type_args) =
        build_generic_parts(db, name_node, generic_params);

    let blank_body = build_variant_value(db, name_node, &variants[0], FuzzableMethod::Blank);
    let generate_body = build_enum_generate(db, name_node, &variants);

    let impl_code = quote! {
        impl #impl_name_with_generics of snforge_std::fuzzable::Fuzzable<#name_with_type_args> {
            fn blank() -> #name_with_type_args {
                #blank_body
            }
            fn generate() -> #name_with_type_args {
                #generate_body
            }
        }
    };

    ProcMacroResult::new(impl_code)
}

fn build_enum_generate(
    db: &SimpleParserDatabase,
    enum_name_node: SyntaxNode,
    variants: &[Variant],
) -> TokenStream {
    if variants.len() == 1 {
        return build_variant_value(db, enum_name_node, &variants[0], FuzzableMethod::Generate);
    }

    let n_minus_one = TokenStream::new(vec![create_single_token((variants.len() - 1).to_string())]);
    let mut body = quote!(
        let __snforge_fuzz_variant_idx = snforge_std::fuzzable::generate_arg(0, #n_minus_one);
    );

    for (i, variant) in variants.iter().enumerate() {
        let variant_value =
            build_variant_value(db, enum_name_node, variant, FuzzableMethod::Generate);

        if i == 0 {
            let idx = TokenStream::new(vec![create_single_token(i.to_string())]);
            body.extend(quote!(
                if __snforge_fuzz_variant_idx == #idx { #variant_value }
            ));
        } else if i == variants.len() - 1 {
            body.extend(quote!(
                else { #variant_value }
            ));
        } else {
            let idx = TokenStream::new(vec![create_single_token(i.to_string())]);
            body.extend(quote!(
                else if __snforge_fuzz_variant_idx == #idx { #variant_value }
            ));
        }
    }

    body
}

fn build_variant_value(
    db: &SimpleParserDatabase,
    enum_name_node: SyntaxNode,
    variant: &Variant,
    method: FuzzableMethod,
) -> TokenStream {
    let method_token = create_single_token(method.method_name());
    let enum_name = SyntaxNodeWithDb::new(&enum_name_node, db);
    let variant_name_syntax = variant.name(db).as_syntax_node();
    let variant_name = SyntaxNodeWithDb::new(&variant_name_syntax, db);

    match variant.type_clause(db) {
        OptionTypeClause::Empty(_) => {
            quote!(#enum_name::#variant_name)
        }
        OptionTypeClause::TypeClause(tc) => {
            let type_syntax = tc.ty(db).as_syntax_node();
            let ty = SyntaxNodeWithDb::new(&type_syntax, db);
            quote!(#enum_name::#variant_name(snforge_std::fuzzable::Fuzzable::<#ty>::#method_token()))
        }
    }
}

/// For a non-generic type `Point`:
/// - `impl_name_with_generics`: `FuzzablePoint`
/// - `name_with_type_args`: `Point`
///
/// For a generic type `Container<T, +Debug<T>>`:
/// - `impl_name_with_generics`: `FuzzableContainer<T, +Debug<T>, +snforge_std::fuzzable::Fuzzable<T>>`
/// - `name_with_type_args`: `Container<T>`
fn build_generic_parts(
    db: &SimpleParserDatabase,
    name_node: SyntaxNode,
    generic_params: OptionWrappedGenericParamList,
) -> (TokenStream, TokenStream) {
    let name_text = name_node.get_text_without_trivia(db.upcast());
    let impl_name_token = format_ident!("Fuzzable{}Impl", name_text);
    let name_with_db = SyntaxNodeWithDb::new(&name_node, db);

    let OptionWrappedGenericParamList::WrappedGenericParamList(wgpl) = generic_params else {
        return (
            TokenStream::new(vec![impl_name_token]),
            quote!(#name_with_db),
        );
    };

    let mut impl_params = TokenStream::empty();
    let mut type_args = TokenStream::empty();
    let mut fuzzable_bounds = TokenStream::empty();

    for (i, param) in wgpl.generic_params(db).elements(db).enumerate() {
        if i > 0 {
            impl_params.extend(quote!(,));
        }
        impl_params.extend(param.to_token_stream(db));

        match param {
            GenericParam::Type(tp) => {
                let tp_name_syntax = tp.name(db).as_syntax_node();
                let tp_name = SyntaxNodeWithDb::new(&tp_name_syntax, db);

                if !type_args.is_empty() {
                    type_args.extend(quote!(,));
                }
                type_args.extend(quote!(#tp_name));

                fuzzable_bounds.extend(quote!(, +snforge_std::fuzzable::Fuzzable<#tp_name>));
            }
            GenericParam::Const(cp) => {
                let cp_name_syntax = cp.name(db).as_syntax_node();
                let cp_name = SyntaxNodeWithDb::new(&cp_name_syntax, db);

                if !type_args.is_empty() {
                    type_args.extend(quote!(,));
                }
                type_args.extend(quote!(#cp_name));
            }
            _ => {}
        }
    }

    // Fuzzable bounds are appended after all original params so user-defined
    // bounds appear first, matching Cairo's impl resolution order.
    impl_params.extend(fuzzable_bounds);

    let mut impl_name_ts = TokenStream::new(vec![impl_name_token]);
    impl_name_ts.extend(quote!(<#impl_params>));

    let mut name_ts = quote!(#name_with_db);
    if !type_args.is_empty() {
        name_ts.extend(quote!(<#type_args>));
    }

    (impl_name_ts, name_ts)
}

#[derive(Clone, Copy)]
enum FuzzableMethod {
    Blank,
    Generate,
}

impl FuzzableMethod {
    fn method_name(self) -> &'static str {
        match self {
            FuzzableMethod::Blank => "blank",
            FuzzableMethod::Generate => "generate",
        }
    }
}
