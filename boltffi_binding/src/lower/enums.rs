use boltffi_ast::{
    EnumDef as SourceEnum, TypeExpr, VariantDef as SourceVariant, VariantPayload as SourcePayload,
};

use crate::{
    CStyleEnumDecl, CStyleVariantDecl, CanonicalName, DataEnumDecl, DataVariantDecl,
    DataVariantPayload, EncodedFieldDecl, EnumDecl, FieldKey, IntegerRepr, IntegerValue, Surface,
    ValueRef, VariantTag,
};

use super::{
    LowerError, codecs, error::UnsupportedType, ids::DeclarationIds, index::Index, metadata,
    primitive,
};

/// Lowers every enum in the source contract.
pub(super) fn lower<S: Surface>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
) -> Result<Vec<EnumDecl<S>>, LowerError> {
    idx.enums()
        .iter()
        .map(|enumeration| lower_one(idx, ids, enumeration))
        .collect()
}

/// Reports whether a source enum codes as a C-style integer
/// discriminant.
///
/// Exposed to the codec lane so a nested `TypeExpr::Enum(id)` agrees
/// with the enum's own declaration on `CStyleEnum` vs `DataEnum`.
pub(super) fn is_c_style(enumeration: &SourceEnum) -> bool {
    enumeration
        .variants
        .iter()
        .all(|variant| matches!(variant.payload, SourcePayload::Unit))
        && effective_integer_repr(enumeration).is_some()
}

fn lower_one<S: Surface>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    enumeration: &SourceEnum,
) -> Result<EnumDecl<S>, LowerError> {
    if is_c_style(enumeration) {
        lower_c_style(ids, enumeration).map(EnumDecl::CStyle)
    } else {
        lower_data(idx, ids, enumeration).map(|enumeration| EnumDecl::Data(Box::new(enumeration)))
    }
}

fn lower_c_style<S: Surface>(
    ids: &DeclarationIds,
    enumeration: &SourceEnum,
) -> Result<CStyleEnumDecl<S>, LowerError> {
    Ok(CStyleEnumDecl::new(
        ids.enumeration(&enumeration.id)?,
        CanonicalName::from(&enumeration.name),
        metadata::decl_meta(enumeration.doc.as_ref(), enumeration.deprecated.as_ref()),
        effective_integer_repr(enumeration)
            .ok_or_else(|| LowerError::unsupported_type(UnsupportedType::EnumRepr))?,
        discriminants(&enumeration.variants)?
            .into_iter()
            .map(|(variant, discriminant)| {
                Ok(CStyleVariantDecl::new(
                    CanonicalName::from(&variant.name),
                    IntegerValue::new(discriminant),
                    metadata::element_meta(variant.doc.as_ref(), None, None)?,
                ))
            })
            .collect::<Result<Vec<_>, LowerError>>()?,
        Vec::new(),
    ))
}

fn lower_data<S: Surface>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    enumeration: &SourceEnum,
) -> Result<DataEnumDecl<S>, LowerError> {
    Ok(DataEnumDecl::new(
        ids.enumeration(&enumeration.id)?,
        CanonicalName::from(&enumeration.name),
        metadata::decl_meta(enumeration.doc.as_ref(), enumeration.deprecated.as_ref()),
        enumeration
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| lower_variant(idx, ids, index, variant))
            .collect::<Result<Vec<_>, LowerError>>()?,
        Vec::new(),
        codecs::plan(
            idx,
            ids,
            &TypeExpr::Enum(enumeration.id.clone()),
            ValueRef::self_value(),
        )?,
    ))
}

fn lower_variant(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    index: usize,
    variant: &SourceVariant,
) -> Result<DataVariantDecl, LowerError> {
    Ok(DataVariantDecl::new(
        CanonicalName::from(&variant.name),
        VariantTag::new(index as u32),
        lower_payload(idx, ids, &variant.payload)?,
        metadata::element_meta(variant.doc.as_ref(), None, None)?,
    ))
}

