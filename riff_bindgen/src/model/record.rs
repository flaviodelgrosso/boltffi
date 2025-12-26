use serde::{Deserialize, Serialize};

use super::types::{Deprecation, Type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub name: String,
    pub fields: Vec<RecordField>,
    pub doc: Option<String>,
    pub deprecated: Option<Deprecation>,
}

impl Record {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            doc: None,
            deprecated: None,
        }
    }

    pub fn with_field(mut self, field: RecordField) -> Self {
        self.fields.push(field);
        self
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn with_deprecated(mut self, deprecation: Deprecation) -> Self {
        self.deprecated = Some(deprecation);
        self
    }

    pub fn is_deprecated(&self) -> bool {
        self.deprecated.is_some()
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn struct_size(&self) -> usize {
        let (size, max_align) = self.fields.iter().fold((0usize, 1usize), |(offset, max_align), field| {
            let (field_size, field_align) = field.field_type.c_layout();
            let aligned_offset = (offset + field_align - 1) & !(field_align - 1);
            (aligned_offset + field_size, max_align.max(field_align))
        });
        (size + max_align - 1) & !(max_align - 1)
    }

    pub fn field_offsets(&self) -> Vec<usize> {
        let mut offsets = Vec::with_capacity(self.fields.len());
        let mut offset = 0usize;
        for field in &self.fields {
            let (field_size, field_align) = field.field_type.c_layout();
            offset = (offset + field_align - 1) & !(field_align - 1);
            offsets.push(offset);
            offset += field_size;
        }
        offsets
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordField {
    pub name: String,
    pub field_type: Type,
    pub doc: Option<String>,
}

impl RecordField {
    pub fn new(name: impl Into<String>, field_type: Type) -> Self {
        Self {
            name: name.into(),
            field_type,
            doc: None,
        }
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }
}
