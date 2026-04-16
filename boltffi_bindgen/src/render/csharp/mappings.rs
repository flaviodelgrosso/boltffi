use crate::ir::types::PrimitiveType;

/// Maps a BoltFFI primitive to the corresponding C# type keyword.
///
/// C# has native unsigned types (`byte`, `ushort`, `uint`, `ulong`) and
/// platform-sized integers (`nint`, `nuint`), so every primitive maps
/// to a distinct C# type with no information loss.
pub fn csharp_type(primitive: PrimitiveType) -> &'static str {
    match primitive {
        PrimitiveType::Bool => "bool",
        PrimitiveType::I8 => "sbyte",
        PrimitiveType::U8 => "byte",
        PrimitiveType::I16 => "short",
        PrimitiveType::U16 => "ushort",
        PrimitiveType::I32 => "int",
        PrimitiveType::U32 => "uint",
        PrimitiveType::I64 => "long",
        PrimitiveType::U64 => "ulong",
        PrimitiveType::ISize => "nint",
        PrimitiveType::USize => "nuint",
        PrimitiveType::F32 => "float",
        PrimitiveType::F64 => "double",
    }
}
