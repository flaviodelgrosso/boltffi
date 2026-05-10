use serde::{Deserialize, Serialize};

use crate::{
    CanonicalName, ElementMeta, HandleRepr, IntegerRepr, NativeSymbol, ReadPlan, ReturnTypeRef,
    TypeRef, WritePlan,
};

/// One callable surface ready to be turned into target code.
///
/// Carries every decision about how the call crosses the boundary: which
/// native symbol to invoke, how the receiver participates, how each
/// argument enters Rust, how the result leaves, how errors are reported,
/// and whether the call is synchronous or asynchronous.
///
/// Same shape whether the caller is a free function, a method, an
/// initializer, or a constant accessor. The owner adds its own context
/// (the type, the visibility, the binding name) on top.
///
/// # Example
///
/// `fn add(a: i32, b: i32) -> i32` becomes a `CallableDecl` with two
/// direct-scalar parameters, a direct-scalar return, no error transport,
/// synchronous execution, and a native symbol such as
/// `boltffi_demo_add`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CallableDecl {
    symbol: NativeSymbol,
    receiver: ReceiverDecl,
    params: Vec<ParamDecl>,
    returns: ReturnDecl,
    error: ErrorDecl,
    execution: ExecutionDecl,
}

impl CallableDecl {
    pub(crate) fn new(
        symbol: NativeSymbol,
        receiver: ReceiverDecl,
        params: Vec<ParamDecl>,
        returns: ReturnDecl,
        error: ErrorDecl,
        execution: ExecutionDecl,
    ) -> Self {
        Self {
            symbol,
            receiver,
            params,
            returns,
            error,
            execution,
        }
    }

    /// Returns the native symbol.
    pub fn symbol(&self) -> &NativeSymbol {
        &self.symbol
    }

    /// Returns the receiver shape.
    pub fn receiver(&self) -> &ReceiverDecl {
        &self.receiver
    }

    /// Returns the parameters in call order.
    pub fn params(&self) -> &[ParamDecl] {
        &self.params
    }

    /// Returns the return shape.
    pub fn returns(&self) -> &ReturnDecl {
        &self.returns
    }

    /// Returns the error transport.
    pub fn error(&self) -> &ErrorDecl {
        &self.error
    }

    /// Returns the execution mode.
    pub fn execution(&self) -> &ExecutionDecl {
        &self.execution
    }
}

/// How a method-like callable handles its receiver.
///
/// `Shared` is `&self`, `Mutable` is `&mut self`, `Owned` is `self`, and
/// `None` covers free functions and static methods. The lower plan records
/// what value actually crosses (a handle, a record by direct memory, an
/// encoded payload), so a renderer does not infer the receiver shape from
/// Rust syntax.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReceiverDecl {
    /// Free function or static method.
    None,
    /// Shared reference receiver.
    Shared(LowerPlan),
    /// Mutable reference receiver.
    Mutable(LowerPlan),
    /// Owning receiver.
    Owned(LowerPlan),
}

/// One parameter accepted by a callable.
///
/// Carries the canonical name the source wrote, the logical type the
/// parameter accepts, the lower plan that says how the foreign value
/// becomes a Rust value at call time, and per-element metadata for docs
/// and defaults.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ParamDecl {
    name: CanonicalName,
    ty: TypeRef,
    lower: LowerPlan,
    meta: ElementMeta,
}

impl ParamDecl {
    pub(crate) fn new(
        name: CanonicalName,
        ty: TypeRef,
        lower: LowerPlan,
        meta: ElementMeta,
    ) -> Self {
        Self {
            name,
            ty,
            lower,
            meta,
        }
    }

    /// Returns the parameter name.
    pub fn name(&self) -> &CanonicalName {
        &self.name
    }

    /// Returns the parameter type.
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }

    /// Returns the lower plan.
    pub fn lower(&self) -> &LowerPlan {
        &self.lower
    }

    /// Returns the element metadata.
    pub fn meta(&self) -> &ElementMeta {
        &self.meta
    }
}