fn lower_payload(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    payload: &SourcePayload,
) -> Result<DataVariantPayload, LowerError> {
    match payload {
        SourcePayload::Unit => Ok(DataVariantPayload::Unit),
        SourcePayload::Tuple(types) => types
            .iter()
            .enumerate()
            .map(|(index, type_expr)| {
                let key = FieldKey::Position(index as u32);
                let value = ValueRef::self_value().field(key.clone());
                let ty = super::types::lower(ids, type_expr)?;
                let codec = codecs::plan(idx, ids, type_expr, value)?;
                Ok(EncodedFieldDecl::new(key, ty, codec, Default::default()))
            })
            .collect::<Result<Vec<_>, LowerError>>()
            .map(DataVariantPayload::Tuple),
        SourcePayload::Struct(fields) => fields
            .iter()
            .map(|field| {
                let key = FieldKey::from(field);
                let value = ValueRef::self_value().field(key.clone());
                let ty = super::types::lower(ids, &field.type_expr)?;
                let codec = codecs::plan(idx, ids, &field.type_expr, value)?;
                Ok(EncodedFieldDecl::new(
                    key,
                    ty,
                    codec,
                    metadata::element_meta(field.doc.as_ref(), None, field.default.as_ref())?,
                ))
            })
            .collect::<Result<Vec<_>, LowerError>>()
            .map(DataVariantPayload::Struct),
    }
}

fn effective_integer_repr(enumeration: &SourceEnum) -> Option<IntegerRepr> {
    primitive::integer_repr(&enumeration.repr).or_else(|| {
        (enumeration
            .variants
            .iter()
            .all(|variant| matches!(variant.payload, SourcePayload::Unit))
            && enumeration.repr.items.is_empty())
        .then_some(IntegerRepr::I32)
    })
}

fn discriminants<'variant>(
    variants: &'variant [SourceVariant],
) -> Result<Vec<(&'variant SourceVariant, i128)>, LowerError> {
    variants
        .iter()
        .try_fold((0_i128, Vec::new()), |(next, mut variants), variant| {
            let discriminant = variant.discriminant.unwrap_or(next);
            variants.push((variant, discriminant));
            let next = discriminant
                .checked_add(1)
                .ok_or_else(LowerError::discriminant_overflow)?;
            Ok((next, variants))
        })
        .map(|(_, variants)| variants)
}

#[cfg(test)]
mod tests {
    use boltffi_ast::{
        CanonicalName as SourceName, EnumDef, PackageInfo as SourcePackage, Primitive, ReprAttr,
        ReprItem, SourceContract, TypeExpr, VariantDef, VariantPayload,
    };

    use crate::{Decl, EnumDecl, IntegerRepr, Native, lower};

    fn package() -> SourceContract {
        SourceContract::new(SourcePackage::new("demo", Some("0.1.0".to_owned())))
    }

    fn name(part: &str) -> SourceName {
        SourceName::single(part)
    }

    fn unit_variant_with_discriminant(variant_name: &str, discriminant: i128) -> VariantDef {
        let mut variant = VariantDef::unit(name(variant_name));
        variant.discriminant = Some(discriminant);
        variant
    }

    fn enumeration(id: &str, enum_name: &str, variants: Vec<VariantDef>) -> EnumDef {
        let mut enumeration = EnumDef::new(id.into(), name(enum_name));
        enumeration.variants = variants;
        enumeration
    }

    fn c_style_enum(bindings: &crate::Bindings<Native>) -> &crate::CStyleEnumDecl<Native> {
        match bindings.decls().first() {
            Some(Decl::Enum(enumeration)) => match enumeration.as_ref() {
                EnumDecl::CStyle(enumeration) => enumeration,
                EnumDecl::Data(_) => panic!("expected c-style enum"),
            },
            _ => panic!("expected enum declaration"),
        }
    }

    fn data_enum(bindings: &crate::Bindings<Native>) -> &crate::DataEnumDecl<Native> {
        match bindings.decls().first() {
            Some(Decl::Enum(enumeration)) => match enumeration.as_ref() {
                EnumDecl::Data(enumeration) => enumeration,
                EnumDecl::CStyle(_) => panic!("expected data enum"),
            },
            _ => panic!("expected enum declaration"),
        }
    }

