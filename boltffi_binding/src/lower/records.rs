use boltffi_ast::{RecordDef as SourceRecord, TypeExpr};

use crate::{
    CanonicalName, DirectFieldDecl, DirectRecordDecl, EncodedFieldDecl, EncodedRecordDecl,
    FieldKey, RecordDecl, Surface, ValueRef,
};

use super::{
    LowerError, codecs, ids::DeclarationIds, index::Index, layout, metadata, primitive, types,
};

/// Lowers every record in the source contract.
pub(super) fn lower<S: Surface>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
) -> Result<Vec<RecordDecl<S>>, LowerError> {
    idx.records()
        .iter()
        .map(|record| lower_one(idx, ids, record))
        .collect()
}

/// Reports whether a source record crosses by direct memory.
///
/// Exposed to the codec lane so a nested `TypeExpr::Record(id)` can
/// pick `DirectRecord` vs `EncodedRecord` from the same predicate the
/// record's own declaration uses.
pub(super) fn is_direct(record: &SourceRecord) -> bool {
    primitive::has_effective_repr_c(&record.repr)
        && !record.fields.is_empty()
        && record
            .fields
            .iter()
            .all(|field| primitive::fixed_primitive(&field.type_expr).is_some())
}

fn lower_one<S: Surface>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    record: &SourceRecord,
) -> Result<RecordDecl<S>, LowerError> {
    if is_direct(record) {
        lower_direct(ids, record).map(RecordDecl::Direct)
    } else {
        lower_encoded(idx, ids, record).map(RecordDecl::Encoded)
    }
}

fn lower_direct<S: Surface>(
    ids: &DeclarationIds,
    record: &SourceRecord,
) -> Result<DirectRecordDecl<S>, LowerError> {
    let fields = record
        .fields
        .iter()
        .map(|field| {
            Ok(DirectFieldDecl::new(
                FieldKey::from(field),
                types::lower(ids, &field.type_expr)?,
                metadata::element_meta(field.doc.as_ref(), None, field.default.as_ref())?,
            ))
        })
        .collect::<Result<Vec<_>, LowerError>>()?;

    Ok(DirectRecordDecl::new(
        ids.record(&record.id)?,
        CanonicalName::from(&record.name),
        metadata::decl_meta(record.doc.as_ref(), record.deprecated.as_ref()),
        fields,
        Vec::new(),
        Vec::new(),
        layout::compute(record)?,
    ))
}

fn lower_encoded<S: Surface>(
    idx: &Index<'_>,
    ids: &DeclarationIds,
    record: &SourceRecord,
) -> Result<EncodedRecordDecl<S>, LowerError> {
    let fields = record
        .fields
        .iter()
        .map(|field| {
            let key = FieldKey::from(field);
            let value = ValueRef::self_value().field(key.clone());
            let ty = types::lower(ids, &field.type_expr)?;
            let codec = codecs::plan(idx, ids, &field.type_expr, value)?;
            Ok(EncodedFieldDecl::new(
                key,
                ty,
                codec,
                metadata::element_meta(field.doc.as_ref(), None, field.default.as_ref())?,
            ))
        })
        .collect::<Result<Vec<_>, LowerError>>()?;

    Ok(EncodedRecordDecl::new(
        ids.record(&record.id)?,
        CanonicalName::from(&record.name),
        metadata::decl_meta(record.doc.as_ref(), record.deprecated.as_ref()),
        fields,
        Vec::new(),
        Vec::new(),
        codecs::plan(
            idx,
            ids,
            &TypeExpr::Record(record.id.clone()),
            ValueRef::self_value(),
        )?,
    ))
}

#[cfg(test)]
mod tests {
    use boltffi_ast::{
        CanonicalName as SourceName, FieldDef, PackageInfo as SourcePackage, Primitive, RecordDef,
        ReprAttr, ReprItem, SourceContract, TypeExpr,
    };

    use crate::{
        ByteSize, CanonicalName, CodecNode, Decl, FieldKey, IntrinsicOp, Native, OpNode,
        RecordDecl, lower,
    };

    fn package() -> SourceContract {
        SourceContract::new(SourcePackage::new("demo", Some("0.1.0".to_owned())))
    }

    fn name(part: &str) -> SourceName {
        SourceName::single(part)
    }

    fn record(id: &str, record_name: &str, fields: Vec<FieldDef>) -> RecordDef {
        let mut record = RecordDef::new(id.into(), name(record_name));
        record.fields = fields;
        record
    }

    fn field(field_name: &str, type_expr: TypeExpr) -> FieldDef {
        FieldDef::new(name(field_name), type_expr)
    }

