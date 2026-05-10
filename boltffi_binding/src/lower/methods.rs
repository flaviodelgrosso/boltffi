//! Record-attached method and initializer lowering.
//!
//! For each [`MethodDef`] on a [`RecordDef`] the pass either produces
//! an [`InitializerDecl<S>`] (static method whose return is `Self`) or
//! a [`MethodDecl<S, NativeSymbol>`] (every other shape, including
//! methods with receivers). Both share the callable lowering in
//! [`super::callable`]; this module only owns the discriminator, the
//! symbol minting, and the record-specific `returns: ReturnTypeRef`
//! field that lives on `InitializerDecl`.
//!
//! `Result<Self, E>` initializers are not yet recognised here. They
//! become recognised at the same time error lowering lands on the
//! return path; both ends move together so the discriminator never
//! produces a value the return lowering rejects.

use boltffi_ast::{MethodDef, Receiver, RecordDef, ReturnDef, TypeExpr};

use crate::{
    CanonicalName, InitializerDecl, InitializerId, MethodDecl, MethodId, NativeSymbol,
    ReturnTypeRef, TypeRef,
};

use super::{
    LowerError, callable,
    ids::DeclarationIds,
    index::Index,
    metadata,
    surface::SurfaceLower,
    symbol::{SymbolAllocator, canonical_new_symbol_name, member_symbol_name},
};

/// Lowers every initializer-shaped method on `record`.
///
/// Iterates `record.methods` once, keeps the entries
/// [`is_initializer`] reports, and assigns each a fresh
/// [`InitializerId`] in source order. `allocator` mints the
/// [`NativeSymbol`] each initializer links against.
pub(super) fn lower_record_initializers<S: SurfaceLower>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    allocator: &mut SymbolAllocator,
    record: &RecordDef,
) -> Result<Vec<InitializerDecl<S>>, LowerError> {
    let owner = callable::CallableOwner::Record(record);
    record
        .methods
        .iter()
        .filter(|method| is_initializer(method))
        .enumerate()
        .map(|(index, method)| {
            lower_initializer::<S>(
                idx,
                ids,
                allocator,
                owner,
                record,
                method,
                InitializerId::from_raw(index as u32),
            )
        })
        .collect()
}

/// Lowers every non-initializer method on `record`.
///
/// Counterpart to [`lower_record_initializers`]: same source list,
/// inverse filter, fresh [`MethodId`] sequence.
pub(super) fn lower_record_methods<S: SurfaceLower>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    allocator: &mut SymbolAllocator,
    record: &RecordDef,
) -> Result<Vec<MethodDecl<S, NativeSymbol>>, LowerError> {
    let owner = callable::CallableOwner::Record(record);
    record
        .methods
        .iter()
        .filter(|method| !is_initializer(method))
        .enumerate()
        .map(|(index, method)| {
            lower_method::<S>(
                idx,
                ids,
                allocator,
                owner,
                method,
                MethodId::from_raw(index as u32),
            )
        })
        .collect()
}

/// Reports whether `method` becomes an [`InitializerDecl`].
///
/// The rule today: a static method (no receiver) whose return is
/// `Self`. `Result<Self, E>` is intentionally excluded until error
/// lowering on the return side recognises it; expanding that branch
/// without expanding the return side would classify methods the
/// callable lowering refuses to build.
fn is_initializer(method: &MethodDef) -> bool {
    matches!(method.receiver, Receiver::None)
        && matches!(method.returns, ReturnDef::Value(TypeExpr::SelfType))
}

fn lower_initializer<S: SurfaceLower>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    allocator: &mut SymbolAllocator,
    owner: callable::CallableOwner<'_>,
    record: &RecordDef,
    method: &MethodDef,
    id: InitializerId,
) -> Result<InitializerDecl<S>, LowerError> {
    let callable_decl = callable::lower_method::<S>(idx, ids, owner, method)?;
    let symbol = mint_method_symbol(allocator, owner, method)?;
    let record_id = ids.record(&record.id)?;
    let returns = ReturnTypeRef::Value(TypeRef::Record(record_id));
    Ok(InitializerDecl::new(
        id,
        CanonicalName::from(&method.name),
        metadata::decl_meta(method.doc.as_ref(), method.deprecated.as_ref()),
        symbol,
        callable_decl,
        returns,
    ))
}

fn lower_method<S: SurfaceLower>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    allocator: &mut SymbolAllocator,
    owner: callable::CallableOwner<'_>,
    method: &MethodDef,
    id: MethodId,
) -> Result<MethodDecl<S, NativeSymbol>, LowerError> {
    let callable_decl = callable::lower_method::<S>(idx, ids, owner, method)?;
    let symbol = mint_method_symbol(allocator, owner, method)?;
    Ok(MethodDecl::new(
        id,
        CanonicalName::from(&method.name),
        metadata::decl_meta(method.doc.as_ref(), method.deprecated.as_ref()),
        symbol,
        callable_decl,
    ))
}

/// Picks the FFI symbol name for `method` and allocates a fresh
/// [`SymbolId`].
///
/// Initializers spelled `new` collapse to the canonical
/// `boltffi_<owner>_new`; every other method name is mangled by
/// [`member_symbol_name`].
///
/// [`SymbolId`]: crate::SymbolId
fn mint_method_symbol(
    allocator: &mut SymbolAllocator,
    owner: callable::CallableOwner<'_>,
    method: &MethodDef,
) -> Result<NativeSymbol, LowerError> {
    let owner_name = owner.ffi_name();
    let method_name = method.name.parts().last().map_or("", |part| part.as_str());
    let symbol_name = if method_name == "new" {
        canonical_new_symbol_name(owner_name)
    } else {
        member_symbol_name(owner_name, method_name)
    };
    allocator.mint(symbol_name)
}
