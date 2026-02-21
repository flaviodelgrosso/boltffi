use crate::ir::abi::{AbiCall, AbiContract, CallId};
use crate::ir::contract::FfiContract;
use crate::ir::definitions::{FunctionDef, ReturnDef};
use crate::ir::types::TypeExpr;

use super::JavaOptions;
use super::mappings;
use super::names::NamingConvention;
use super::plan::{
    JavaFunction, JavaModule, JavaNative, JavaNativeFunction, JavaNativeParam, JavaParam,
};

pub struct JavaLowerer<'a> {
    ffi: &'a FfiContract,
    abi: &'a AbiContract,
    package_name: String,
    module_name: String,
    options: JavaOptions,
}

impl<'a> JavaLowerer<'a> {
    pub fn new(
        ffi: &'a FfiContract,
        abi: &'a AbiContract,
        package_name: String,
        module_name: String,
        options: JavaOptions,
    ) -> Self {
        Self {
            ffi,
            abi,
            package_name,
            module_name,
            options,
        }
    }

    pub fn module(&self) -> JavaModule {
        let lib_name = self
            .options
            .library_name
            .clone()
            .unwrap_or_else(|| self.module_name.clone());

        let prefix = boltffi_ffi_rules::naming::ffi_prefix().to_string();

        let functions: Vec<JavaFunction> = self
            .ffi
            .functions
            .iter()
            .filter(|f| self.is_primitive_function(f))
            .map(|f| self.lower_function(f))
            .collect();

        let native_functions: Vec<JavaNativeFunction> = self
            .ffi
            .functions
            .iter()
            .filter(|f| self.is_primitive_function(f))
            .map(|f| self.lower_native_function(f))
            .collect();

        JavaModule {
            package_name: self.package_name.clone(),
            class_name: NamingConvention::class_name(&self.module_name),
            lib_name,
            java_version: self.options.min_java_version,
            functions,
            native: JavaNative {
                prefix,
                functions: native_functions,
            },
        }
    }

    fn is_primitive_function(&self, func: &FunctionDef) -> bool {
        let params_primitive = func
            .params
            .iter()
            .all(|p| self.is_primitive_type(&p.type_expr));
        let return_primitive = match &func.returns {
            ReturnDef::Void => true,
            ReturnDef::Value(ty) => self.is_primitive_type(ty),
            ReturnDef::Result { .. } => false,
        };
        params_primitive && return_primitive && !func.is_async
    }

    fn is_primitive_type(&self, ty: &TypeExpr) -> bool {
        matches!(ty, TypeExpr::Primitive(_))
    }

    fn lower_function(&self, func: &FunctionDef) -> JavaFunction {
        let call = self.abi_call_for_function(func);

        let params: Vec<JavaParam> = func
            .params
            .iter()
            .map(|p| JavaParam {
                name: NamingConvention::field_name(p.name.as_str()),
                java_type: self.java_type(&p.type_expr),
            })
            .collect();

        let return_type = match &func.returns {
            ReturnDef::Void => "void".to_string(),
            ReturnDef::Value(ty) => self.java_type(ty),
            ReturnDef::Result { .. } => "void".to_string(),
        };

        JavaFunction {
            name: NamingConvention::method_name(func.id.as_str()),
            params,
            return_type,
            ffi_name: call.symbol.as_str().to_string(),
        }
    }

    fn lower_native_function(&self, func: &FunctionDef) -> JavaNativeFunction {
        let call = self.abi_call_for_function(func);

        let params: Vec<JavaNativeParam> = func
            .params
            .iter()
            .map(|p| JavaNativeParam {
                name: NamingConvention::field_name(p.name.as_str()),
                jni_type: self.jni_type(&p.type_expr),
            })
            .collect();

        let return_type = match &func.returns {
            ReturnDef::Void => "void".to_string(),
            ReturnDef::Value(ty) => self.jni_type(ty),
            ReturnDef::Result { .. } => "void".to_string(),
        };

        JavaNativeFunction {
            ffi_name: call.symbol.as_str().to_string(),
            params,
            return_type,
        }
    }

    fn abi_call_for_function(&self, func: &FunctionDef) -> &AbiCall {
        self.abi
            .calls
            .iter()
            .find(|c| matches!(&c.id, CallId::Function(id) if id == &func.id))
            .expect("abi call not found for function")
    }

    fn java_type(&self, ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Primitive(p) => mappings::java_type(*p).to_string(),
            _ => "Object".to_string(),
        }
    }

    fn jni_type(&self, ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Primitive(p) => mappings::jni_type(*p).to_string(),
            _ => "Object".to_string(),
        }
    }
}
