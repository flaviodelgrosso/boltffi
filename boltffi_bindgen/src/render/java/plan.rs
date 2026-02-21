use super::JavaVersion;

#[derive(Debug, Clone)]
pub struct JavaModule {
    pub package_name: String,
    pub class_name: String,
    pub lib_name: String,
    pub java_version: JavaVersion,
    pub functions: Vec<JavaFunction>,
    pub native: JavaNative,
}

impl JavaModule {
    pub fn package_path(&self) -> String {
        self.package_name.replace('.', "/")
    }
}

#[derive(Debug, Clone)]
pub struct JavaNative {
    pub prefix: String,
    pub functions: Vec<JavaNativeFunction>,
}

#[derive(Debug, Clone)]
pub struct JavaNativeFunction {
    pub ffi_name: String,
    pub params: Vec<JavaNativeParam>,
    pub return_type: String,
}

#[derive(Debug, Clone)]
pub struct JavaNativeParam {
    pub name: String,
    pub jni_type: String,
}

#[derive(Debug, Clone)]
pub struct JavaFunction {
    pub name: String,
    pub params: Vec<JavaParam>,
    pub return_type: String,
    pub ffi_name: String,
}

#[derive(Debug, Clone)]
pub struct JavaParam {
    pub name: String,
    pub java_type: String,
}
