use boltffi_ffi_rules::primitive::Primitive;
use quote::quote;
use syn::Type;

use crate::lowering::transport::NamedTypeTransport;
use crate::registries::data_types::DataTypeCategory;

use super::model::{
    EncodedReturnStrategy, ReturnLoweringContext, ScalarReturnStrategy, ValueReturnStrategy,
};

pub fn extract_vec_inner(ty: &Type) -> Option<syn::Type> {
    if let Type::Path(path) = ty
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty.clone());
    }
    None
}

pub fn extract_option_inner(ty: &Type) -> Option<syn::Type> {
    if let Type::Path(path) = ty
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty.clone());
    }
    None
}

pub fn is_primitive_type(type_name: &str) -> bool {
    type_name == "()" || type_name.parse::<Primitive>().is_ok()
}

pub fn type_is_primitive(ty: &Type) -> bool {
    let type_name = quote!(#ty).to_string().replace(' ', "");
    is_primitive_type(&type_name)
}

pub fn primitive_for_type(ty: &Type) -> Option<Primitive> {
    quote!(#ty).to_string().replace(' ', "").parse().ok()
}

pub fn classify_value_return_strategy(
    rust_type: &Type,
    return_lowering: &ReturnLoweringContext<'_>,
) -> ValueReturnStrategy {
    let type_name = quote!(#rust_type).to_string().replace(' ', "");

    if type_name == "()" {
        return ValueReturnStrategy::Void;
    }

    if type_name == "String" || type_name == "std::string::String" {
        return ValueReturnStrategy::Buffer(EncodedReturnStrategy::Utf8String);
    }

    if let Some(inner) = extract_vec_inner(rust_type) {
        let buffer_strategy = if return_lowering
            .named_type_transport_classifier()
            .supports_direct_vec_transport(&inner)
        {
            EncodedReturnStrategy::DirectVec
        } else {
            EncodedReturnStrategy::WireEncoded
        };
        return ValueReturnStrategy::Buffer(buffer_strategy);
    }

    if let Type::Path(path) = rust_type
        && let Some(segment) = path.path.segments.last()
    {
        if segment.ident == "Result"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && args.args.len() >= 2
            && let Some(syn::GenericArgument::Type(ok_ty)) = args.args.first()
            && let Some(syn::GenericArgument::Type(err_ty)) = args.args.iter().nth(1)
        {
            return if type_is_primitive(ok_ty) && type_is_primitive(err_ty) {
                ValueReturnStrategy::Buffer(EncodedReturnStrategy::ResultScalar)
            } else {
                ValueReturnStrategy::Buffer(EncodedReturnStrategy::WireEncoded)
            };
        }

        if segment.ident == "Option"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        {
            return if type_is_primitive(inner_ty) {
                ValueReturnStrategy::Buffer(EncodedReturnStrategy::OptionScalar)
            } else {
                ValueReturnStrategy::Buffer(EncodedReturnStrategy::WireEncoded)
            };
        }
    }

    if is_primitive_type(&type_name) {
        return ValueReturnStrategy::Scalar(ScalarReturnStrategy::PrimitiveValue);
    }

    match return_lowering
        .named_type_transport_classifier()
        .classify_named_type_transport(rust_type)
    {
        NamedTypeTransport::WireEncoded => {
            ValueReturnStrategy::Buffer(EncodedReturnStrategy::WireEncoded)
        }
        NamedTypeTransport::Passable => {
            match return_lowering.data_types().category_for(rust_type) {
                Some(DataTypeCategory::Scalar) => {
                    ValueReturnStrategy::Scalar(ScalarReturnStrategy::CStyleEnumTag)
                }
                Some(DataTypeCategory::Blittable) => ValueReturnStrategy::CompositeValue,
                Some(DataTypeCategory::WireEncoded) | None => {
                    unreachable!("passable return transport requires scalar or blittable data type")
                }
            }
        }
    }
}
