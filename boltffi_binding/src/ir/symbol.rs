use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{BindingError, BindingErrorKind, SymbolId};

/// A linker-visible name as it appears in the compiled Rust artifact.
///
/// Foreign code calls these names through whatever FFI mechanism the
/// target uses: P/Invoke for .NET, `dlsym`/`dlopen` for POSIX, JNI for the
/// JVM, `dart:ffi` for Dart, and so on. The constructor enforces
/// C-identifier shape so foreign linkers can resolve the name without
/// escaping or quoting.
///
/// # Example
///
/// `boltffi_demo_add` is a valid `SymbolName`. `1bad_name`, `with-dash`,
/// and the empty string are not.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolName(String);

impl SymbolName {
    pub(crate) fn parse(name: impl Into<String>) -> Result<Self, BindingError> {
        let name = name.into();

        if is_valid_symbol_name(&name) {
            Ok(Self(name))
        } else {
            Err(BindingError::new(BindingErrorKind::InvalidSymbolName(name)))
        }
    }

    /// Returns the exported name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A name in the compiled Rust artifact's symbol table.
///
/// The classifier picks one `NativeSymbol` for every callable surface the
/// contract exposes (free functions, methods, initializers, accessors, the
/// helper symbols around an async call). Foreign code invokes the name
/// through the target language's FFI mechanism.
///
/// The id is stable inside the table even if a renaming pass changes the
/// spelling later, and even if two declarations want to share the same
/// exported name. Code that needs to refer to a callable across passes
/// uses the id; code that actually invokes the callable uses the name.
///
/// # Example
///
/// A Rust function `fn add(a: i32, b: i32) -> i32` exported as
/// `boltffi_demo_add` is represented by a `NativeSymbol` carrying that
/// name and the id the table assigned.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct NativeSymbol {
    id: SymbolId,
    name: SymbolName,
}

impl NativeSymbol {
    pub(crate) fn new(id: SymbolId, name: SymbolName) -> Self {
        Self { id, name }
    }

    /// Returns the symbol id.
    pub const fn id(&self) -> SymbolId {
        self.id
    }

    /// Returns the symbol name.
    pub fn name(&self) -> &SymbolName {
        &self.name
    }
}

/// The full set of native symbols a [`Bindings`](crate::Bindings) needs at
/// link time.
///
/// Listing every symbol in one place enables two things: ahead-of-time
/// verification that the compiled Rust artifact actually exports them all,
/// and id-based lookup without walking every declaration.
///
/// A held table is consistent: ids are unique, names are unique, and every
/// name is callable.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct NativeSymbolTable {
    symbols: Vec<NativeSymbol>,
}

impl NativeSymbolTable {
    pub(crate) fn from_symbols(symbols: Vec<NativeSymbol>) -> Result<Self, BindingError> {
        let table = Self { symbols };
        table.validate()?;
        Ok(table)
    }

    /// Returns the symbols in registration order.
    pub fn symbols(&self) -> &[NativeSymbol] {
        &self.symbols
    }

    /// Returns the symbol with the given id, or `None`.
    pub fn find(&self, id: SymbolId) -> Option<&NativeSymbol> {
        self.symbols.iter().find(|symbol| symbol.id == id)
    }

    /// Returns `Ok` when every name is callable, every id is unique, and
    /// every name is unique. Returns the first failed invariant otherwise.
    pub fn validate(&self) -> Result<(), BindingError> {
        validate_symbol_names(&self.symbols)?;
        validate_unique_symbol_ids(&self.symbols)?;
        validate_unique_symbol_names(&self.symbols)
    }
}

fn validate_symbol_names(symbols: &[NativeSymbol]) -> Result<(), BindingError> {
    symbols
        .iter()
        .map(NativeSymbol::name)
        .find(|name| !is_valid_symbol_name(name.as_str()))
        .map_or(Ok(()), |name| {
            Err(BindingError::new(BindingErrorKind::InvalidSymbolName(
                name.as_str().to_owned(),
            )))
        })
}

fn validate_unique_symbol_ids(symbols: &[NativeSymbol]) -> Result<(), BindingError> {
    symbols
        .iter()
        .map(NativeSymbol::id)
        .try_fold(HashSet::new(), |mut seen, symbol_id| {
            if seen.insert(symbol_id) {
                Ok(seen)
            } else {
                Err(BindingError::new(BindingErrorKind::DuplicateSymbolId(
                    symbol_id,
                )))
            }
        })
        .map(|_| ())
}

fn validate_unique_symbol_names(symbols: &[NativeSymbol]) -> Result<(), BindingError> {
    symbols
        .iter()
        .map(|symbol| symbol.name().as_str().to_owned())
        .try_fold(HashSet::new(), |mut seen, symbol_name| {
            if seen.insert(symbol_name.clone()) {
                Ok(seen)
            } else {
                Err(BindingError::new(BindingErrorKind::DuplicateSymbolName(
                    symbol_name,
                )))
            }
        })
        .map(|_| ())
}

fn is_valid_symbol_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
