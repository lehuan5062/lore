// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use proc_macro::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Variant;

pub fn get_invoke_impl(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let variants: Vec<&Variant> = match &input.data {
        Data::Enum(enum_data) => enum_data.variants.iter().collect(),
        _ => panic!("LoreCommand should only be used on the LoreCommand enum"),
    };

    let mut inside_match = quote! {};
    for variant in variants.iter() {
        let ident = &variant.ident;
        inside_match = quote! {
            #inside_match
            #name::#ident(args) => { args.invoke_local(globals, callback).await }
        }
    }

    quote! {
        impl #name {
            pub async fn invoke_local(self, globals: crate::interface::LoreGlobalArgs, callback: crate::interface::LoreEventCallback) -> i32 {
                use crate::args::InvokableLoreArgs;
                match self {
                    #inside_match
                }
            }
        }
    }.into()
}