    #[test]
    fn classifies_c_style_enum_without_repr_as_i32() {
        let mut contract = package();
        contract.enums.push(enumeration(
            "demo::Direction",
            "direction",
            vec![
                VariantDef::unit(name("north")),
                VariantDef::unit(name("south")),
            ],
        ));

        let bindings = lower::<Native>(&contract).expect("enum should lower");
        let enumeration = c_style_enum(&bindings);

        assert_eq!(enumeration.repr(), IntegerRepr::I32);
        assert_eq!(
            enumeration
                .variants()
                .iter()
                .map(|variant| variant.discriminant().get())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn advances_c_style_enum_discriminants_from_explicit_values() {
        let mut contract = package();
        contract.enums.push(enumeration(
            "demo::Status",
            "status",
            vec![
                unit_variant_with_discriminant("created", 10),
                VariantDef::unit(name("running")),
                unit_variant_with_discriminant("stopped", 8),
                VariantDef::unit(name("finished")),
            ],
        ));

        let bindings = lower::<Native>(&contract).expect("enum should lower");
        let enumeration = c_style_enum(&bindings);

        assert_eq!(
            enumeration
                .variants()
                .iter()
                .map(|variant| variant.discriminant().get())
                .collect::<Vec<_>>(),
            vec![10, 11, 8, 9]
        );
    }

    #[test]
    fn classifies_c_style_enum_with_integer_repr() {
        let mut contract = package();
        let mut enumeration = enumeration(
            "demo::Code",
            "code",
            vec![
                VariantDef::unit(name("ok")),
                VariantDef::unit(name("failed")),
            ],
        );
        enumeration.repr = ReprAttr::new(vec![ReprItem::Primitive(Primitive::U8)]);
        contract.enums.push(enumeration);

        let bindings = lower::<Native>(&contract).expect("enum should lower");
        let enumeration = c_style_enum(&bindings);

        assert_eq!(enumeration.repr(), IntegerRepr::U8);
    }

    #[test]
    fn classifies_fieldless_repr_c_enum_without_integer_repr_as_data() {
        let mut contract = package();
        let mut enumeration = enumeration(
            "demo::Direction",
            "direction",
            vec![
                VariantDef::unit(name("north")),
                VariantDef::unit(name("south")),
            ],
        );
        enumeration.repr = ReprAttr::new(vec![ReprItem::C]);
        contract.enums.push(enumeration);

        let bindings = lower::<Native>(&contract).expect("enum should lower");

        data_enum(&bindings);
    }

    #[test]
    fn classifies_payload_enum_as_data() {
        let mut contract = package();
        let mut event = enumeration("demo::Event", "event", Vec::new());
        event.variants = vec![
            VariantDef::unit(name("none")),
            VariantDef {
                name: name("message"),
                discriminant: None,
                payload: VariantPayload::Tuple(vec![TypeExpr::String]),
                doc: None,
                user_attrs: Vec::new(),
                source: boltffi_ast::Source::exported(),
                source_span: None,
            },
        ];
        contract.enums.push(event);

        let bindings = lower::<Native>(&contract).expect("enum should lower");
        let enumeration = data_enum(&bindings);

        assert_eq!(
            enumeration
                .variants()
                .iter()
                .map(|variant| variant.tag().get())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn data_enum_tags_ignore_source_discriminants() {
        let mut contract = package();
        let mut event = enumeration("demo::Event", "event", Vec::new());
        event.variants = vec![
            unit_variant_with_discriminant("none", 10),
            VariantDef {
                name: name("message"),
                discriminant: Some(20),
                payload: VariantPayload::Tuple(vec![TypeExpr::String]),
                doc: None,
                user_attrs: Vec::new(),
                source: boltffi_ast::Source::exported(),
                source_span: None,
            },
        ];
        contract.enums.push(event);

        let bindings = lower::<Native>(&contract).expect("enum should lower");
        let enumeration = data_enum(&bindings);

        assert_eq!(
            enumeration
                .variants()
                .iter()
                .map(|variant| variant.tag().get())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }
}
