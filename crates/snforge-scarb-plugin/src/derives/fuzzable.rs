use crate::format_ident;
use crate::utils::{SyntaxNodeUtils, create_single_token};
use cairo_lang_macro::{Diagnostic, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::Terminal;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::{
    GenericParam, ItemEnum, ItemStruct, ModuleItem, OptionTypeClause,
    OptionWrappedGenericParamList, SyntaxFile, TerminalIdentifier, Variant,
};
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

    SyntaxFile::from_syntax_node(syntax_db, parsed)
        .items(syntax_db)
        .elements(syntax_db)
        .find_map(|module_item| match module_item {
            ModuleItem::Struct(struct_item) => Some(generate_struct_impl(&db, &struct_item)),
            ModuleItem::Enum(enum_item) => Some(generate_enum_impl(&db, &enum_item)),
            _ => None,
        })
        .unwrap_or_else(|| {
            ProcMacroResult::new(TokenStream::empty()).with_diagnostics(
                Diagnostic::error("#[derive(Fuzzable)] can only be used on structs and enums")
                    .into(),
            )
        })
}

fn generate_struct_impl(db: &SimpleParserDatabase, struct_item: &ItemStruct) -> ProcMacroResult {
    let struct_name = struct_item.name(db);

    let (impl_name_with_generics, name_with_type_args) =
        build_generic_parts(db, &struct_name, struct_item.generic_params(db));

    let blank_fields = build_struct_fields(db, struct_item, FuzzableMethod::Blank);
    let generate_fields = build_struct_fields(db, struct_item, FuzzableMethod::Generate);

    let struct_name = struct_name.to_token_stream(db);

    let impl_code = quote! {
        impl #impl_name_with_generics of snforge_std::fuzzable::Fuzzable<#name_with_type_args> {
            fn blank() -> #name_with_type_args {
                #struct_name { #blank_fields }
            }
            fn generate() -> #name_with_type_args {
                #struct_name { #generate_fields }
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
    for member in struct_item.members(db).elements(db) {
        let name = member.name(db).to_token_stream(db);
        let ty = member.type_clause(db).ty(db).to_token_stream(db);
        append_comma_separated(
            &mut result,
            quote!(#name: snforge_std::fuzzable::Fuzzable::<#ty>::#method_token()),
        );
    }

    result
}

fn generate_enum_impl(db: &SimpleParserDatabase, enum_item: &ItemEnum) -> ProcMacroResult {
    let variants: Vec<_> = enum_item.variants(db).elements(db).collect();

    if variants.is_empty() {
        // Empty enum is a valid enum in Cairo, but it cannot be constructed.
        return ProcMacroResult::new(TokenStream::empty()).with_diagnostics(
            Diagnostic::error("#[derive(Fuzzable)] cannot be used on an enum with no variants")
                .into(),
        );
    }

    let enum_name = enum_item.name(db);
    let (impl_name_with_generics, name_with_type_args) =
        build_generic_parts(db, &enum_name, enum_item.generic_params(db));

    let enum_name = enum_name.to_token_stream(db);
    let generate_body = build_enum_generate(db, &enum_name, &variants);
    let blank_body = build_variant_value(db, &enum_name, &variants[0], FuzzableMethod::Blank);

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

/// NOTE: `variants` must be non-empty (caller must guard this).
fn build_enum_generate(
    db: &SimpleParserDatabase,
    enum_name: &TokenStream,
    variants: &[Variant],
) -> TokenStream {
    let match_body = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let variant_value =
                build_variant_value(db, enum_name, variant, FuzzableMethod::Generate);
            let idx = int_token(idx);
            quote!(#idx => { #variant_value },)
        })
        .fold(TokenStream::empty(), |mut acc, x| {
            acc.extend(x);
            acc
        });

    let n_minus_one = int_token(variants.len() - 1);

    quote!(
        let variant_idx = snforge_std::fuzzable::generate_arg(0, #n_minus_one);
        match variant_idx {
            #match_body
            _ => panic!("unexpected value of generate_arg cheatcode")
        }
    )
}

fn int_token(n: usize) -> TokenStream {
    TokenStream::new(vec![create_single_token(n.to_string())])
}

fn build_variant_value(
    db: &SimpleParserDatabase,
    enum_name: &TokenStream,
    variant: &Variant,
    method: FuzzableMethod,
) -> TokenStream {
    let method_token = create_single_token(method.method_name());
    let variant_name = variant.name(db).to_token_stream(db);
    let enum_name = enum_name.clone();

    match variant.type_clause(db) {
        OptionTypeClause::Empty(_) => {
            quote!(#enum_name::#variant_name)
        }
        OptionTypeClause::TypeClause(tc) => {
            let ty = tc.ty(db).to_token_stream(db);
            quote!(#enum_name::#variant_name(snforge_std::fuzzable::Fuzzable::<#ty>::#method_token()))
        }
    }
}

/// For a non-generic type `Regular` returns a tuple:
/// - `FuzzableRegularImpl<>`
/// - `Regular<>`
///
/// For a generic type `Generic<T, +SomeTrait<T>>` returns a tuple:
/// - `FuzzableGenericImpl<T, +snforge_std::fuzzable::Fuzzable<T>, +core::fmt::Debug<T>, +SomeTrait<T>>`
/// - `Generic<T, _>`
fn build_generic_parts(
    db: &SimpleParserDatabase,
    name_node: &TerminalIdentifier,
    generic_params: OptionWrappedGenericParamList,
) -> (TokenStream, TokenStream) {
    let params: Vec<_> = match generic_params {
        OptionWrappedGenericParamList::Empty(_) => Vec::new(),
        OptionWrappedGenericParamList::WrappedGenericParamList(wgpl) => {
            wgpl.generic_params(db).elements(db).collect()
        }
    };

    let mut impl_generic_params = TokenStream::empty();
    let mut type_generic_params = TokenStream::empty();

    for param in &params {
        append_comma_separated(&mut impl_generic_params, param.to_token_stream(db));

        match param {
            GenericParam::Type(tp) => {
                append_comma_separated(&mut type_generic_params, param.to_token_stream(db));

                let tp_name = tp.name(db).to_token_stream(db);

                impl_generic_params.extend(quote!(, +snforge_std::fuzzable::Fuzzable<#tp_name>));
                impl_generic_params.extend(quote!(, +core::fmt::Debug<#tp_name>));
            }
            // Impl params still occupy a generic arg slot in the type,
            // so we emit `_` to keep arg counts aligned.
            GenericParam::ImplNamed(_) | GenericParam::ImplAnonymous(_) => {
                append_comma_separated(&mut type_generic_params, quote!(_));
            }
            GenericParam::Const(cst) => {
                append_comma_separated(&mut type_generic_params, cst.name(db).to_token_stream(db));
            }
            // It is invalid semantically, so we do not care:
            // `Negative impls supported only in impl definitions.(E2078)`.
            GenericParam::NegativeImpl(_) => {}
        }
    }

    let impl_name_token = format_ident!("Fuzzable{}Impl", name_node.text(db));
    let impl_name_ts = quote!(#impl_name_token<#impl_generic_params>);

    let name_with_db = name_node.to_token_stream(db);
    let name_ts = quote!(#name_with_db<#type_generic_params>);

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

fn append_comma_separated(target: &mut TokenStream, item: TokenStream) {
    if !target.is_empty() {
        target.extend(quote!(,));
    }
    target.extend(item);
}
