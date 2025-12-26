use crate::model::Type;

#[derive(Debug, Clone)]
pub enum ReturnKind {
    Void,
    Primitive,
    String,
    Vec { inner: String, len_fn: String, copy_fn: String },
    VecRecord { inner: String, reader: String },
    Option { inner: String },
    Result { ok: String },
    Enum { name: String },
    Record { name: String },
}

impl ReturnKind {
    pub fn from_type(ty: &Type, ffi_base: &str) -> Self {
        match ty {
            Type::Void => Self::Void,
            Type::Primitive(_) => Self::Primitive,
            Type::String => Self::String,
            Type::Vec(inner) => match inner.as_ref() {
                Type::Record(name) => Self::VecRecord {
                    inner: super::NamingConvention::class_name(name),
                    reader: format!("{}Reader", super::NamingConvention::class_name(name)),
                },
                _ => Self::Vec {
                    inner: super::TypeMapper::map_type(inner),
                    len_fn: format!("{}_len", ffi_base),
                    copy_fn: format!("{}_copy_into", ffi_base),
                },
            },
            Type::Option(inner) => Self::Option {
                inner: super::TypeMapper::map_type(inner),
            },
            Type::Result { ok, .. } => Self::Result {
                ok: super::TypeMapper::map_type(ok),
            },
            Type::Enum(name) => Self::Enum {
                name: super::NamingConvention::class_name(name),
            },
            Type::Record(name) => Self::Record {
                name: super::NamingConvention::class_name(name),
            },
            Type::Bytes => panic!("Bytes return type not yet supported in Kotlin bindings"),
            Type::Slice(_) => panic!("Slice return type not yet supported in Kotlin bindings"),
            Type::MutSlice(_) => panic!("MutSlice return type not yet supported in Kotlin bindings"),
            Type::Object(name) => panic!("Object return type '{}' not yet supported in Kotlin bindings", name),
            Type::BoxedTrait(name) => panic!("BoxedTrait return type '{}' not yet supported in Kotlin bindings", name),
            Type::Callback(_) => panic!("Callback return type not yet supported in Kotlin bindings"),
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    pub fn is_vec(&self) -> bool {
        matches!(self, Self::Vec { .. })
    }

    pub fn is_vec_record(&self) -> bool {
        matches!(self, Self::VecRecord { .. })
    }

    pub fn reader_name(&self) -> Option<&str> {
        match self {
            Self::VecRecord { reader, .. } => Some(reader),
            _ => None,
        }
    }

    pub fn is_option(&self) -> bool {
        matches!(self, Self::Option { .. })
    }

    pub fn is_result(&self) -> bool {
        matches!(self, Self::Result { .. })
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum { .. })
    }

    pub fn inner_type(&self) -> Option<&str> {
        match self {
            Self::Vec { inner, .. } => Some(inner),
            Self::VecRecord { inner, .. } => Some(inner),
            Self::Option { inner } => Some(inner),
            Self::Result { ok } => Some(ok),
            _ => None,
        }
    }

    pub fn len_fn(&self) -> Option<&str> {
        match self {
            Self::Vec { len_fn, .. } => Some(len_fn),
            _ => None,
        }
    }

    pub fn copy_fn(&self) -> Option<&str> {
        match self {
            Self::Vec { copy_fn, .. } => Some(copy_fn),
            _ => None,
        }
    }
}

pub struct ParamConversion;

impl ParamConversion {
    pub fn to_ffi(param_name: &str, ty: &Type) -> String {
        match ty {
            Type::String => format!("{}.toByteArray()", param_name),
            Type::Bytes => param_name.to_string(),
            Type::Primitive(_) => param_name.to_string(),
            Type::Record(_) => param_name.to_string(),
            Type::Enum(_) => format!("{}.value", param_name),
            Type::Object(_) => format!("{}.handle", param_name),
            Type::Slice(_) => format!("{}.toTypedArray()", param_name),
            _ => param_name.to_string(),
        }
    }
}

// JNI-specific types for C glue generation

#[derive(Debug, Clone)]
pub enum JniReturnKind {
    Void,
    Primitive { jni_type: String },
    String { ffi_name: String },
    Vec { len_fn: String, copy_fn: String },
    Enum { ffi_name: String },
}

impl JniReturnKind {
    pub fn from_type(ty: Option<&Type>, func_name: &str) -> Self {
        use riff_ffi_rules::naming;
        match ty {
            None | Some(Type::Void) => Self::Void,
            Some(Type::Primitive(p)) => Self::Primitive {
                jni_type: super::TypeMapper::c_jni_type(&Type::Primitive(*p)),
            },
            Some(Type::String) => Self::String {
                ffi_name: naming::function_ffi_name(func_name),
            },
            Some(Type::Vec(_)) => Self::Vec {
                len_fn: naming::function_ffi_vec_len(func_name),
                copy_fn: naming::function_ffi_vec_copy_into(func_name),
            },
            Some(Type::Enum(_)) => Self::Enum {
                ffi_name: naming::function_ffi_name(func_name),
            },
            _ => Self::Void,
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String { .. })
    }

