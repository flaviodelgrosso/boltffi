use askama::Template;
use riff_ffi_rules::naming;

use super::marshal::{JniParamInfo, JniReturnKind};
use crate::model::{Class, Function, Method, Module, Type};

#[derive(Template)]
#[template(path = "kotlin/jni_glue.txt", escape = "none")]
pub struct JniGlueTemplate {
    pub prefix: String,
    pub jni_prefix: String,
    pub module_name: String,
    pub functions: Vec<JniFunctionView>,
    pub classes: Vec<JniClassView>,
}

pub struct JniFunctionView {
    pub ffi_name: String,
    pub jni_name: String,
    pub jni_return: String,
    pub jni_params: String,
    pub return_kind: JniReturnKind,
    pub params: Vec<JniParamInfo>,
    pub is_vec: bool,
    pub vec_len_ffi: String,
    pub vec_len_jni_name: String,
    pub vec_copy_ffi: String,
    pub vec_copy_jni_name: String,
    pub vec_c_type: String,
    pub vec_jni_array_type: String,
}

pub struct JniClassView {
    pub ffi_prefix: String,
    pub jni_ffi_prefix: String,
    pub jni_prefix: String,
    pub constructors: Vec<JniCtorView>,
    pub methods: Vec<JniMethodView>,
}

pub struct JniCtorView {
    pub ffi_name: String,
    pub jni_name: String,
    pub jni_params: String,
    pub params: Vec<JniParamInfo>,
}

pub struct JniMethodView {
    pub ffi_name: String,
    pub jni_name: String,
    pub jni_return: String,
    pub jni_params: String,
    pub return_kind: JniReturnKind,
    pub params: Vec<JniParamInfo>,
}

pub struct JniGenerator;

impl JniGenerator {
    pub fn generate(module: &Module, package: &str) -> String {
        let template = JniGlueTemplate::from_module(module, package);
        template.render().expect("JNI template render failed")
    }
}

impl JniGlueTemplate {
    pub fn from_module(module: &Module, package: &str) -> Self {
        let prefix = naming::ffi_prefix().to_string();
        let jni_prefix = package.replace('.', "_").replace('-', "_");

        let functions: Vec<JniFunctionView> = module
            .functions
            .iter()
            .filter(|f| !f.is_async && Self::is_supported_function(f))
            .map(|f| Self::map_function(f, &prefix, &jni_prefix))
            .collect();

        let classes: Vec<JniClassView> = module
            .classes
            .iter()
            .map(|c| Self::map_class(c, &prefix, &jni_prefix))
            .collect();

        Self {
            prefix,
            jni_prefix,
            module_name: module.name.clone(),
            functions,
            classes,
        }
    }

    fn is_supported_function(func: &Function) -> bool {
        let supported_output = match &func.output {
            None => true,
            Some(Type::Primitive(_)) => true,
            Some(Type::String) => true,
            Some(Type::Vec(inner)) => matches!(inner.as_ref(), Type::Primitive(_)),
            _ => false,
        };

        let supported_inputs = func.inputs.iter().all(|p| {
            matches!(&p.param_type, Type::Primitive(_) | Type::String)
        });

        supported_output && supported_inputs
    }

    fn is_supported_method(method: &Method) -> bool {
        let supported_output = match &method.output {
            None => true,
            Some(Type::Primitive(_)) => true,
            _ => false,
        };

        let supported_inputs = method.inputs.iter().all(|p| {
            matches!(&p.param_type, Type::Primitive(_))
        });

        supported_output && supported_inputs
    }

