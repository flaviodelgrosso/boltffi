use boltffi_ffi_rules::transport::{
    ReturnInvocationContext, ReturnPlatform, ValueReturnMethod, ValueReturnStrategy,
};
use syn::Type;

use crate::lowering::returns::model::{ResolvedReturn, ReturnLoweringContext};

pub(super) struct LoweredCallbackReturn {
    resolved_return: ResolvedReturn,
}

impl LoweredCallbackReturn {
    pub(super) fn new(ty: &Type, return_lowering: &ReturnLoweringContext<'_>) -> Self {
        Self {
            resolved_return: return_lowering.lower_type(ty),
        }
    }

    pub(super) fn value_return_method(
        &self,
        return_lowering: &ReturnLoweringContext<'_>,
        context: ReturnInvocationContext,
        platform: ReturnPlatform,
    ) -> ValueReturnMethod {
        let _ = return_lowering;
        self.resolved_return.value_return_method(context, platform)
    }

    pub(super) fn uses_wire_payload(&self, return_lowering: &ReturnLoweringContext<'_>) -> bool {
        !matches!(
            self.resolved_return.value_return_strategy(),
            ValueReturnStrategy::Scalar(_)
        )
    }
}
