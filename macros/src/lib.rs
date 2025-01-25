//! A tool to securely back up files and directories.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(unused_mut)]
#![warn(clippy::missing_docs_in_private_items)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::if_not_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

/// Derives a method that returns a class name representation of each variant on
/// the enum. Intended for enums comprised solely of unit variants.
#[proc_macro_derive(ClassName)]
pub fn derive_class_name(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemEnum);

    let ident = &item.ident;
    let variants = item
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let class_name = variant.ident.to_string().to_case(Case::Kebab);

            quote! {
                Self::#variant_ident => #class_name,
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl #ident {
            /// Returns the class name representation of the currently active
            /// variant.
            pub const fn class_name(&self) -> &'static str {
                match self {
                    #(#variants)*
                }
            }
        }
    }
    .into()
}