/// The result a callable produces.
///
/// Carries the logical return type and the lift plan that says how a Rust
/// value becomes a foreign value before control returns. A void return is
/// represented as [`LiftPlan::Void`] rather than as the absence of a
/// `ReturnDecl`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ReturnDecl {
    ty: ReturnTypeRef,
    lift: LiftPlan,
    meta: ElementMeta,
}

impl ReturnDecl {
    pub(crate) fn new(ty: ReturnTypeRef, lift: LiftPlan, meta: ElementMeta) -> Self {
        Self { ty, lift, meta }
    }

    /// Returns the return type.
    pub fn ty(&self) -> &ReturnTypeRef {
        &self.ty
    }

    /// Returns the lift plan.
    pub fn lift(&self) -> &LiftPlan {
        &self.lift
    }

    /// Returns the element metadata.
    pub fn meta(&self) -> &ElementMeta {
        &self.meta
    }
}

/// How a foreign-language value enters Rust at call time.
///
/// `Direct` passes the value through native ABI registers as-is. `Encoded`
/// writes the value into the contract's wire format on the calling side
/// and reconstructs it on the Rust side. `Handle` carries an opaque
/// integer whose meaning is owned by Rust.
///
/// # Example
///
/// A primitive `i32` argument lowers as
/// `Direct(TypeRef::Primitive(Primitive::I32))`. A `String` argument
/// lowers as `Encoded(WritePlan { ... })` so its bytes can cross without
/// copying through every layer.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LowerPlan {
    /// Pass the value directly through the native ABI.
    Direct(TypeRef),
    /// Cross the value through the encoded representation.
    Encoded(WritePlan),
    /// Cross the value as an opaque handle.
    Handle(HandleRepr),
}

/// How a Rust return value reaches foreign code.
///
/// Mirror of [`LowerPlan`] for the return direction. `Void` is a separate
/// variant because the absence of a value cannot be represented by any
/// other lift.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LiftPlan {
    /// The callable returns no value.
    Void,
    /// Read the value directly from the native return slot.
    Direct(TypeRef),
    /// Read the value from the encoded representation.
    Encoded(ReadPlan),
    /// Read an opaque handle.
    Handle(HandleRepr),
}

/// How a fallible callable reports a Rust error to foreign code.
///
/// `None` means the callable cannot fail across the boundary. `StatusCode`
/// returns an integer where one designated value is success and the
/// others map to specific failures. `Encoded` carries the failure value
/// as a payload, the same way a successful value would.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ErrorDecl {
    /// Cannot fail across the boundary.
    None,
    /// Reports failure through an integer status value.
    StatusCode {
        /// Integer representation used for the status.
        repr: IntegerRepr,
    },
    /// Reports failure through an encoded error value.
    Encoded {
        /// Logical error type.
        ty: TypeRef,
        /// Plan used to read the error value.
        read: ReadPlan,
    },
}

/// Whether a callable returns immediately or through an async protocol.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExecutionDecl {
    /// Control returns when the call returns.
    Synchronous,
    /// Control returns through an async protocol.
    Asynchronous(AsyncDecl),
}

/// The async protocol selected for a callable.
///
/// `NativeFuture` returns a runtime-native future-like value to the
/// foreign side. `Continuation` runs to completion in Rust and invokes a
/// callback symbol when finished. `PollHandle` returns a handle the
/// foreign side polls until completion, then extracts the result and
/// releases the handle.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AsyncDecl {
    /// Returns a runtime-native future-like value.
    NativeFuture,
    /// Reports completion by invoking a continuation symbol.
    Continuation {
        /// Native symbol used to deliver completion.
        symbol: NativeSymbol,
    },
    /// Returns a handle the foreign side polls.
    PollHandle {
        /// Handle carrier for the async state.
        handle: HandleRepr,
        /// Native symbol used to poll the state.
        poll: NativeSymbol,
        /// Native symbol used to extract the result on completion.
        complete: NativeSymbol,
        /// Native symbol used to release the state.
        free: NativeSymbol,
    },
}
