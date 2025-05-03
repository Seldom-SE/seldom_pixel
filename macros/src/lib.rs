//! Macros for `seldom_pixel`

#![warn(missing_docs)]

use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Error, Expr, ExprLit, Fields,
    Ident, Index, Lit, Meta, Result,
};

fn fields_min_next(
    default_next: TokenStream,
    fields: Fields,
    container_ident: TokenStream,
) -> (TokenStream, TokenStream) {
    let fields = match fields {
        Fields::Named(fields) => fields.named,
        Fields::Unnamed(fields) => fields.unnamed,
        Fields::Unit => Punctuated::new(),
    };

    let field_count = fields.len();
    let mut next = quote! { { #default_next } };

    let field_bindings = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field
                .ident
                .clone()
                .unwrap_or_else(|| Ident::new(&format!("__field_{index}"), field.span()))
        })
        .collect::<Vec<_>>();
    let field_idents = fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            if let Some(ident) = field.ident {
                quote! { #ident }
            } else {
                let index = Index::from(index);
                quote! { #index }
            }
        })
        .collect::<Vec<_>>();

    let mut field_values = vec![quote! { ::seldom_pixel::math::Next::MIN }; field_count];

    let min = quote! { #container_ident {
        #(#field_idents: #field_values,)*
    } };

    for field in 0..field_count {
        let binding = &field_bindings[field];
        field_values[field] = quote! { next };

        next = quote! { if let ::std::option::Option::Some(
            next
        ) = ::seldom_pixel::math::Next::next(#binding) {
            ::std::option::Option::Some(#container_ident {
                #(#field_idents: #field_values,)*
            })
        } else #next };

        field_values[field] = quote! { #binding };
    }

    (min, next)
}

// TODO Move to its own crate
fn derive_next_inner(input: proc_macro::TokenStream) -> Result<TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let input_span = input.span();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = input.ident;

    let (min, next) = match input.data {
        Data::Struct(data) => {
            let field_bindings =
                data.fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let field_ident = if let Some(ref ident) = field.ident {
                            quote! { #ident }
                        } else {
                            let index = Index::from(index);
                            quote! { #index }
                        };
                        let ident = field.ident.clone().unwrap_or_else(|| {
                            Ident::new(&format!("__field_{index}"), field.span())
                        });

                        quote! { #field_ident: #ident }
                    })
                    .collect::<Vec<_>>();

            let (min, next) = fields_min_next(
                quote! { ::std::option::Option::None },
                data.fields,
                quote! { Self },
            );

            (
                min,
                quote! {
                    let Self { #(#field_bindings,)* } = self;

                    #next
                },
            )
        }
        Data::Enum(data) => {
            let mut variants = BTreeMap::new();
            let mut discriminant = 0;

            for variant in data.variants {
                match variant.discriminant {
                    Some((
                        _,
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(variant_discriminant),
                            ..
                        }),
                    )) => {
                        discriminant = variant_discriminant.base10_parse::<isize>()?;
                    }
                    Some((_, discriminant)) => {
                        return Err(Error::new_spanned(
                            discriminant,
                            "cannot derive `Next` for enum with non-literal discriminant",
                        ));
                    }
                    None => (),
                }

                let ident_span = variant.ident.span();
                if let Some((ident, _)) =
                    variants.insert(discriminant, (variant.ident, variant.fields))
                {
                    const ERR: &str = "multiple variants have the same discriminant";

                    let mut err = Error::new_spanned(ident, ERR);
                    err.combine(Error::new(ident_span, ERR));

                    return Err(err);
                }

                discriminant += 1;
            }

            let (variant_idents, variant_fields): (Vec<_>, Vec<_>) = variants.into_values().unzip();
            let variant_field_bindings = variant_fields
                .iter()
                .map(|fields| {
                    fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| {
                            let field_ident = if let Some(ref ident) = field.ident {
                                quote! { #ident }
                            } else {
                                let index = Index::from(index);
                                quote! { #index }
                            };
                            let ident = field.ident.clone().unwrap_or_else(|| {
                                Ident::new(&format!("__field_{index}"), field.span())
                            });

                            quote! { #field_ident: #ident }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let mut last_min = None;

            let mut variant_nexts = variant_fields
                .into_iter()
                .enumerate()
                .rev()
                .map(|(variant, fields)| {
                    let ident = &variant_idents[variant];

                    let (min, next) = fields_min_next(
                        if let Some(last_min) = last_min.take() {
                            quote! { ::std::option::Option::Some(#last_min) }
                        } else {
                            quote! { ::std::option::Option::None }
                        },
                        fields,
                        quote! { Self::#ident },
                    );
                    last_min = Some(min);
                    next
                })
                .collect::<Vec<_>>();

            variant_nexts.reverse();

            (
                last_min.ok_or_else(|| {
                    Error::new(input_span, "cannot derive `Next` for uninhabited type")
                })?,
                quote! {
                    match self {
                        #(Self::#variant_idents { #(#variant_field_bindings,)* } => #variant_nexts)*
                    }
                },
            )
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "cannot derive `Next` for union",
            ))
        }
    };

    Ok(quote! {
        #[automatically_derived]
        #[allow(non_shorthand_field_patterns)]
        impl #impl_generics ::seldom_pixel::math::Next for #ident #ty_generics #where_clause {
            const MIN: Self = #min;

            fn next(self) -> ::std::option::Option<Self> {
                #next
            }
        }
    })
}

#[proc_macro_derive(Next)]
pub fn derive_next(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_next_inner(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

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
                ::seldom_pixel::math::Next,
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
