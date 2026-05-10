use serde::{Deserialize, Serialize};

use crate::{CallbackId, ClassId, CustomTypeId, EnumId, HandleRepr, Primitive, RecordId};

/// The value a binding declaration accepts or returns.
///
/// Higher-level than [`Primitive`]: covers the heap-managed primitives
/// the contract treats specially (`String`, `Bytes`), references to
/// user-declared types (`Record`, `Enum`, `Class`, `Callback`, `Custom`),
/// and the container shapes (`Optional`, `Sequence`, `Tuple`, `Map`).
///
/// Source spelling is gone by the time a value reaches `TypeRef`. A Rust
/// `Option<Vec<UserProfile>>` is represented as
/// `Optional(Sequence(Record(id_of_user_profile)))`; whether it renders as
/// `[UserProfile]?` in Swift or `list[UserProfile] | None` in Python is a
/// later decision.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TypeRef {
    /// Primitive scalar value.
    Primitive(Primitive),
    /// UTF-8 string value.
    String,
    /// Byte buffer value.
    Bytes,
    /// Record reference.
    Record(RecordId),
    /// Enum reference.
    Enum(EnumId),
    /// Class reference.
    Class(ClassId),
    /// Callback reference.
    Callback(CallbackId),
    /// Inline closure type.
    Closure(Box<ClosureTypeRef>),
    /// Custom type reference.
    Custom(CustomTypeId),
    /// Optional value.
    Optional(Box<TypeRef>),
    /// Sequence value.
    Sequence(Box<TypeRef>),
    /// Tuple value.
    Tuple(Vec<TypeRef>),
    /// Map value.
    Map {
        /// Key type.
        key: Box<TypeRef>,
        /// Value type.
        value: Box<TypeRef>,
    },
}

/// The result type of a callable, including the absence of a result.
///
/// `()` is meaningful in a return position and meaningless as a field or
/// parameter type, so a separate wrapper keeps the latter from accepting a
/// "void" value.
///
/// # Example
///
/// `ReturnTypeRef::Void` for `fn save() -> ()`,
/// `ReturnTypeRef::Value(TypeRef::Primitive(Primitive::I32))` for
/// `fn count() -> i32`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReturnTypeRef {
    /// The callable returns no value.
    Void,
    /// The callable returns one value.
    Value(TypeRef),
}

/// An inline closure crossing the boundary as a parameter value.
///
/// Distinct from a callback trait: a callback trait is a named declaration
/// with an id and a methods list, while a closure is an anonymous parameter
/// type. Both eventually cross as a handle, but the closure's signature is
/// recorded next to the parameter that accepts it instead of in a sibling
/// declaration.
///
/// # Example
///
/// A Rust parameter typed `impl Fn(i32) -> String` produces a
/// `ClosureTypeRef` with one `i32` parameter, a string return, and the
/// handle representation chosen for callable objects on the target.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ClosureTypeRef {
    parameters: Vec<TypeRef>,
    returns: ReturnTypeRef,
    handle: HandleRepr,
}

impl ClosureTypeRef {
    /// Builds a closure type reference.
    pub fn new(parameters: Vec<TypeRef>, returns: ReturnTypeRef, handle: HandleRepr) -> Self {
        Self {
            parameters,
            returns,
            handle,
        }
    }

    /// Returns the parameter types.
    pub fn parameters(&self) -> &[TypeRef] {
        &self.parameters
    }

    /// Returns the result type.
    pub fn returns(&self) -> &ReturnTypeRef {
        &self.returns
    }

    /// Returns the handle carrier.
    pub const fn handle(&self) -> HandleRepr {
        self.handle
    }
}
