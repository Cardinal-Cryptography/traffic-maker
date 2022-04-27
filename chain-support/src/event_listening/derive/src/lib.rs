extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{
    self, spanned::Spanned, DeriveInput, Error as SynError, Lit, Meta, MetaNameValue,
    Result as SynResult,
};

#[proc_macro_derive(Event, attributes(pallet, event))]
pub fn event_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree that we can manipulate.
    let ast = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    let pallet = match check_pallet(&ast) {
        Ok(pallet) => pallet,
        Err(e) => return e.to_compile_error().into(),
    };

    // Build the trait implementation.
    impl_event(&ast, pallet)
}

fn check_pallet(input: &DeriveInput) -> SynResult<String> {
    let attr = match input.attrs.iter().position(|a| a.path.is_ident("pallet")) {
        Some(idx) => &input.attrs[idx],
        None => {
            return Err(SynError::new(
                input.ident.span(),
                "Struct should have exactly one `pallet` attribute",
            ))
        }
    };

    match attr.parse_meta()? {
        Meta::NameValue(MetaNameValue {
            lit: Lit::Str(lit_str),
            ..
        }) => Ok(lit_str.value()),
        err => Err(SynError::new(err.span(), "Invalid `pallet` attribute")),
    }
}

fn impl_event(ast: &DeriveInput, pallet: String) -> TokenStream {
    let name = &ast.ident;

    let pallet = &*pallet;
    let variant = &*name.to_string();

    (quote! {
        impl Event for #name {
            fn kind(&self) -> (&'static str, &'static str) {
                (#pallet, #variant)
            }

            fn matches(&self, _: &Self) -> bool {
                true
            }

        }
    })
    .into()
}
