//! Lowers a scanned Rust source contract into a binding contract for a
//! target [`Surface`].
//!
//! The pass runs once. The returned [`Bindings<S>`] contains the
//! decisions consumers render: direct records carry layout, encoded
//! records carry codec plans, and enums have already been split into
//! c-style or data-bearing forms. Source shapes that do not have a
//! binding-IR representation yet return [`LowerError`] instead of being
//! guessed.
//!
//! # Pipeline
//!
//! 1. Build [`DeclarationIds`] from the source. Duplicate ids in the
//!    same family fail here, before any walk.
//! 2. Reject declaration families that have no IR slice yet (functions,
//!    classes, callbacks, streams, constants, custom types) and methods
//!    on records or enums.
//! 3. Build an [`Index`] of the source for cross-decl lookups during
//!    type and codec lowering.
//! 4. Lower every record into [`RecordDecl<S>`] and every enum into
//!    [`EnumDecl<S>`].
//! 5. Hand the collected decls to [`Bindings::from_decls`], which
//!    derives the native symbol table from the symbols the decls
//!    reference and validates the result.
//!
//! Each step in the pipeline returns either final IR or the
//! infrastructure the next step uses; nothing returns a private
//! domain-shaped middle value.
//!
//! The surface is selected at the call site:
//!
//! ```ignore
//! let native = boltffi_binding::lower::<boltffi_binding::Native>(&source)?;
//! let wasm   = boltffi_binding::lower::<boltffi_binding::Wasm32>(&source)?;
//! ```

mod codecs;
mod enums;
mod error;
mod ids;
mod index;
mod layout;
mod metadata;
mod names;
mod primitive;
mod records;
mod types;

use boltffi_ast::SourceContract;

use crate::{Bindings, BindingError, CanonicalName, Decl, PackageInfo, Surface};

pub use self::error::{DeclarationFamily, LowerError, LowerErrorKind, UnsupportedType};

use self::{ids::DeclarationIds, index::Index};

/// Lowers a source contract into a binding contract for surface `S`.
///
/// See the module-level docs for the steps each call runs through.
pub fn lower<S: Surface>(source: &SourceContract) -> Result<Bindings<S>, LowerError> {
    let ids = DeclarationIds::from_source(source)?;
    reject_unsupported(source)?;

    let index = Index::new(source);

    let records = records::lower::<S>(&index, &ids)?;
    let enums = enums::lower::<S>(&index, &ids)?;

    let decls = records
        .into_iter()
        .map(|record| Decl::Record(Box::new(record)))
        .chain(
            enums
                .into_iter()
                .map(|enumeration| Decl::Enum(Box::new(enumeration))),
        )
        .collect::<Vec<_>>();

    let package = PackageInfo::new(
        CanonicalName::single(source.package.name.as_str()),
        source.package.version.clone(),
    );

    Ok(Bindings::from_decls(package, decls)?)
}

fn reject_unsupported(source: &SourceContract) -> Result<(), LowerError> {
    [
        (!source.functions.is_empty(), DeclarationFamily::Functions),
        (!source.classes.is_empty(), DeclarationFamily::Classes),
        (
            !source.callback_traits.is_empty(),
            DeclarationFamily::CallbackTraits,
        ),
        (!source.streams.is_empty(), DeclarationFamily::Streams),
        (!source.constants.is_empty(), DeclarationFamily::Constants),
        (!source.customs.is_empty(), DeclarationFamily::CustomTypes),
        (
            source
                .records
                .iter()
                .any(|record| !record.methods.is_empty()),
            DeclarationFamily::RecordMethods,
        ),
        (
            source
                .enums
                .iter()
                .any(|enumeration| !enumeration.methods.is_empty()),
            DeclarationFamily::EnumMethods,
        ),
    ]
    .into_iter()
    .find_map(|(present, declaration)| present.then_some(declaration))
    .map_or(Ok(()), |declaration| {
        Err(LowerError::unsupported_declaration(declaration))
    })
}

impl From<BindingError> for LowerError {
    fn from(error: BindingError) -> Self {
        Self::new(LowerErrorKind::InvalidBindings(error))
    }
}
