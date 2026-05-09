//! The binding contract for an FFI-exported Rust API.
//!
//! BoltFFI turns a Rust crate annotated with `#[data]`, `#[export]`, and
//! `#[data(impl)]` into target-language source. The path from one to the
//! other runs through several crates; this one owns the middle stage.
//!
//! # The pipeline
//!
//! ```text
//!   user crate (Rust)
//!         │
//!         │  boltffi_macros scan the source
//!         ▼
//!   boltffi_ast::SourceContract           ← what the user wrote
//!         │
//!         │  classify (this crate)
//!         ▼
//!   Bindings                              ← what crosses, in what shape
//!         │
//!         │  expand (this crate, via boltffi_macros)
//!         ▼
//!   Rust glue + serialized metadata in the user's .rlib
//!         │
//!         │  boltffi_bindgen reads the metadata back into Bindings
//!         ▼
//!   per-language source                   ← Swift, Kotlin, Python, C, …
//! ```
//!
//! `boltffi_ast` records what the user wrote. This crate decides what
//! that source means at the FFI boundary: a record is direct or encoded,
//! an enum is C-style or data-bearing, a callable gets concrete lower
//! and lift plans, a native symbol is picked and validated. The
//! classifier runs once; nothing downstream re-classifies the same source.
//!
//! # Three lanes
//!
//! `classify` translates a `SourceContract` into a [`Bindings`]. It is
//! the single classifier in the system. `boltffi_macros` invokes it while
//! expanding the user's crate. `boltffi_bindgen` does not invoke it;
//! bindgen reconstructs the same [`Bindings`] from the metadata embedded
//! in the user's compiled artifact.
//!
//! `expand` is the lane the macros use to emit Rust glue. Each
//! `#[data]` or `#[export]` item in the user's source needs an
//! `extern "C"` wrapper, a `Passable` or `WirePassable` impl, and an
//! entry in the serialized metadata. This lane pairs each AST item with
//! its classified counterpart so the macros emit that glue without
//! classifying anything themselves.
//!
//! [`ir`] is the public surface. Every type a downstream consumer
//! touches lives there: [`Bindings`], the [`Decl`] enum and its
//! per-family variants, codec and op plans, native symbol tables,
//! layouts, metadata, and errors. `boltffi_bindgen` and the backend
//! crates import [`ir`] and nothing else.
//!
//! `classify` and `expand` are private. [`ir`] is the entry point for
//! anything outside this crate.
//!
//! # What this crate does not do
//!
//! No target-language code generation. No filesystem writes. No
//! dependency on any specific backend. The crate ends with a [`Bindings`]
//! value; turning that into Swift, Kotlin, Python, or any other target
//! lives in separate backend crates.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod classify;
mod expand;
pub mod ir;

pub use ir::*;
