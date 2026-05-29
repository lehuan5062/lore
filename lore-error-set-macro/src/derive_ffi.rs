// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Derive macro implementation for `FfiError`.
//!
//! Parses `#[ffi_code(N)]` on a struct or enum and generates an
//! `impl FfiError` that returns the given integer constant.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_ffi_error(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let code = match extract_ffi_code(input) {
        Ok(lit) => lit,
        Err(err) => return err.to_compile_error(),
    };

    quote! {
        impl lore_error_set::FfiError for #name {
            fn ffi_code(&self) -> i32 { #code }
        }
    }
}

fn extract_ffi_code(input: &DeriveInput) -> syn::Result<syn::LitInt> {
    for attr in &input.attrs {
        if attr.path().is_ident("ffi_code") {
            let lit: syn::LitInt = attr.parse_args()?;
            // Validate it parses as i32.
            lit.base10_parse::<i32>().map_err(|_err| {
                syn::Error::new_spanned(&lit, "ffi_code must be an integer literal")
            })?;
            return Ok(lit);
        }
    }

    Err(syn::Error::new_spanned(
        &input.ident,
        "#[derive(FfiError)] requires #[ffi_code(N)] attribute",
    ))
}
