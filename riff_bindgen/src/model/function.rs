use serde::{Deserialize, Serialize};

use super::method::Parameter;
use super::types::{Deprecation, Type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Parameter>,
    pub output: Option<Type>,
    pub error: Option<Type>,
    pub is_async: bool,
    pub doc: Option<String>,
    pub deprecated: Option<Deprecation>,
}

impl Function {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            inputs: Vec::new(),
            output: None,
            error: None,
            is_async: false,
            doc: None,
            deprecated: None,
        }
    }

    pub fn with_param(mut self, param: Parameter) -> Self {
        self.inputs.push(param);
        self
    }

    pub fn with_output(mut self, output: Type) -> Self {
        self.output = Some(output);
        self
    }

    pub fn with_error(mut self, error: Type) -> Self {
        self.error = Some(error);
        self
    }

    pub fn make_async(mut self) -> Self {
        self.is_async = true;
        self
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn with_deprecated(mut self, deprecation: Deprecation) -> Self {
        self.deprecated = Some(deprecation);
        self
    }

    pub fn throws(&self) -> bool {
        self.error.is_some()
    }

    pub fn is_deprecated(&self) -> bool {
        self.deprecated.is_some()
    }

    pub fn has_return_value(&self) -> bool {
        self.output.as_ref().is_some_and(|output| !output.is_void())
    }

    pub fn has_callbacks(&self) -> bool {
        self.inputs
            .iter()
            .any(|p| matches!(p.param_type, Type::Callback(_)))
    }

    pub fn callback_params(&self) -> impl Iterator<Item = &Parameter> {
        self.inputs
            .iter()
            .filter(|p| matches!(p.param_type, Type::Callback(_)))
    }

    pub fn non_callback_params(&self) -> impl Iterator<Item = &Parameter> {
        self.inputs
            .iter()
            .filter(|p| !matches!(p.param_type, Type::Callback(_)))
    }
}
