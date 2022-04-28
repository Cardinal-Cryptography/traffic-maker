extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

use anyhow::Result as AnyResult;
/// `quote` crate operates on `proc_macro2` objects and hence while constructing the code,
/// we have to use the same types.
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    self, spanned::Spanned, Data, DeriveInput, Error as SynError, Fields, Lit, Meta, MetaNameValue,
    Result as SynResult,
};
use thiserror::Error;

/// Errors corresponding to improper macro usage.
#[derive(Debug, Error)]
enum DeriveError {
    #[error("❌ Only structs can derive from `Event`.")]
    UnexpectedData,
    #[error("❌ Currently, structs with unnamed fields are not supported.")]
    UnnamedFields,
}

/// Checks whether `input` has `#[pallet = "<pallet_name>"]` attribute added. If so, returns
/// its value, i.e. `pallet_name`.
///
/// If there are multiple such attributes, takes into consideration only the first one.
/// Other are ignored and do not lead to `Err(_)`.
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

/// Produces boolean 'equality' formula for the struct represented by `ast`. The formula is supposed
/// to be used within a function with a signature:
/// ```no_run
///     struct Foo {
///         // ...
///     };
///     impl Foo {
///         fn f(&self, other_instance: &Self) -> bool { todo!() }
///     }
/// ```
///
/// For unit structs (without fields) it just returns `true`. Structs with anonymous fields,
/// like `struct Foo(u8)`, are not supported and thus `DeriveError::UnnamedFields` is returned.
/// They can be easily handled, but Substrate events are never of this form and thus there is
/// no usage for them.
///
/// Structs with named fields are compared field-wise using standard equality operator. However,
/// fields annotated with `#[event_ignore]` attribute are ignored.
/// Note that all other fields must implement `Eq` trait.
///
/// If `ast` does not represent `struct`, `Err(DeriveError::UnexpectedData)` is returned.
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

/// Generate implementation of `Event` trait for the type represented by `ast`. For `kind()`
/// method `pallet` and `ast.ident` values are used.
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

/// Derives `Event` trait for the type represented by `input`. For now, we only allow
/// such a derivation for structs.
///
/// The struct has to be annotated with an appropriate attribute: `#[pallet = "..."]`, which
/// indicates the origin of the event. Struct name should be identical to the event name
/// (corresponding enum variant from Substrate code).
///
/// The `matches` method is by default an equality test between two instances. However,
/// one can exclude some fields from being taken into account with the attribute `#[event_ignore]`.
/// Thus, the whole struct does not have to implement `Eq`, but its included fields must.
///
/// For example, `Balances::Transfer` event can be declared like this:
/// ```no_run
///     #[derive(Clone, Debug, Event, Decode, PartialEq, Eq)]
///     #[pallet = "Balances"]
///     pub struct Transfer {
///         from: AccountId,
///         to: AccountId,
///         amount: u128,
///     }
/// ```
/// which will be expanded to:
/// ```no_run
///     pub struct Transfer {
///         from: AccountId,
///         to: AccountId,
///         amount: u128,
///     }
///     //...
///     impl Event for Transfer {
///         fn kind(&self) -> (&'static str, &'static str) {
///             ("Balances", "Transfer")
///         }
///         fn matches(&self, other: &Self) -> bool {
///             self.from == other.from && self.to == other.to && self.amount == other.amount
///         }
///     }
/// ```
///
/// Unit structs:
/// ```no_run
///     #[derive(Debug, Clone, Event, Decode)]
///     #[pallet = "Utility"]
///     struct BatchCompleted;
/// ```
/// are expanded like:
/// ```no_run
///     struct BatchCompleted;
///     //...
///     impl Event for BatchCompleted {
///         fn kind(&self) -> (&'static str, &'static str) {
///             ("Utility", "BatchCompleted")
///         }
///         fn matches(&self, other: &Self) -> bool {
///             true
///         }
///     }
/// ```
///
/// As mentioned, you can also ignore some irrelevant fields:
/// ```no_run
///     #[derive(Debug, Clone, Event, Decode)]
///     #[pallet = "Multisig"]
///     pub struct MultisigExecuted {
///         approving: AccountId,
///         #[event_ignore]
///         timepoint: Timepoint<BlockNumber>,
///         multisig: AccountId,
///         call_hash: CallHash,
///         #[event_ignore]
///         result: DispatchResult,
///     }
/// ```
/// to obtain:
/// ```no_run
///     impl Event for MultisigExecuted {
///         fn kind(&self) -> (&'static str, &'static str) {
///             ("Multisig", "MultisigExecuted")
///         }
///         fn matches(&self, other: &Self) -> bool {
///             self.approving == other.approving
///                 && self.multisig == other.multisig
///                 && self.call_hash == other.call_hash
///         }
///     }
/// ```
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
