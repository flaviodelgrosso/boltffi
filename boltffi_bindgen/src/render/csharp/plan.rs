use boltffi_ffi_rules::naming::{LibraryName, Name};

/// Represents a lowered C# module, containing everything the templates need
/// to render a `.cs` file.
#[derive(Debug, Clone)]
pub struct CSharpModule {
    /// C# namespace for the generated file (e.g., `"MyApp"`).
    pub namespace: String,
    /// Top-level class name (e.g., `"MyApp"`).
    pub class_name: String,
    /// Native library name used in `[DllImport("...")]` declarations.
    pub lib_name: Name<LibraryName>,
    /// FFI symbol prefix (e.g., `"boltffi"`).
    pub prefix: String,
    /// Top-level primitive functions. Used by both the public wrapper class
    /// and the `[DllImport]` native declarations — C# P/Invoke passes
    /// primitives directly, so one struct serves both layers.
    pub functions: Vec<CSharpFunction>,
}

impl CSharpModule {
    pub fn has_functions(&self) -> bool {
        !self.functions.is_empty()
    }
}

/// A primitive function binding. Serves double duty: the template uses `name`
/// and C# types for the public static method, and `ffi_name` for the
/// `[DllImport]` entry point.
#[derive(Debug, Clone)]
pub struct CSharpFunction {
    /// PascalCase method name (e.g., `"EchoI32"`).
    pub name: String,
    /// Parameters with C# types.
    pub params: Vec<CSharpParam>,
    /// C# return type (e.g., `"int"`, `"void"`).
    pub return_type: String,
    /// The C symbol name (e.g., `"boltffi_echo_i32"`).
    pub ffi_name: String,
}

impl CSharpFunction {
    pub fn is_void(&self) -> bool {
        self.return_type == "void"
    }
}

/// A parameter in a C# function.
#[derive(Debug, Clone)]
pub struct CSharpParam {
    /// camelCase parameter name, keyword-escaped with `@` if needed.
    pub name: String,
    /// C# type (e.g., `"int"`, `"double"`, `"bool"`).
    pub csharp_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn function_with_return(return_type: &str) -> CSharpFunction {
        CSharpFunction {
            name: "Test".to_string(),
            params: vec![],
            return_type: return_type.to_string(),
            ffi_name: "boltffi_test".to_string(),
        }
    }

    #[rstest]
    #[case::void("void", true)]
    #[case::int("int", false)]
    #[case::bool("bool", false)]
    #[case::double("double", false)]
    fn is_void(#[case] return_type: &str, #[case] expected: bool) {
        assert_eq!(function_with_return(return_type).is_void(), expected);
    }
}
