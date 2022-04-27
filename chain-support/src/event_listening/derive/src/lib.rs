extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

use anyhow::Result as AnyResult;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    self, spanned::Spanned, Data, DeriveInput, Error as SynError, Fields, Lit, Meta, MetaNameValue,
    Result as SynResult,
};
use thiserror::Error;

#[derive(Debug, Error)]
enum DeriveError {
    #[error("❌ Only structs can derive from `Event`.")]
    UnexpectedData,
    #[error("❌ Currently, structs with unnamed fields are not supported.")]
    UnnamedFields,
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

fn derive_match(ast: &DeriveInput, other_instance: &TokenStream2) -> AnyResult<TokenStream2> {
    let fields = match ast.data {
        Data::Struct(ref data) => &data.fields,
        _ => return Err(DeriveError::UnexpectedData.into()),
    };

    match fields {
        Fields::Named(ref fields) => {
            let relevant = fields
                .named
                .iter()
                .filter(|f| !f.attrs.iter().any(|a| a.path.is_ident("event_ignore")));

            let paired = relevant.map(|f| {
                let name = f.ident.clone().expect("This is a named field");
                quote_spanned!(f.span()=> self.#name == #other_instance.#name)
            });

            Ok(quote! {#(#paired)&&*})
        }
        Fields::Unit => Ok(quote! {true}),
        Fields::Unnamed(_) => Err(DeriveError::UnnamedFields.into()),
    }
}

fn impl_event(ast: &DeriveInput, pallet: String) -> AnyResult<TokenStream> {
    let name = &ast.ident;

    let pallet = &*pallet;
    let variant = &*name.to_string();

    let other_instance_name = TokenStream2::from_str("other").unwrap();
    let derived_match = derive_match(ast, &other_instance_name)?;

    Ok((quote! {
        impl Event for #name {
            fn kind(&self) -> (&'static str, &'static str) {
                (#pallet, #variant)
            }

            fn matches(&self, #other_instance_name: &Self) -> bool {
                #derived_match
            }

        }
    })
    .into())
}

#[proc_macro_derive(Event, attributes(pallet, event_ignore))]
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
    match impl_event(&ast, pallet) {
        Ok(implementation) => implementation,
        Err(e) => SynError::new(ast.span(), e.to_string())
            .to_compile_error()
            .into(),
    }
}
