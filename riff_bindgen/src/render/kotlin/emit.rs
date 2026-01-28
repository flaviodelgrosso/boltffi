use crate::ir::codec::{EnumLayout, VecLayout};
use crate::ir::ids::BuiltinId;
use crate::ir::ops::{OffsetExpr, ReadOp, ReadSeq, SizeExpr, ValueExpr, WriteOp, WriteSeq};
use crate::ir::types::{PrimitiveType, TypeExpr};
use crate::kotlin::NamingConvention;

pub fn render_value(expr: &ValueExpr) -> String {
    match expr {
        ValueExpr::Instance => String::new(),
        ValueExpr::Var(name) => name.clone(),
        ValueExpr::Named(name) => NamingConvention::property_name(name),
        ValueExpr::Field(parent, field) => {
            let parent_str = render_value(parent);
            let field_str = NamingConvention::property_name(field.as_str());
            if parent_str.is_empty() {
                field_str
            } else {
                format!("{}.{}", parent_str, field_str)
            }
        }
    }
}

pub fn emit_size_expr(size: &SizeExpr) -> String {
    match size {
        SizeExpr::Fixed(value) => value.to_string(),
        SizeExpr::Runtime => "0".to_string(),
        SizeExpr::StringLen(value) => format!("Utf8Codec.maxBytes({})", render_value(value)),
        SizeExpr::BytesLen(value) => format!("{}.size", render_value(value)),
        SizeExpr::ValueSize(value) => render_value(value),
        SizeExpr::WireSize { value } => format!("{}.wireEncodedSize()", render_value(value)),
        SizeExpr::BuiltinSize { id, value } => emit_builtin_size(id, &render_value(value)),
        SizeExpr::Sum(parts) => {
            let rendered = parts
                .iter()
                .map(emit_size_expr)
                .collect::<Vec<_>>()
                .join(" + ");
            format!("({})", rendered)
        }
        SizeExpr::OptionSize { value, inner } => {
            let inner_expr = emit_size_expr(inner);
            format!(
                "({}?.let {{ v -> 1 + {} }} ?: 1)",
                render_value(value),
                inner_expr
            )
        }
        SizeExpr::VecSize {
            value,
            inner,
            layout,
        } => emit_vec_size(&render_value(value), inner, layout),
        SizeExpr::ResultSize { value, ok, err } => {
            let v = render_value(value);
            let ok_expr = emit_size_expr(ok);
            let err_expr = emit_size_expr(err);
            format!(
                "(when (val _r = {}) {{ is RiffResult.Ok -> {{ val okVal = _r.value; 1 + {} }}; is RiffResult.Err -> {{ val errVal = _r.error; 1 + {} }} }})",
                v, ok_expr, err_expr
            )
        }
    }
}

pub fn emit_read_pair(seq: &ReadSeq, base_name: &str, base_expr: &str) -> String {
    let op = seq.ops.first().expect("read ops");
    match op {
        ReadOp::Primitive { primitive, offset } => {
            let value_expr = emit_read_primitive(*primitive, offset, base_name, base_expr);
            let size = emit_size_expr(&seq.size);
            format!("{} to {}", value_expr, size)
        }
        ReadOp::String { offset } => emit_read_string(offset, base_name, base_expr),
        ReadOp::Bytes { offset } => emit_read_bytes(offset, base_name, base_expr),
        ReadOp::Option { tag_offset, some } => {
            let offset_expr = emit_offset_expr(tag_offset, base_name, base_expr);
            let inner = emit_read_pair(some, "it", "it");
            format!("wire.readNullable({}, {{ {} }})", offset_expr, inner)
        }
        ReadOp::Vec {
            len_offset,
            element_type,
            element,
            layout,
        } => emit_read_vec_pair(
            len_offset,
            element_type,
            element,
            layout,
            base_name,
            base_expr,
        ),
        ReadOp::Record { id, offset, .. } => {
            let offset_expr = emit_offset_expr(offset, base_name, base_expr);
            format!("{}.decode(wire, {})", id.as_str(), offset_expr)
        }
        ReadOp::Enum { id, offset, layout } => {
            let offset_expr = emit_offset_expr(offset, base_name, base_expr);
            match layout {
                EnumLayout::CStyle { .. } => {
                    let size = emit_size_expr(&seq.size);
                    format!(
                        "{}.fromValue(wire.readI32({})) to {}",
                        id.as_str(),
                        offset_expr,
                        size
                    )
                }
                EnumLayout::Data { .. } | EnumLayout::Recursive => {
                    format!("{}.decode(wire, {})", id.as_str(), offset_expr)
                }
            }
        }
        ReadOp::Result {
            tag_offset,
            ok,
            err,
        } => {
            let offset_expr = emit_offset_expr(tag_offset, base_name, base_expr);
            let ok_expr = emit_read_pair(ok, "it", "it");
            let err_expr = emit_read_pair(err, "it", "it");
            format!(
                "wire.readResult({}, {{ {} }}, {{ {} }})",
                offset_expr, ok_expr, err_expr
            )
        }
        ReadOp::Builtin { id, offset } => {
            let value_expr = emit_read_builtin(id, offset, base_name, base_expr);
            let size = emit_size_expr(&seq.size);
            format!("{} to {}", value_expr, size)
        }
        ReadOp::Custom { id, .. } => {
            format!(
                "{}.decode(wire, {}).let {{ v -> v.first to v.second }}",
                id.as_str(),
                base_expr
            )
        }
    }
}

