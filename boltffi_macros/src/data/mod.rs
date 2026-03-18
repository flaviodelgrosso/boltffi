mod passable;
mod record_impl;

use proc_macro::TokenStream;
use quote::{format_ident, quote};

use crate::custom_types;

pub use record_impl::data_impl_block;

pub fn data_impl(item: TokenStream) -> TokenStream {
    let item_clone = item.clone();

    if let Ok(mut item_struct) = syn::parse::<syn::ItemStruct>(item_clone.clone()) {
        let has_repr = item_struct.attrs.iter().any(|a| a.path().is_ident("repr"));
        if !has_repr {
            item_struct.attrs.insert(0, syn::parse_quote!(#[repr(C)]));
        }

        strip_boltffi_field_attrs(&mut item_struct.fields);

        let struct_name = &item_struct.ident;
        let free_fn_name = format_ident!("boltffi_free_buf_{}", struct_name);

        let custom_types = match custom_types::registry_for_current_crate() {
            Ok(registry) => registry,
            Err(error) => return error.to_compile_error().into(),
        };

        let wire_impls = passable::generate_struct_wire_impls(&item_struct, &custom_types);
        let passable_impl = passable::classify_and_generate_struct_passable(&item_struct);

        return TokenStream::from(quote! {
            #item_struct
            #wire_impls
            #passable_impl

            #[cfg(not(test))]
            #[unsafe(no_mangle)]
            pub extern "C" fn #free_fn_name(buf: ::boltffi::__private::FfiBuf) {
                drop(buf);
            }
        });
    }

    if let Ok(mut item_enum) = syn::parse::<syn::ItemEnum>(item_clone) {
        let has_repr = item_enum.attrs.iter().any(|a| a.path().is_ident("repr"));
        if !has_repr {
            let has_data = item_enum.variants.iter().any(|v| !v.fields.is_empty());
            if has_data {
                item_enum
                    .attrs
                    .insert(0, syn::parse_quote!(#[repr(C, i32)]));
            } else {
                item_enum.attrs.insert(0, syn::parse_quote!(#[repr(i32)]));
            }
        }

        let custom_types = match custom_types::registry_for_current_crate() {
            Ok(registry) => registry,
            Err(error) => return error.to_compile_error().into(),
        };

        let wire_impls = passable::generate_enum_wire_impls(&item_enum, &custom_types);
        let passable_impl = passable::classify_and_generate_enum_passable(&item_enum);

        return TokenStream::from(quote! {
            #item_enum
            #wire_impls
            #passable_impl
        });
    }

    syn::Error::new_spanned(
        proc_macro2::TokenStream::from(item),
        "data can only be applied to struct or enum",
    )
    .to_compile_error()
    .into()
}

fn strip_boltffi_field_attrs(fields: &mut syn::Fields) {
    fields.iter_mut().for_each(|field| {
        field.attrs.retain(|attr| !is_boltffi_field_attr(attr));
    });
}

fn is_boltffi_field_attr(attr: &syn::Attribute) -> bool {
    let path = attr.path();
    path.segments.len() == 2
        && path.segments[0].ident == "boltffi"
        && path.segments[1].ident == "default"
}

pub fn derive_data_impl(input: TokenStream) -> TokenStream {
    let derive_input = match syn::parse::<syn::DeriveInput>(input) {
        Ok(derive_input) => derive_input,
        Err(error) => return error.to_compile_error().into(),
    };

    syn::Error::new_spanned(
        derive_input.ident,
        "#[derive(Data)] is not supported; use #[data] or #[error] instead",
    )
    .to_compile_error()
    .into()
}
