pub use boltffi_ffi_rules::transport::{
    EncodedReturnStrategy, ErrorReturnStrategy, ReturnInvocationContext, ReturnPlatform,
    ScalarReturnStrategy, ValueReturnMethod, ValueReturnStrategy,
};
use syn::{ReturnType, Type};

use crate::lowering::transport::NamedTypeTransportClassifier;
use crate::registries::custom_types::CustomTypeRegistry;
use crate::registries::data_types::DataTypeRegistry;

use super::classify::classify_value_return_strategy;

#[derive(Clone)]
pub struct ResolvedReturn {
    rust_type: syn::Type,
    value_return_strategy: ValueReturnStrategy,
}

impl ResolvedReturn {
    pub fn new(rust_type: syn::Type, value_return_strategy: ValueReturnStrategy) -> Self {
        Self {
            rust_type,
            value_return_strategy,
        }
    }

    pub fn rust_type(&self) -> &syn::Type {
        &self.rust_type
    }

    pub fn value_return_strategy(&self) -> ValueReturnStrategy {
        self.value_return_strategy
    }

    pub fn encoded_return_strategy(&self) -> Option<EncodedReturnStrategy> {
        match self.value_return_strategy {
            ValueReturnStrategy::Buffer(strategy) => Some(strategy),
            _ => None,
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self.value_return_strategy, ValueReturnStrategy::Void)
    }

    pub fn is_primitive_scalar(&self) -> bool {
        matches!(
            self.value_return_strategy,
            ValueReturnStrategy::Scalar(ScalarReturnStrategy::PrimitiveValue)
        )
    }

    pub fn is_passable_value(&self) -> bool {
        matches!(
            self.value_return_strategy,
            ValueReturnStrategy::Scalar(ScalarReturnStrategy::CStyleEnumTag)
                | ValueReturnStrategy::CompositeValue
        )
    }

    pub fn value_return_method(
        &self,
        context: ReturnInvocationContext,
        platform: ReturnPlatform,
    ) -> ValueReturnMethod {
        self.value_return_strategy
            .return_method(ErrorReturnStrategy::None, context, platform)
    }
}

#[derive(Clone, Copy)]
pub struct ReturnLoweringContext<'a> {
    custom_types: &'a CustomTypeRegistry,
    data_types: &'a DataTypeRegistry,
}

impl<'a> ReturnLoweringContext<'a> {
    pub fn new(custom_types: &'a CustomTypeRegistry, data_types: &'a DataTypeRegistry) -> Self {
        Self {
            custom_types,
            data_types,
        }
    }

    pub fn custom_types(&self) -> &'a CustomTypeRegistry {
        self.custom_types
    }

    pub fn data_types(&self) -> &'a DataTypeRegistry {
        self.data_types
    }

    pub(crate) fn named_type_transport_classifier(&self) -> NamedTypeTransportClassifier<'a> {
        NamedTypeTransportClassifier::new(self.custom_types, self.data_types)
    }

    pub fn lower_output(&self, output: &ReturnType) -> ResolvedReturn {
        match output {
            ReturnType::Default => {
                ResolvedReturn::new(syn::parse_quote!(()), ValueReturnStrategy::Void)
            }
            ReturnType::Type(_, rust_type) => self.lower_type(rust_type),
        }
    }

    pub fn lower_type(&self, rust_type: &Type) -> ResolvedReturn {
        ResolvedReturn::new(
            rust_type.clone(),
            classify_value_return_strategy(rust_type, self),
        )
    }
}

#[derive(Clone, Copy)]
pub struct WasmOptionScalarEncoding {
    pub(super) primitive: boltffi_ffi_rules::primitive::Primitive,
}