    fn direct_record(bindings: &crate::Bindings<Native>) -> &crate::DirectRecordDecl<Native> {
        match bindings.decls().first() {
            Some(Decl::Record(record)) => match record.as_ref() {
                RecordDecl::Direct(record) => record,
                RecordDecl::Encoded(_) => panic!("expected direct record"),
            },
            _ => panic!("expected record declaration"),
        }
    }

    fn encoded_record(bindings: &crate::Bindings<Native>) -> &crate::EncodedRecordDecl<Native> {
        match bindings.decls().first() {
            Some(Decl::Record(record)) => match record.as_ref() {
                RecordDecl::Encoded(record) => record,
                RecordDecl::Direct(_) => panic!("expected encoded record"),
            },
            _ => panic!("expected record declaration"),
        }
    }

    fn sequence_len_value(node: &CodecNode) -> &crate::ValueRef {
        match node {
            CodecNode::Sequence { len, .. } => match len.node() {
                OpNode::Intrinsic {
                    intrinsic: IntrinsicOp::SequenceLen,
                    args,
                } => match args.first() {
                    Some(OpNode::Value(value)) => value,
                    _ => panic!("expected sequence length value argument"),
                },
                _ => panic!("expected sequence length intrinsic"),
            },
            _ => panic!("expected sequence codec"),
        }
    }

    #[test]
    fn classifies_unannotated_primitive_record_as_direct() {
        let mut contract = package();
        contract.records.push(record(
            "demo::Point",
            "point",
            vec![
                field("x", TypeExpr::Primitive(Primitive::F64)),
                field("y", TypeExpr::Primitive(Primitive::F64)),
            ],
        ));

        let bindings = lower::<Native>(&contract).expect("record should lower");
        let record = direct_record(&bindings);

        assert_eq!(record.layout().size(), ByteSize::new(16));
        assert_eq!(record.layout().alignment().get(), 8);
        assert_eq!(
            record
                .layout()
                .fields()
                .iter()
                .map(|field| field.offset().get())
                .collect::<Vec<_>>(),
            vec![0, 8]
        );
    }

    #[test]
    fn lays_out_direct_record_with_padding() {
        let mut contract = package();
        contract.records.push(record(
            "demo::Header",
            "header",
            vec![
                field("tag", TypeExpr::Primitive(Primitive::U8)),
                field("count", TypeExpr::Primitive(Primitive::U32)),
            ],
        ));

        let bindings = lower::<Native>(&contract).expect("record should lower");
        let record = direct_record(&bindings);

        assert_eq!(record.layout().size(), ByteSize::new(8));
        assert_eq!(record.layout().alignment().get(), 4);
        assert_eq!(
            record
                .layout()
                .fields()
                .iter()
                .map(|field| field.offset().get())
                .collect::<Vec<_>>(),
            vec![0, 4]
        );
    }

    #[test]
    fn classifies_empty_record_as_encoded() {
        let mut contract = package();
        contract
            .records
            .push(record("demo::Empty", "empty", Vec::new()));

        let bindings = lower::<Native>(&contract).expect("record should lower");
        let record = encoded_record(&bindings);

        assert_eq!(record.fields().len(), 0);
    }

    #[test]
    fn classifies_platform_sized_field_as_encoded() {
        let mut contract = package();
        contract.records.push(record(
            "demo::Index",
            "index",
            vec![field("raw", TypeExpr::Primitive(Primitive::USize))],
        ));

        let bindings = lower::<Native>(&contract).expect("record should lower");

        encoded_record(&bindings);
    }

    #[test]
    fn classifies_non_primitive_field_as_encoded() {
        let mut contract = package();
        contract.records.push(record(
            "demo::User",
            "user",
            vec![field("name", TypeExpr::String)],
        ));

        let bindings = lower::<Native>(&contract).expect("record should lower");

        encoded_record(&bindings);
    }

    #[test]
    fn classifies_transparent_record_as_encoded() {
        let mut contract = package();
        let mut record = record(
            "demo::UserId",
            "user_id",
            vec![field("raw", TypeExpr::Primitive(Primitive::U64))],
        );
        record.repr = ReprAttr::new(vec![ReprItem::Transparent]);
        contract.records.push(record);

        let bindings = lower::<Native>(&contract).expect("record should lower");

        encoded_record(&bindings);
    }

    #[test]
    fn sequence_field_codec_counts_the_field_value() {
        let mut contract = package();
        contract.records.push(record(
            "demo::Names",
            "names",
            vec![field("items", TypeExpr::vec(TypeExpr::String))],
        ));

        let bindings = lower::<Native>(&contract).expect("record should lower");
        let record = encoded_record(&bindings);
        let value = sequence_len_value(record.fields()[0].write().root());

        assert_eq!(
            value.path(),
            &[FieldKey::Named(CanonicalName::single("items"))]
        );
    }
}