    pub fn is_vec(&self) -> bool {
        matches!(self, Self::Vec { .. })
    }

    pub fn jni_return_type(&self) -> &str {
        match self {
            Self::Void => "void",
            Self::Primitive { jni_type } => jni_type,
            Self::String { .. } => "jstring",
            Self::Vec { .. } => "jlong",
            Self::Enum { .. } => "jint",
        }
    }
}

#[derive(Debug, Clone)]
pub struct JniParamInfo {
    pub name: String,
    pub jni_type: String,
    pub is_string: bool,
    pub is_handle: bool,
}

impl JniParamInfo {
    pub fn from_param(name: &str, ty: &Type) -> Self {
        Self {
            name: name.to_string(),
            jni_type: super::TypeMapper::c_jni_type(ty),
            is_string: matches!(ty, Type::String),
            is_handle: matches!(ty, Type::Object(_) | Type::BoxedTrait(_)),
        }
    }

    pub fn jni_param_decl(&self) -> String {
        format!("{} {}", self.jni_type, self.name)
    }

    pub fn ffi_arg(&self) -> String {
        if self.is_string {
            format!("(const uint8_t*)_{}_c, {} ? strlen(_{}_c) : 0", self.name, self.name, self.name)
        } else if self.is_handle {
            format!("(void*){}", self.name)
        } else {
            self.name.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Primitive;

    #[test]
    fn test_return_kind_primitives() {
        assert!(ReturnKind::from_type(&Type::Primitive(Primitive::I32), "test").is_primitive());
        assert!(ReturnKind::from_type(&Type::Primitive(Primitive::Bool), "test").is_primitive());
    }

    #[test]
    fn test_return_kind_string() {
        assert!(ReturnKind::from_type(&Type::String, "test").is_string());
    }

    #[test]
    fn test_return_kind_vec() {
        let vec_type = Type::Vec(Box::new(Type::Primitive(Primitive::I32)));
        let kind = ReturnKind::from_type(&vec_type, "test_fn");
        assert!(kind.is_vec());
        assert_eq!(kind.len_fn(), Some("test_fn_len"));
        assert_eq!(kind.copy_fn(), Some("test_fn_copy_into"));
        assert_eq!(kind.inner_type(), Some("Int"));
    }

    #[test]
    fn test_return_kind_void() {
        assert!(ReturnKind::from_type(&Type::Void, "test").is_unit());
    }

    #[test]
    fn test_param_conversion_string() {
        assert_eq!(ParamConversion::to_ffi("name", &Type::String), "name.toByteArray()");
    }

    #[test]
    fn test_param_conversion_enum() {
        assert_eq!(
            ParamConversion::to_ffi("status", &Type::Enum("Status".into())),
            "status.value"
        );
    }

    #[test]
    fn test_param_conversion_object() {
        assert_eq!(
            ParamConversion::to_ffi("sensor", &Type::Object("Sensor".into())),
            "sensor.handle"
        );
    }

    #[test]
    fn test_param_conversion_primitive() {
        assert_eq!(
            ParamConversion::to_ffi("count", &Type::Primitive(Primitive::I32)),
            "count"
        );
    }
}
