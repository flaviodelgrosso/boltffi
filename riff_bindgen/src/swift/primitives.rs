use crate::model::Primitive;

#[derive(Debug, Clone, Copy)]
pub struct SwiftPrimitiveInfo {
    pub swift_type: &'static str,
    pub default_value: &'static str,
}

pub const fn info(p: Primitive) -> SwiftPrimitiveInfo {
    match p {
        Primitive::Bool => SwiftPrimitiveInfo {
            swift_type: "Bool",
            default_value: "false",
        },
        Primitive::I8 => SwiftPrimitiveInfo {
            swift_type: "Int8",
            default_value: "0",
        },
        Primitive::U8 => SwiftPrimitiveInfo {
            swift_type: "UInt8",
            default_value: "0",
        },
        Primitive::I16 => SwiftPrimitiveInfo {
            swift_type: "Int16",
            default_value: "0",
        },
        Primitive::U16 => SwiftPrimitiveInfo {
            swift_type: "UInt16",
            default_value: "0",
        },
        Primitive::I32 => SwiftPrimitiveInfo {
            swift_type: "Int32",
            default_value: "0",
        },
        Primitive::U32 => SwiftPrimitiveInfo {
            swift_type: "UInt32",
            default_value: "0",
        },
        Primitive::I64 => SwiftPrimitiveInfo {
            swift_type: "Int64",
            default_value: "0",
        },
        Primitive::U64 => SwiftPrimitiveInfo {
            swift_type: "UInt64",
            default_value: "0",
        },
        Primitive::Isize => SwiftPrimitiveInfo {
            swift_type: "Int",
            default_value: "0",
        },
        Primitive::Usize => SwiftPrimitiveInfo {
            swift_type: "UInt",
            default_value: "0",
        },
        Primitive::F32 => SwiftPrimitiveInfo {
            swift_type: "Float",
            default_value: "0.0",
        },
        Primitive::F64 => SwiftPrimitiveInfo {
            swift_type: "Double",
            default_value: "0.0",
        },
    }
}
