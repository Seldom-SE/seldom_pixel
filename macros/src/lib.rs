//! Macros for `seldom_pixel`

#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Meta};

/// Derives required traits for a layer. Use as `#[px_layer]` on an item. Equivalent to
/// `#[derive(ExtractComponent, Component, Ord, PartialOrd, Eq, PartialEq, Clone, Default, Debug)]`.
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
                ::bevy::render::extract_component::ExtractComponent,
                ::bevy::prelude::Component,
                ::std::cmp::Ord,
                ::std::cmp::PartialOrd,
                ::std::cmp::Eq,
                ::std::cmp::PartialEq,
                ::std::clone::Clone,
                ::std::default::Default,
                ::std::fmt::Debug,
            )]
        }
    });

    output.extend(input);
    output
}
