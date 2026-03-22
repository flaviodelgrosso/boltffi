use boltffi_ffi_rules::primitive::Primitive;
use syn::{PathArguments, Type};

pub(crate) trait TypeShapeExt {
    fn is_primitive_type(&self) -> bool;
    fn is_string_like_type(&self) -> bool;
    fn is_named_nominal_type(&self) -> bool;
    fn is_generic_nominal_type(&self) -> bool;
}

impl TypeShapeExt for Type {
    fn is_primitive_type(&self) -> bool {
        match self {
            Type::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|ident| ident.to_string().parse::<Primitive>().is_ok()),
            Type::Group(group) => group.elem.as_ref().is_primitive_type(),
            Type::Paren(paren) => paren.elem.as_ref().is_primitive_type(),
            _ => false,
        }
    }

    fn is_string_like_type(&self) -> bool {
        match self {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "String"),
            Type::Reference(reference) => match reference.elem.as_ref() {
                Type::Path(path) => path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "str"),
                _ => false,
            },
            Type::Group(group) => group.elem.as_ref().is_string_like_type(),
            Type::Paren(paren) => paren.elem.as_ref().is_string_like_type(),
            _ => false,
        }
    }

    fn is_named_nominal_type(&self) -> bool {
        match self {
            Type::Path(type_path) if type_path.qself.is_none() => {
                let Some(segment) = type_path.path.segments.last() else {
                    return false;
                };
                if !matches!(segment.arguments, PathArguments::None) {
                    return false;
                }
                let type_name = segment.ident.to_string();
                (type_name != "()")
                    && type_name.parse::<Primitive>().is_err()
                    && type_name
                        .chars()
                        .next()
                        .is_some_and(|character| character.is_uppercase())
            }
            Type::Group(group) => group.elem.as_ref().is_named_nominal_type(),
            Type::Paren(paren) => paren.elem.as_ref().is_named_nominal_type(),
            _ => false,
        }
    }

    fn is_generic_nominal_type(&self) -> bool {
        match self {
            Type::Path(type_path) if type_path.qself.is_none() => {
                type_path.path.segments.last().is_some_and(|segment| {
                    matches!(segment.arguments, PathArguments::AngleBracketed(_))
                })
            }
            Type::Group(group) => group.elem.as_ref().is_generic_nominal_type(),
            Type::Paren(paren) => paren.elem.as_ref().is_generic_nominal_type(),
            _ => false,
        }
    }
}
