use crate::model::{Primitive, Type};

use super::NamingConvention;

pub struct TypeMapper;

impl TypeMapper {
    pub fn map_type(ty: &Type) -> String {
        match ty {
            Type::Primitive(primitive) => Self::map_primitive(primitive),
            Type::String => "String".into(),
            Type::Bytes => "ByteArray".into(),
            Type::Slice(inner) => format!("List<{}>", Self::map_type(inner)),
            Type::MutSlice(inner) => format!("MutableList<{}>", Self::map_type(inner)),
            Type::Vec(inner) => format!("List<{}>", Self::map_type(inner)),
            Type::Option(inner) => format!("{}?", Self::map_type(inner)),
            Type::Result { ok, .. } => Self::map_type(ok),
            Type::Callback(inner) => format!("({}) -> Unit", Self::map_type(inner)),
            Type::Object(name) => NamingConvention::class_name(name),
            Type::Record(name) => NamingConvention::class_name(name),
            Type::Enum(name) => NamingConvention::class_name(name),
            Type::BoxedTrait(name) => NamingConvention::class_name(name),
            Type::Void => "Unit".into(),
        }
    }

    fn map_primitive(primitive: &Primitive) -> String {
        match primitive {
            Primitive::I8 => "Byte",
            Primitive::I16 => "Short",
            Primitive::I32 => "Int",
            Primitive::I64 => "Long",
            Primitive::U8 => "UByte",
            Primitive::U16 => "UShort",
            Primitive::U32 => "UInt",
            Primitive::U64 => "ULong",
            Primitive::F32 => "Float",
            Primitive::F64 => "Double",
            Primitive::Bool => "Boolean",
            Primitive::Usize => "Long",
            Primitive::Isize => "Long",
        }
        .into()
    }

    pub fn jni_type(ty: &Type) -> String {
        match ty {
            Type::Primitive(primitive) => Self::jni_primitive(primitive),
            Type::String => "String".into(),
            Type::Bytes => "ByteArray".into(),
            Type::Object(_) | Type::BoxedTrait(_) => "Long".into(),
            Type::Record(name) => NamingConvention::class_name(name),
            Type::Enum(_) => "Int".into(),
            Type::Vec(_) | Type::Slice(_) | Type::MutSlice(_) => "Long".into(),
            Type::Option(inner) => format!("{}?", Self::jni_type(inner)),
            Type::Result { ok, .. } => Self::jni_type(ok),
            Type::Callback(_) => "Long".into(),
            Type::Void => "Unit".into(),
        }
    }

    fn jni_primitive(primitive: &Primitive) -> String {
        match primitive {
            Primitive::I8 => "Byte",
            Primitive::I16 => "Short",
            Primitive::I32 => "Int",
            Primitive::I64 => "Long",
            Primitive::U8 => "Byte",
            Primitive::U16 => "Short",
            Primitive::U32 => "Int",
            Primitive::U64 => "Long",
            Primitive::F32 => "Float",
            Primitive::F64 => "Double",
            Primitive::Bool => "Boolean",
            Primitive::Usize => "Long",
            Primitive::Isize => "Long",
        }
        .into()
    }

    pub fn default_value(ty: &Type) -> String {
        match ty {
            Type::Primitive(primitive) => Self::primitive_default(primitive),
            Type::String => "\"\"".into(),
            Type::Bytes => "byteArrayOf()".into(),
            Type::Vec(_) | Type::Slice(_) | Type::MutSlice(_) => "emptyList()".into(),
            Type::Option(_) => "null".into(),
            Type::Void => "Unit".into(),
            _ => "TODO()".into(),
        }
    }

    fn primitive_default(primitive: &Primitive) -> String {
        match primitive {
            Primitive::Bool => "false",
            Primitive::F32 => "0.0f",
            Primitive::F64 => "0.0",
            _ => "0",
        }
        .into()
    }
}