pub fn emit_read_value(seq: &ReadSeq, base_name: &str, base_expr: &str) -> String {
    let op = seq.ops.first().expect("read ops");
    match op {
        ReadOp::Primitive { primitive, offset } => {
            emit_read_primitive(*primitive, offset, base_name, base_expr)
        }
        ReadOp::String { offset } => {
            format!("{}.first", emit_read_string(offset, base_name, base_expr))
        }
        ReadOp::Bytes { offset } => {
            format!("{}.first", emit_read_bytes(offset, base_name, base_expr))
        }
        ReadOp::Option { tag_offset, some } => {
            let offset_expr = emit_offset_expr(tag_offset, base_name, base_expr);
            let inner = emit_read_pair(some, "it", "it");
            format!("wire.readNullable({}, {{ {} }}).first", offset_expr, inner)
        }
        ReadOp::Vec {
            len_offset,
            element_type,
            element,
            layout,
        } => {
            let pair = emit_read_vec_pair(
                len_offset,
                element_type,
                element,
                layout,
                base_name,
                base_expr,
            );
            format!("{}.first", pair)
        }
        ReadOp::Record { id, offset, .. } => {
            let offset_expr = emit_offset_expr(offset, base_name, base_expr);
            format!("{}.decode(wire, {}).first", id.as_str(), offset_expr)
        }
        ReadOp::Enum { id, offset, layout } => {
            let offset_expr = emit_offset_expr(offset, base_name, base_expr);
            match layout {
                EnumLayout::CStyle { .. } => {
                    format!("{}.fromValue(wire.readI32({}))", id.as_str(), offset_expr)
                }
                EnumLayout::Data { .. } | EnumLayout::Recursive => {
                    format!("{}.decode(wire, {}).first", id.as_str(), offset_expr)
                }
            }
        }
        ReadOp::Result {
            tag_offset,
            ok,
            err,
        } => {
            let offset_expr = emit_offset_expr(tag_offset, base_name, base_expr);
            let ok_expr = emit_read_pair(ok, "it", "it");
            let err_expr = emit_read_pair(err, "it", "it");
            format!(
                "wire.readResult({}, {{ {} }}, {{ {} }}).first",
                offset_expr, ok_expr, err_expr
            )
        }
        ReadOp::Builtin { id, offset } => emit_read_builtin(id, offset, base_name, base_expr),
        ReadOp::Custom { id, .. } => {
            format!("{}.decode(wire, {}).first", id.as_str(), base_expr)
        }
    }
}

pub fn emit_write_expr(seq: &WriteSeq) -> String {
    let op = seq.ops.first().expect("write ops");
    match op {
        WriteOp::Primitive { primitive, value } => {
            emit_write_primitive(*primitive, &render_value(value))
        }
        WriteOp::String { value } => format!("wire.writeString({})", render_value(value)),
        WriteOp::Bytes { value } => format!("wire.writeBytes({})", render_value(value)),
        WriteOp::Option { value, some } => {
            let inner = emit_write_expr(some);
            format!(
                "{}?.let {{ v -> wire.writeU8(1u); {} }} ?: wire.writeU8(0u)",
                render_value(value),
                inner
            )
        }
        WriteOp::Vec {
            value,
            element_type,
            element,
            layout,
        } => emit_write_vec(&render_value(value), element_type, element, layout),
        WriteOp::Record { value, .. } => {
            format!("{}.wireEncodeTo(wire)", render_value(value))
        }
        WriteOp::Enum { value, layout, .. } => match layout {
            EnumLayout::CStyle { .. } => format!("wire.writeI32({}.value)", render_value(value)),
            EnumLayout::Data { .. } | EnumLayout::Recursive => {
                format!("{}.wireEncodeTo(wire)", render_value(value))
            }
        },
        WriteOp::Result { value, ok, err } => {
            let v = render_value(value);
            let ok_expr = emit_write_expr(ok);
            let err_expr = emit_write_expr(err);
            format!(
                "when ({}) {{ is RiffResult.Ok -> {{ wire.writeU8(0u); val okVal = {}.value; {} }} is RiffResult.Err -> {{ wire.writeU8(1u); val errVal = {}.error; {} }} }}",
                v, v, ok_expr, v, err_expr
            )
        }
        WriteOp::Builtin { id, value } => emit_write_builtin(id, &render_value(value)),
        WriteOp::Custom { value, .. } => {
            format!("{}.wireEncodeTo(wire)", render_value(value))
        }
    }
}

