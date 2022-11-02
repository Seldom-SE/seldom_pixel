//! Macros for `seldom_pixel`

#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Meta};

/// Derives required traits for a layer. Use as `#[px_layer]` on an item. Equivalent to
/// `#[derive(Clone, Component, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]`.
#[proc_macro_attribute]
pub fn px_layer(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = TokenStream::from(if !args.is_empty() {
        let error = match syn::parse::<Meta>(args) {
            Ok(args) => Error::new_spanned(args, "px_layer should not have arguments"),
            Err(error) => error,
        }
        .into_compile_error();

        quote! {
            #error
        }
    } else {
        quote! {
            #[derive(
                ::std::clone::Clone,
                ::bevy::prelude::Component,
                ::std::fmt::Debug,
                ::std::default::Default,
                ::std::cmp::Eq,
                ::std::cmp::Ord,
                ::std::cmp::PartialEq,
                ::std::cmp::PartialOrd
            )]
        }
    });

    output.extend(input);
    output
}
