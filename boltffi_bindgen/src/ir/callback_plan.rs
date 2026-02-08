//! Legacy callback plan types for non-ABI backends.
//!
//! DEPRECATED: Use `AbiCallbackInvocation` from `ir::abi` instead.
//! This module exists only for backends that haven't migrated to AbiContract yet.
//! TODO: Remove after Kotlin backend migrates to AbiContract.

use crate::ir::codec::CodecPlan;
use crate::ir::ids::{CallbackId, MethodId, ParamName};
use crate::ir::plan::DirectPlan;
use boltffi_ffi_rules::naming::{CreateFn, Name, RegisterFn, VtableField, VtableType};

/// Legacy callback invocation plan.
///
/// DEPRECATED: Use `AbiCallbackInvocation` instead.
#[derive(Debug, Clone)]
pub struct CallbackInvocationPlan {
    pub callback_id: CallbackId,
    pub vtable_type: Name<VtableType>,
    pub register_fn: Name<RegisterFn>,
    pub create_fn: Name<CreateFn>,
    pub methods: Vec<CallbackMethodPlan>,
}

#[derive(Debug, Clone)]
pub struct CallbackMethodPlan {
    pub id: MethodId,
    pub vtable_field: Name<VtableField>,
    pub params: Vec<CallbackParamPlan>,
    pub returns: CallbackReturnPlan,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct CallbackParamPlan {
    pub name: ParamName,
    pub strategy: CallbackParamStrategy,
}

#[derive(Debug, Clone)]
pub enum CallbackParamStrategy {
    Direct(DirectPlan),
    Encoded { codec: CodecPlan },
}

#[derive(Debug, Clone)]
pub enum CallbackReturnPlan {
    Void,
    Direct(DirectPlan),
    Encoded { codec: CodecPlan },
    Async { completion_codec: Option<CodecPlan> },
}