pub fn emit_inline_decode(seq: &ReadSeq, _local_name: &str, pos_var: &str) -> String {
    match seq.size {
        SizeExpr::Fixed(fixed) => {
            let value_expr = emit_read_value(seq, pos_var, pos_var);
            format!(
                "run {{ val v = {}; {} += {}; v }}",
                value_expr, pos_var, fixed
            )
        }
        _ => {
            let pair_expr = emit_read_pair(seq, pos_var, pos_var);
            format!("run {{ val (v, s) = {}; {} += s; v }}", pair_expr, pos_var)
        }
    }
}

fn emit_vec_size(value: &str, inner: &SizeExpr, layout: &VecLayout) -> String {
    match layout {
        VecLayout::Blittable { .. } => {
            format!("(4 + {}.size * {})", value, emit_size_expr(inner))
        }
        VecLayout::Encoded => {
            format!(
                "(4 + {}.sumOf {{ item -> {} }})",
                value,
                emit_size_expr(inner)
            )
        }
    }
}

fn emit_builtin_size(id: &BuiltinId, value: &str) -> String {
    if id.as_str() == "Url" {
        format!("Utf8Codec.maxBytes({}.toString())", value)
    } else {
        format!("{}.wireEncodedSize()", value)
    }
}

fn emit_offset_expr(offset: &OffsetExpr, base_name: &str, base_expr: &str) -> String {
    match offset {
        OffsetExpr::Fixed(value) => value.to_string(),
        OffsetExpr::Base => base_expr.to_string(),
        OffsetExpr::BasePlus(add) => {
            if *add == 0 {
                base_expr.to_string()
            } else {
                format!("{} + {}", base_expr, add)
            }
        }
        OffsetExpr::Var(name) => {
            if name == base_name {
                base_expr.to_string()
            } else {
                name.to_string()
            }
        }
        OffsetExpr::VarPlus(name, add) => {
            let base = if name == base_name {
                base_expr.to_string()
            } else {
                name.to_string()
            };
            if *add == 0 {
                base
            } else {
                format!("{} + {}", base, add)
            }
        }
    }
}

fn emit_read_primitive(
    primitive: PrimitiveType,
    offset: &OffsetExpr,
    base_name: &str,
    base_expr: &str,
) -> String {
    let offset_expr = emit_offset_expr(offset, base_name, base_expr);
    match primitive {
        PrimitiveType::Bool => format!("wire.readBool({})", offset_expr),
        PrimitiveType::I8 => format!("wire.readI8({})", offset_expr),
        PrimitiveType::U8 => format!("wire.readU8({})", offset_expr),
        PrimitiveType::I16 => format!("wire.readI16({})", offset_expr),
        PrimitiveType::U16 => format!("wire.readU16({})", offset_expr),
        PrimitiveType::I32 => format!("wire.readI32({})", offset_expr),
        PrimitiveType::U32 => format!("wire.readU32({})", offset_expr),
        PrimitiveType::I64 | PrimitiveType::ISize => format!("wire.readI64({})", offset_expr),
        PrimitiveType::U64 | PrimitiveType::USize => format!("wire.readU64({})", offset_expr),
        PrimitiveType::F32 => format!("wire.readF32({})", offset_expr),
        PrimitiveType::F64 => format!("wire.readF64({})", offset_expr),
    }
}

fn emit_read_string(offset: &OffsetExpr, base_name: &str, base_expr: &str) -> String {
    let offset_expr = emit_offset_expr(offset, base_name, base_expr);
    format!("wire.readString({})", offset_expr)
}

fn emit_read_bytes(offset: &OffsetExpr, base_name: &str, base_expr: &str) -> String {
    let offset_expr = emit_offset_expr(offset, base_name, base_expr);
    format!("wire.readByteArray({})", offset_expr)
}

