use crate::model::{Primitive, Type};

pub struct ByteBufferHelpers;

impl ByteBufferHelpers {
    pub fn getter(ty: &Type) -> &'static str {
        match ty {
            Type::Primitive(Primitive::I8) | Type::Primitive(Primitive::U8) => "get",
            Type::Primitive(Primitive::I16) | Type::Primitive(Primitive::U16) => "getShort",
            Type::Primitive(Primitive::I32) | Type::Primitive(Primitive::U32) => "getInt",
            Type::Primitive(Primitive::I64) | Type::Primitive(Primitive::U64) 
                | Type::Primitive(Primitive::Usize) | Type::Primitive(Primitive::Isize) => "getLong",
            Type::Primitive(Primitive::F32) => "getFloat",
            Type::Primitive(Primitive::F64) => "getDouble",
            Type::Primitive(Primitive::Bool) => "get",
            _ => "getLong",
        }
    }

    pub fn conversion(ty: &Type) -> &'static str {
        match ty {
            Type::Primitive(Primitive::Bool) => " != 0.toByte()",
            _ => "",
        }
    }
}
