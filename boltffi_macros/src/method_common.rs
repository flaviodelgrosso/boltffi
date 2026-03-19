use quote::quote;
use syn::{FnArg, ReturnType, Type};

use crate::returns::ReturnAbi;

pub fn sync_error_return_expr(return_abi: &ReturnAbi) -> proc_macro2::TokenStream {
    match return_abi {
        ReturnAbi::Unit => quote! { ::boltffi::__private::FfiStatus::INVALID_ARG },
        ReturnAbi::Scalar { .. } => quote! { ::core::default::Default::default() },
        ReturnAbi::Encoded { .. } => quote! { ::boltffi::__private::FfiBuf::default() },
        ReturnAbi::Passable { rust_type } => quote! {
            unsafe {
                ::core::mem::MaybeUninit::<<#rust_type as ::boltffi::__private::Passable>::Out>::zeroed().assume_init()
            }
        },
    }
}

pub fn is_factory_constructor(method: &syn::ImplItemFn, type_name: &syn::Ident) -> bool {
    let has_self = method
        .sig
        .inputs
        .first()
        .is_some_and(|arg| matches!(arg, FnArg::Receiver(_)));

    if has_self {
        return false;
    }

    is_factory_return(&method.sig.output, type_name)
}

pub fn is_factory_return(output: &ReturnType, type_name: &syn::Ident) -> bool {
    match output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(type_path) => {
                is_self_type_path(&type_path.path, type_name)
                    || is_result_of_self_type_path(&type_path.path, type_name)
            }
            _ => false,
        },
    }
}

pub fn is_self_type_path(path: &syn::Path, type_name: &syn::Ident) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "Self" || segment.ident == *type_name)
}

pub fn is_result_of_self_type_path(path: &syn::Path, type_name: &syn::Ident) -> bool {
    let Some(result_segment) = path.segments.last() else {
        return false;
    };
    if result_segment.ident != "Result" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &result_segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(Type::Path(ok_type_path))) = args.args.first() else {
        return false;
    };
    is_self_type_path(&ok_type_path.path, type_name)
}

pub fn exported_methods(item_impl: &syn::ItemImpl) -> impl Iterator<Item = &syn::ImplItemFn> + '_ {
    item_impl
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method) => Some(method),
            _ => None,
        })
        .filter(|method| matches!(method.vis, syn::Visibility::Public(_)))
        .filter(|method| !method.attrs.iter().any(|a| a.path().is_ident("skip")))
}

pub fn impl_type_name(item_impl: &syn::ItemImpl) -> Option<syn::Ident> {
    match item_impl.self_ty.as_ref() {
        Type::Path(path) => path.path.segments.last().map(|s| s.ident.clone()),
        _ => None,
    }
}