fn emit_read_builtin(
    id: &BuiltinId,
    offset: &OffsetExpr,
    base_name: &str,
    base_expr: &str,
) -> String {
    let offset_expr = emit_offset_expr(offset, base_name, base_expr);
    match id.as_str() {
        "Duration" => format!("wire.readDuration({})", offset_expr),
        "SystemTime" => format!("wire.readInstant({})", offset_expr),
        "Uuid" => format!("wire.readUuid({})", offset_expr),
        "Url" => format!("wire.readUri({})", offset_expr),
        _ => format!("wire.readString({})", offset_expr),
    }
}

fn emit_read_vec_pair(
    len_offset: &OffsetExpr,
    element_type: &TypeExpr,
    element: &ReadSeq,
    layout: &VecLayout,
    base_name: &str,
    base_expr: &str,
) -> String {
    let offset_expr = emit_offset_expr(len_offset, base_name, base_expr);
    match layout {
        VecLayout::Blittable { .. } => match element_type {
            TypeExpr::Primitive(primitive) => match primitive {
                PrimitiveType::I32 | PrimitiveType::U32 => {
                    format!("wire.readIntArray({})", offset_expr)
                }
                PrimitiveType::I16 | PrimitiveType::U16 => {
                    format!("wire.readShortArray({})", offset_expr)
                }
                PrimitiveType::I64
                | PrimitiveType::U64
                | PrimitiveType::ISize
                | PrimitiveType::USize => {
                    format!("wire.readLongArray({})", offset_expr)
                }
                PrimitiveType::F32 => format!("wire.readFloatArray({})", offset_expr),
                PrimitiveType::F64 => format!("wire.readDoubleArray({})", offset_expr),
                PrimitiveType::U8 | PrimitiveType::I8 => {
                    format!("wire.readByteArray({})", offset_expr)
                }
                PrimitiveType::Bool => format!("wire.readBooleanArray({})", offset_expr),
            },
            _ => {
                let inner = emit_read_pair(element, "it", "it");
                format!("wire.readList({}, {{ {} }})", offset_expr, inner)
            }
        },
        VecLayout::Encoded => {
            let inner = emit_read_pair(element, "it", "it");
            format!("wire.readList({}, {{ {} }})", offset_expr, inner)
        }
    }
}

fn emit_write_primitive(primitive: PrimitiveType, value: &str) -> String {
    match primitive {
        PrimitiveType::Bool => format!("wire.writeBool({})", value),
        PrimitiveType::I8 => format!("wire.writeI8({})", value),
        PrimitiveType::U8 => format!("wire.writeU8({})", value),
        PrimitiveType::I16 => format!("wire.writeI16({})", value),
        PrimitiveType::U16 => format!("wire.writeU16({})", value),
        PrimitiveType::I32 => format!("wire.writeI32({})", value),
        PrimitiveType::U32 => format!("wire.writeU32({})", value),
        PrimitiveType::I64 | PrimitiveType::ISize => format!("wire.writeI64({})", value),
        PrimitiveType::U64 | PrimitiveType::USize => format!("wire.writeU64({})", value),
        PrimitiveType::F32 => format!("wire.writeF32({})", value),
        PrimitiveType::F64 => format!("wire.writeF64({})", value),
    }
}

fn emit_write_vec(
    value: &str,
    element_type: &TypeExpr,
    element: &WriteSeq,
    layout: &VecLayout,
) -> String {
    match layout {
        VecLayout::Blittable { .. } => match element_type {
            TypeExpr::Primitive(_) => format!("wire.writePrimitiveList({})", value),
            TypeExpr::Record(id) => format!(
                "wire.writeU32({}.size.toUInt()); {}Writer.writeAllToWire(wire, {})",
                value,
                id.as_str(),
                value
            ),
            _ => {
                let inner = emit_write_expr(element);
                format!(
                    "wire.writeU32({}.size.toUInt()); {}.forEach {{ item -> {} }}",
                    value, value, inner
                )
            }
        },
        VecLayout::Encoded => {
            let inner = emit_write_expr(element);
            format!(
                "wire.writeU32({}.size.toUInt()); {}.forEach {{ item -> {} }}",
                value, value, inner
            )
        }
    }
}

fn emit_write_builtin(id: &BuiltinId, value: &str) -> String {
    match id.as_str() {
        "Duration" => format!("wire.writeDuration({})", value),
        "SystemTime" => format!("wire.writeInstant({})", value),
        "Uuid" => format!("wire.writeUuid({})", value),
        "Url" => format!("wire.writeUri({})", value),
        _ => format!("wire.writeString({})", value),
    }
}
