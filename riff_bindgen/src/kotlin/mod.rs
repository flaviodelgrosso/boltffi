mod names;
mod types;

pub use names::NamingConvention;
pub use types::TypeMapper;

use crate::model::{Enumeration, Module, Record};

pub struct Kotlin;

impl Kotlin {
    pub fn render_module(module: &Module) -> String {
        let mut sections = Vec::new();

        sections.push(Self::render_preamble(module));

        module
            .enums
            .iter()
            .for_each(|enumeration| sections.push(Self::render_enum(enumeration)));

        module
            .records
            .iter()
            .for_each(|record| sections.push(Self::render_record(record)));

        let mut output = sections
            .into_iter()
            .map(|section| section.trim().to_string())
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        output.push('\n');
        output
    }

    fn render_preamble(module: &Module) -> String {
        let package_name = NamingConvention::class_name(&module.name).to_lowercase();
        format!(
            "package {}\n\nimport java.nio.ByteBuffer\nimport java.nio.ByteOrder",
            package_name
        )
    }

    fn render_enum(enumeration: &Enumeration) -> String {
        if enumeration.is_c_style() {
            Self::render_c_style_enum(enumeration)
        } else {
            Self::render_sealed_class_enum(enumeration)
        }
    }

    fn render_c_style_enum(enumeration: &Enumeration) -> String {
        let class_name = NamingConvention::class_name(&enumeration.name);

        let entries: Vec<String> = enumeration
            .variants
            .iter()
            .map(|variant| {
                let entry_name = NamingConvention::enum_entry_name(&variant.name);
                let value = variant.discriminant.unwrap_or(0);
                format!("    {}({})", entry_name, value)
            })
            .collect();

        format!(
            "enum class {}(val value: Int) {{\n{};\n\n    companion object {{\n        fun fromValue(value: Int): {} = entries.first {{ it.value == value }}\n    }}\n}}",
            class_name,
            entries.join(",\n"),
            class_name
        )
    }

    fn render_sealed_class_enum(enumeration: &Enumeration) -> String {
        let class_name = NamingConvention::class_name(&enumeration.name);

        let variants: Vec<String> = enumeration
            .variants
            .iter()
            .map(|variant| {
                let variant_name = NamingConvention::class_name(&variant.name);
                if variant.fields.is_empty() {
                    format!("    data object {} : {}()", variant_name, class_name)
                } else {
                    let fields: Vec<String> = variant
                        .fields
                        .iter()
                        .map(|field| {
                            let field_name = NamingConvention::property_name(&field.name);
                            let field_type = TypeMapper::map_type(&field.field_type);
                            format!("val {}: {}", field_name, field_type)
                        })
                        .collect();
                    format!(
                        "    data class {}({}) : {}()",
                        variant_name,
                        fields.join(", "),
                        class_name
                    )
                }
            })
            .collect();

        format!(
            "sealed class {} {{\n{}\n}}",
            class_name,
            variants.join("\n")
        )
    }

    fn render_record(record: &Record) -> String {
        let class_name = NamingConvention::class_name(&record.name);

        let fields: Vec<String> = record
            .fields
            .iter()
            .map(|field| {
                let field_name = NamingConvention::property_name(&field.name);
                let field_type = TypeMapper::map_type(&field.field_type);
                format!("    val {}: {}", field_name, field_type)
            })
            .collect();

        format!("data class {}(\n{}\n)", class_name, fields.join(",\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Primitive, RecordField, Type, Variant};

    #[test]
    fn test_kotlin_type_mapping() {
        assert_eq!(
            TypeMapper::map_type(&Type::Primitive(Primitive::I32)),
            "Int"
        );
        assert_eq!(
            TypeMapper::map_type(&Type::Primitive(Primitive::I64)),
            "Long"
        );
        assert_eq!(
            TypeMapper::map_type(&Type::Primitive(Primitive::Bool)),
            "Boolean"
        );
        assert_eq!(TypeMapper::map_type(&Type::String), "String");
        assert_eq!(TypeMapper::map_type(&Type::Bytes), "ByteArray");
        assert_eq!(
            TypeMapper::map_type(&Type::Vec(Box::new(Type::Primitive(Primitive::F64)))),
            "List<Double>"
        );
    }

    #[test]
    fn test_kotlin_naming() {
        assert_eq!(NamingConvention::class_name("sensor_manager"), "SensorManager");
        assert_eq!(NamingConvention::method_name("get_reading"), "getReading");
        assert_eq!(NamingConvention::enum_entry_name("active"), "ACTIVE");
    }

    #[test]
    fn test_kotlin_keyword_escaping() {
        assert_eq!(NamingConvention::escape_keyword("value"), "`value`");
        assert_eq!(NamingConvention::escape_keyword("count"), "count");
    }

    #[test]
    fn test_render_c_style_enum() {
        let status = Enumeration::new("sensor_status")
            .with_variant(Variant::new("idle").with_discriminant(0))
            .with_variant(Variant::new("active").with_discriminant(1))
            .with_variant(Variant::new("error").with_discriminant(2));

        let output = Kotlin::render_enum(&status);
        assert!(output.contains("enum class SensorStatus"));
        assert!(output.contains("IDLE(0)"));
        assert!(output.contains("ACTIVE(1)"));
        assert!(output.contains("fromValue(value: Int)"));
    }

    #[test]
    fn test_render_sealed_class_enum() {
        let result_enum = Enumeration::new("api_result")
            .with_variant(Variant::new("success"))
            .with_variant(
                Variant::new("error")
                    .with_field(RecordField::new("code", Type::Primitive(Primitive::I32))),
            );

        let output = Kotlin::render_enum(&result_enum);
        assert!(output.contains("sealed class ApiResult"));
        assert!(output.contains("data object Success"));
        assert!(output.contains("data class Error"));
        assert!(output.contains("val code: Int"));
    }

    #[test]
    fn test_render_record() {
        let reading = Record::new("sensor_reading")
            .with_field(RecordField::new(
                "timestamp",
                Type::Primitive(Primitive::U64),
            ))
            .with_field(RecordField::new("value", Type::Primitive(Primitive::F64)));

        let output = Kotlin::render_record(&reading);
        assert!(output.contains("data class SensorReading"));
        assert!(output.contains("val timestamp: ULong"));
        assert!(output.contains("val value: Double"));
    }
}
