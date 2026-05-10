//! AST-to-binding classification internals.
//!
//! This lane will own the single classifier that turns `boltffi_ast` source
//! data into [`Bindings`](crate::ir::Bindings). It is not part of the backend
//! render contract.
