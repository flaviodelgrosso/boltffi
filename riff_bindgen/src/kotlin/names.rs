use heck::{ToLowerCamelCase, ToUpperCamelCase};
use riff_ffi_rules::naming;

pub struct NamingConvention;

impl NamingConvention {
    pub fn class_name(name: &str) -> String {
        name.to_upper_camel_case()
    }

    pub fn method_name(name: &str) -> String {
        let converted = name.to_lower_camel_case();
        Self::escape_keyword(&converted)
    }

    pub fn param_name(name: &str) -> String {
        let converted = name.to_lower_camel_case();
        Self::escape_keyword(&converted)
    }

    pub fn property_name(name: &str) -> String {
        let converted = name.to_lower_camel_case();
        Self::escape_keyword(&converted)
    }

    pub fn enum_entry_name(name: &str) -> String {
        name.to_uppercase()
    }

    pub fn escape_keyword(name: &str) -> String {
        if Self::is_kotlin_keyword(name) {
            format!("`{}`", name)
        } else {
            name.to_string()
        }
    }

    fn is_kotlin_keyword(name: &str) -> bool {
        matches!(
            name,
            "as" | "break"
                | "class"
                | "continue"
                | "do"
                | "else"
                | "false"
                | "for"
                | "fun"
                | "if"
                | "in"
                | "interface"
                | "is"
                | "null"
                | "object"
                | "package"
                | "return"
                | "super"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typealias"
                | "typeof"
                | "val"
                | "var"
                | "when"
                | "while"
                | "by"
                | "catch"
                | "constructor"
                | "delegate"
                | "dynamic"
                | "field"
                | "file"
                | "finally"
                | "get"
                | "import"
                | "init"
                | "param"
                | "property"
                | "receiver"
                | "set"
                | "setparam"
                | "value"
                | "where"
                | "actual"
                | "abstract"
                | "annotation"
                | "companion"
                | "const"
                | "crossinline"
                | "data"
                | "enum"
                | "expect"
                | "external"
                | "final"
                | "infix"
                | "inline"
                | "inner"
                | "internal"
                | "lateinit"
                | "noinline"
                | "open"
                | "operator"
                | "out"
                | "override"
                | "private"
                | "protected"
                | "public"
                | "reified"
                | "sealed"
                | "suspend"
                | "tailrec"
                | "vararg"
        )
    }

    pub fn ffi_prefix() -> String {
        naming::ffi_prefix().to_string()
    }

    pub fn class_ffi_prefix(class_name: &str) -> String {
        naming::class_ffi_prefix(class_name)
    }

    pub fn ffi_module_name(crate_name: &str) -> String {
        naming::ffi_module_name(crate_name)
    }

    pub fn jni_class_path(package: &str, class_name: &str) -> String {
        format!("{}.{}", package, class_name)
    }
}
