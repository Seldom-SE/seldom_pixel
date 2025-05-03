//! Macros for `seldom_pixel`

#![warn(missing_docs)]

use quote::quote;
use syn::{Error, Meta};

/// Derives required traits for a layer. Use as `#[px_layer]` on an item. Equivalent to
/// `#[derive(ExtractComponent, Component, Next, Ord, PartialOrd, Eq, PartialEq, Clone, Default, Debug)]`.
#[proc_macro_attribute]
pub fn px_layer(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut output = proc_macro::TokenStream::from(if !args.is_empty() {
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
                ::seldom_pixel::prelude::Next,
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
