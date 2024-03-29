extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

use anyhow::Result as AnyResult;
/// `quote` crate operates on `proc_macro2` objects and hence while constructing the code,
/// we have to use the same types.
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    self, spanned::Spanned, Attribute, Data, DeriveInput, Error as SynError, Fields, Lit, Meta,
    MetaNameValue, NestedMeta, Result as SynResult,
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

/// Internal representation of struct fields for the purpose of implementing the macro transform.
mod private {
    use proc_macro2::Span;
    use syn::{Ident, Type};

    use crate::TokenStream2;

    #[derive(Clone)]
    pub struct Field {
        pub span: Span,
        pub name: Ident,
        pub ty: Type,
        pub ignored: bool,
        pub default: Option<TokenStream2>,
    }

    #[derive(Clone)]
    pub struct Fields {
        pub relevant: Vec<Field>,
        pub ignored: Vec<Field>,
    }
}

/// If `attr` is of form `#[xxx(default = "yyy")]`, where `xxx` is some identifier, then this
/// function returns `Some("yyy")` as `TokenStream2`. Otherwise it returns `None`.
fn get_default_value(attr: &Attribute) -> Option<TokenStream2> {
    match attr.parse_meta().ok()? {
        Meta::List(meta) => {
            match meta
                .nested
                .into_iter()
                .next()
                .expect("List should not be empty")
            {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                    path,
                    lit: Lit::Str(lit_str),
                    ..
                })) => {
                    if path.is_ident("default") {
                        Some(TokenStream2::from_str(lit_str.value().as_str()).ok()?)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Returns all fields of the struct represented by `ast` divided into two sets (according to
/// `private::Fields`): relevant fields and ignored fields.
///
/// Additionally, if an ignored field has a default value specified through the
/// `#[event_match_ignore(default = "...")]` attribute, then it is read and saved.
fn get_fields(ast: &DeriveInput) -> AnyResult<private::Fields> {
    let fields = match ast.data {
        Data::Struct(ref data) => &data.fields,
        _ => return Err(DeriveError::UnexpectedData.into()),
    };

    match fields {
        Fields::Named(ref fields) => {
            let fields = fields.named.iter().map(|f| {
                let ignore_attr: Option<&Attribute> = f
                    .attrs
                    .iter()
                    .find(|a| a.path.is_ident("event_match_ignore"));
                let default = ignore_attr.and_then(get_default_value);

                private::Field {
                    span: f.span(),
                    name: f.ident.clone().expect("This is a named field"),
                    ty: f.ty.clone(),
                    ignored: ignore_attr.is_some(),
                    default,
                }
            });

            let (ignored, relevant) = fields.partition(|field| field.ignored);
            Ok(private::Fields { relevant, ignored })
        }
        Fields::Unit => Ok(private::Fields {
            relevant: vec![],
            ignored: vec![],
        }),
        Fields::Unnamed(_) => Err(DeriveError::UnnamedFields.into()),
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
/// fields annotated with `#[event_match_ignore]` attribute are ignored.
/// Note that all other fields must implement `Eq` trait.
///
/// If `ast` does not represent `struct`, `Err(DeriveError::UnexpectedData)` is returned.
fn derive_match(ast: &DeriveInput, other_instance: &TokenStream2) -> AnyResult<TokenStream2> {
    let private::Fields { relevant, .. } = get_fields(ast)?;

    if relevant.is_empty() {
        Ok(quote! {true})
    } else {
        let paired = relevant
            .into_iter()
            .map(|private::Field{span, name, ..}| quote_spanned!(span=> self.#name == #other_instance.#name))
            .collect::<Vec<_>>();
        Ok(quote! {#(#paired)&&*})
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
            fn kind() -> (&'static str, &'static str) {
                (#pallet, #variant)
            }

            fn matches(&self, #other_instance_name: &Self) -> bool {
                #derived_match
            }
        }
    })
    .into())
}

/// Generate `from_relevant_fields`: a constructor over fields *without* `#[event_match_ignore]`
/// attribute.
///
/// The ignored fields are initialized using `Default::default` or the expression passed in
/// `#[event_match_ignore = "..."]` attribute.
fn impl_constructor(ast: &DeriveInput) -> AnyResult<TokenStream> {
    use private::*;

    let name = &ast.ident;

    let Fields { relevant, ignored } = get_fields(ast)?;

    let declaration_list = relevant
        .clone()
        .into_iter()
        .map(|Field { span, name, ty, .. }| quote_spanned!(span=> #name: #ty));
    let declaration_list = quote! {#(#declaration_list),*};

    let rel_initialization_list = relevant
        .into_iter()
        .map(|Field { span, name, .. }| quote_spanned!(span=> #name));
    let ign_initialization_list = ignored.into_iter().map(
        |Field {
             span,
             name,
             default,
             ..
         }| {
            match default {
                Some(default) => quote_spanned!(span=> #name: #default),
                None => quote_spanned!(span=> #name: Default::default()),
            }
        },
    );
    let initialization_list = rel_initialization_list.chain(ign_initialization_list);
    let initialization_list = quote! {#(#initialization_list),*};

    Ok((quote! {
        impl #name {
            pub fn from_relevant_fields(#declaration_list) -> Self {
                Self { #initialization_list }
            }
        }
    })
    .into())
}

/// Derives `Event` trait for the type represented by `input`. For now, we only allow
/// such a derivation for structs. Additionally, provides `Self::from_relevant_fields` method
/// which serves as a constructor (over unignored fields).
///
/// The struct has to be annotated with an appropriate attribute: `#[pallet = "..."]`, which
/// indicates the origin of the event. Struct name should be identical to the event name
/// (corresponding enum variant from Substrate code).
///
/// The `matches` method is by default an equality test between two instances. However,
/// one can exclude some fields from being taken into account with the attribute
/// `#[event_match_ignore]`. Thus, the whole struct does not have to implement `Eq`, but its
/// included fields must.
///
/// The `from_relevant_fields` constructor requires that the ignored fields either implement
/// `Default` trait or their default value is specified with `#[event_match_ignore]` attribute.
///
/// For example, `Balances::Transfer` event can be declared like this:
/// ```no_run
///     # use chain_support::Event;
///     # use codec::Decode;
///     # type AccountId = ();
///
///     #[derive(Clone, Debug, Event, Decode)]
///     #[pallet = "Balances"]
///     struct Transfer {
///         from: AccountId,
///         to: AccountId,
///         amount: u128,
///     }
/// ```
/// which will be expanded to:
/// ```no_run
///     # use chain_support::Event;
///     # use codec::Decode;
///     # type AccountId = u128;
///
///     #[derive(Clone, Debug, Decode)]
///     struct Transfer {
///         from: AccountId,
///         to: AccountId,
///         amount: u128,
///     }
///
///     impl Event for Transfer {
///         fn kind() -> (&'static str, &'static str) {
///             ("Balances", "Transfer")
///         }
///         fn matches(&self, other: &Self) -> bool {
///             self.from == other.from && self.to == other.to && self.amount == other.amount
///         }
///     }
///
///     impl Transfer {
///         pub fn from_relevant_fields(from: AccountId, to: AccountId, amount: u128) -> Self {
///             Self { from, to, amount }
///         }
///     }
/// ```
///
/// Unit structs:
///
/// ```no_run
///     # use chain_support::Event;
///     # use codec::Decode;
///
///     #[derive(Debug, Clone, Event, Decode)]
///     #[pallet = "Utility"]
///     struct BatchCompleted;
/// ```
///
/// are expanded like:
/// ```no_run
///     # use chain_support::Event;
///     # use codec::Decode;
///
///     #[derive(Debug, Clone, Decode)]
///     struct BatchCompleted;
///
///     impl Event for BatchCompleted {
///         fn kind() -> (&'static str, &'static str) {
///             ("Utility", "BatchCompleted")
///         }
///         fn matches(&self, other: &Self) -> bool {
///             true
///         }
///     }
///
///     impl BatchCompleted {
///         pub fn from_relevant_fields() -> Self {
///             Self {}
///         }
///     }
/// ```
///
/// As mentioned, you can also ignore some irrelevant fields:
/// ```no_run
///     # use chain_support::Event;
///     # use codec::Decode;
///     # type AccountId = ();
///     # type CallHash = ();
///     # type DispatchResult = Result<(), ()>;
///     # type Timepoint = ();
///
///     #[derive(Debug, Clone, Event, Decode)]
///     #[pallet = "Multisig"]
///     struct MultisigExecuted {
///         approving: AccountId,
///         #[event_match_ignore]
///         timepoint: Timepoint,
///         multisig: AccountId,
///         call_hash: CallHash,
///         #[event_match_ignore(default = "Ok(())")]
///         result: DispatchResult,
///     }
/// ```
/// to obtain:
/// ```no_run
///     # use chain_support::Event;
///     # use codec::Decode;
///     # type AccountId = ();
///     # type CallHash = ();
///     # type DispatchResult = Result<(), ()>;
///     # type Timepoint = ();
///
///     #[derive(Debug, Clone, Decode)]
///     struct MultisigExecuted {
///         approving: AccountId,
///         timepoint: Timepoint,
///         multisig: AccountId,
///         call_hash: CallHash,
///         result: DispatchResult,
///     }
///
///     impl Event for MultisigExecuted {
///         fn kind() -> (&'static str, &'static str) {
///             ("Multisig", "MultisigExecuted")
///         }
///
///         fn matches(&self, other: &Self) -> bool {
///             self.approving == other.approving
///                 && self.multisig == other.multisig
///                 && self.call_hash == other.call_hash
///         }
///     }
///
///     impl MultisigExecuted {
///         pub fn from_relevant_fields(
///             approving: AccountId,
///             multisig: AccountId,
///             call_hash: CallHash,
///         ) -> Self {
///             Self {
///                 approving,
///                 multisig,
///                 call_hash,
///                 timepoint: Default::default(),
///                 result: Ok(()),
///             }
///         }
///     }
/// ```
#[proc_macro_derive(Event, attributes(pallet, event_match_ignore))]
pub fn event_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree that we can manipulate.
    let ast = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    // Read pallet name from `#[pallet]` attribute.
    let pallet = match check_pallet(&ast) {
        Ok(pallet) => pallet,
        Err(e) => return e.to_compile_error().into(),
    };

    // Build the trait implementation.
    let trait_impl = match impl_event(&ast, pallet) {
        Ok(implementation) => implementation,
        Err(e) => {
            return SynError::new(ast.span(), e.to_string())
                .to_compile_error()
                .into()
        }
    };

    // Build the `from_relevant_fields` constructor implementation.
    let constructor_impl = match impl_constructor(&ast) {
        Ok(constructor) => constructor,
        Err(e) => {
            return SynError::new(ast.span(), e.to_string())
                .to_compile_error()
                .into()
        }
    };

    TokenStream::from_iter(vec![trait_impl, constructor_impl])
}