    fn map_function(func: &Function, prefix: &str, jni_prefix: &str) -> JniFunctionView {
        let ffi_name = format!("{}_{}", prefix, func.name);
        let jni_name = format!(
            "Java_{}_Native_{}",
            jni_prefix,
            ffi_name.replace('_', "_1")
        );

        let return_kind = JniReturnKind::from_type(func.output.as_ref(), &func.name);
        let params: Vec<JniParamInfo> = func
            .inputs
            .iter()
            .map(|p| JniParamInfo::from_param(&p.name, &p.param_type))
            .collect();

        let jni_return = return_kind.jni_return_type().to_string();
        let jni_params = if params.is_empty() {
            String::new()
        } else {
            format!(
                ", {}",
                params
                    .iter()
                    .map(|p| p.jni_param_decl())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let (is_vec, vec_len_ffi, vec_len_jni_name, vec_copy_ffi, vec_copy_jni_name, vec_c_type, vec_jni_array_type) =
            if let Some(Type::Vec(inner)) = &func.output {
                let len_ffi = naming::function_ffi_vec_len(&func.name);
                let copy_ffi = naming::function_ffi_vec_copy_into(&func.name);
                (
                    true,
                    len_ffi.clone(),
                    format!("Java_{}_Native_{}", jni_prefix, len_ffi.replace('_', "_1")),
                    copy_ffi.clone(),
                    format!("Java_{}_Native_{}", jni_prefix, copy_ffi.replace('_', "_1")),
                    Self::primitive_c_type(inner),
                    Self::primitive_jni_array_type(inner),
                )
            } else {
                (false, String::new(), String::new(), String::new(), String::new(), String::new(), String::new())
            };

        JniFunctionView {
            ffi_name,
            jni_name,
            jni_return,
            jni_params,
            return_kind,
            params,
            is_vec,
            vec_len_ffi,
            vec_len_jni_name,
            vec_copy_ffi,
            vec_copy_jni_name,
            vec_c_type,
            vec_jni_array_type,
        }
    }

    fn primitive_c_type(ty: &Type) -> String {
        match ty {
            Type::Primitive(p) => match p {
                crate::model::Primitive::I8 => "int8_t",
                crate::model::Primitive::U8 => "uint8_t",
                crate::model::Primitive::I16 => "int16_t",
                crate::model::Primitive::U16 => "uint16_t",
                crate::model::Primitive::I32 => "int32_t",
                crate::model::Primitive::U32 => "uint32_t",
                crate::model::Primitive::I64 => "int64_t",
                crate::model::Primitive::U64 => "uint64_t",
                crate::model::Primitive::Isize => "intptr_t",
                crate::model::Primitive::Usize => "uintptr_t",
                crate::model::Primitive::F32 => "float",
                crate::model::Primitive::F64 => "double",
                crate::model::Primitive::Bool => "bool",
            }
            .to_string(),
            _ => "void*".to_string(),
        }
    }

    fn primitive_jni_array_type(ty: &Type) -> String {
        match ty {
            Type::Primitive(p) => match p {
                crate::model::Primitive::I8 | crate::model::Primitive::U8 => "jbyteArray",
                crate::model::Primitive::I16 | crate::model::Primitive::U16 => "jshortArray",
                crate::model::Primitive::I32 | crate::model::Primitive::U32 => "jintArray",
                crate::model::Primitive::I64 | crate::model::Primitive::U64
                | crate::model::Primitive::Isize | crate::model::Primitive::Usize => "jlongArray",
                crate::model::Primitive::F32 => "jfloatArray",
                crate::model::Primitive::F64 => "jdoubleArray",
                crate::model::Primitive::Bool => "jbooleanArray",
            }
            .to_string(),
            _ => "jlongArray".to_string(),
        }
    }

    fn map_class(class: &Class, _prefix: &str, jni_prefix: &str) -> JniClassView {
        let ffi_prefix = naming::class_ffi_prefix(&class.name);

        let constructors: Vec<JniCtorView> = class
            .constructors
            .iter()
            .map(|ctor| {
                let ffi_name = format!("{}_new", ffi_prefix);
                let jni_name = format!(
                    "Java_{}_Native_{}_1new",
                    jni_prefix,
                    ffi_prefix.replace('_', "_1")
                );
                let params: Vec<JniParamInfo> = ctor
                    .inputs
                    .iter()
                    .map(|p| JniParamInfo::from_param(&p.name, &p.param_type))
                    .collect();
                let jni_params = if params.is_empty() {
                    String::new()
                } else {
                    format!(
                        ", {}",
                        params
                            .iter()
                            .map(|p| p.jni_param_decl())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                JniCtorView {
                    ffi_name,
                    jni_name,
                    jni_params,
                    params,
                }
            })
            .collect();

        let methods: Vec<JniMethodView> = class
            .methods
            .iter()
            .filter(|m| Self::is_supported_method(m))
            .map(|method| {
                let ffi_name = naming::method_ffi_name(&class.name, &method.name);
                let jni_name = format!(
                    "Java_{}_Native_{}",
                    jni_prefix,
                    ffi_name.replace('_', "_1")
                );
                let return_kind = JniReturnKind::from_type(method.output.as_ref(), &method.name);
                let params: Vec<JniParamInfo> = method
                    .inputs
                    .iter()
                    .map(|p| JniParamInfo::from_param(&p.name, &p.param_type))
                    .collect();
                let jni_return = return_kind.jni_return_type().to_string();
                let jni_params = if params.is_empty() {
                    String::new()
                } else {
                    format!(
                        ", {}",
                        params
                            .iter()
                            .map(|p| p.jni_param_decl())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                JniMethodView {
                    ffi_name,
                    jni_name,
                    jni_return,
                    jni_params,
                    return_kind,
                    params,
                }
            })
            .collect();

        JniClassView {
            ffi_prefix: ffi_prefix.clone(),
            jni_ffi_prefix: ffi_prefix.replace('_', "_1"),
            jni_prefix: jni_prefix.to_string(),
            constructors,
            methods,
        }
    }
}
