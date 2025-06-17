//! Schema converter between Arrow and Iceberg formats

use crate::error::{Error, Result};
use arrow::datatypes::{DataType, Schema as ArrowSchema};
use iceberg::spec::{NestedField, PrimitiveType, Schema, Type};

/// Converts Arrow schemas to Iceberg schemas
pub struct SchemaConverter;

/// Generates unique field IDs for Iceberg schema
struct FieldIdGenerator {
    next_id: i32,
}

impl FieldIdGenerator {
    fn new() -> Self {
        Self { next_id: 1 }
    }
    
    fn next_id(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl SchemaConverter {
    /// Convert an Arrow schema to an Iceberg schema
    pub fn arrow_to_iceberg(arrow_schema: &ArrowSchema) -> Result<Schema> {
        let mut field_id_generator = FieldIdGenerator::new();
        let mut fields = Vec::new();
        
        for field in arrow_schema.fields().iter() {
            let iceberg_type = Self::convert_type(field.data_type(), &mut field_id_generator)?;
            let nested_field = if field.is_nullable() {
                NestedField::optional(field_id_generator.next_id(), field.name(), iceberg_type)
            } else {
                NestedField::required(field_id_generator.next_id(), field.name(), iceberg_type)
            };
            fields.push(nested_field.into());
        }
        
        Schema::builder()
            .with_schema_id(1)
            .with_fields(fields)
            .build()
            .map_err(|e| Error::InvalidOperation(format!("Failed to build Iceberg schema: {}", e)))
    }
    
    /// Convert Arrow data type to Iceberg type
    fn convert_type(arrow_type: &DataType, id_gen: &mut FieldIdGenerator) -> Result<Type> {
        match arrow_type {
            DataType::Boolean => Ok(Type::Primitive(PrimitiveType::Boolean)),
            DataType::Int8 | DataType::Int16 | DataType::Int32 => {
                Ok(Type::Primitive(PrimitiveType::Int))
            }
            DataType::Int64 => Ok(Type::Primitive(PrimitiveType::Long)),
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 => {
                // Iceberg doesn't have unsigned types, promote to signed
                Ok(Type::Primitive(PrimitiveType::Long))
            }
            DataType::UInt64 => Ok(Type::Primitive(PrimitiveType::Long)),
            DataType::Float32 => Ok(Type::Primitive(PrimitiveType::Float)),
            DataType::Float64 => Ok(Type::Primitive(PrimitiveType::Double)),
            DataType::Utf8 | DataType::LargeUtf8 => Ok(Type::Primitive(PrimitiveType::String)),
            DataType::Binary | DataType::LargeBinary => Ok(Type::Primitive(PrimitiveType::Binary)),
            DataType::FixedSizeBinary(len) => Ok(Type::Primitive(PrimitiveType::Fixed(*len as u64))),
            DataType::Date32 | DataType::Date64 => Ok(Type::Primitive(PrimitiveType::Date)),
            DataType::Time32(_) | DataType::Time64(_) => Ok(Type::Primitive(PrimitiveType::Time)),
            DataType::Timestamp(_unit, tz) => {
                match tz {
                    Some(_) => Ok(Type::Primitive(PrimitiveType::Timestamptz)),
                    None => Ok(Type::Primitive(PrimitiveType::Timestamp)),
                }
            }
            DataType::Decimal128(precision, scale) => {
                Ok(Type::Primitive(PrimitiveType::Decimal { 
                    precision: *precision as u32, 
                    scale: *scale as u32 
                }))
            }
            DataType::Decimal256(precision, scale) => {
                // Iceberg only supports up to 38 precision
                let precision = (*precision as u32).min(38);
                Ok(Type::Primitive(PrimitiveType::Decimal { 
                    precision, 
                    scale: *scale as u32 
                }))
            }
            DataType::List(field) | DataType::LargeList(field) => {
                let inner_type = Self::convert_type(field.data_type(), id_gen)?;
                Ok(Type::List(iceberg::spec::ListType {
                    element_field: std::sync::Arc::new(
                        NestedField::optional(id_gen.next_id(), "element", inner_type)
                    ),
                }))
            }
            DataType::Struct(fields) => {
                let mut struct_fields = Vec::new();
                for field in fields.iter() {
                    let iceberg_type = Self::convert_type(field.data_type(), id_gen)?;
                    let nested_field = if field.is_nullable() {
                        NestedField::optional(id_gen.next_id(), field.name(), iceberg_type)
                    } else {
                        NestedField::required(id_gen.next_id(), field.name(), iceberg_type)
                    };
                    struct_fields.push(nested_field.into());
                }
                Ok(Type::Struct(iceberg::spec::StructType::new(struct_fields)))
            }
            DataType::Map(key_field, _sorted) => {
                let key_type = Self::convert_type(&key_field.data_type(), id_gen)?;
                let value_field = key_field; // In Arrow, Map type reuses the field for both key and value info
                let value_type = Self::convert_type(&value_field.data_type(), id_gen)?;
                
                Ok(Type::Map(iceberg::spec::MapType {
                    key_field: std::sync::Arc::new(
                        NestedField::required(id_gen.next_id(), "key", key_type)
                    ),
                    value_field: std::sync::Arc::new(
                        NestedField::optional(id_gen.next_id(), "value", value_type)
                    ),
                }))
            }
            _ => Err(Error::InvalidOperation(
                format!("Unsupported Arrow type for Iceberg conversion: {:?}", arrow_type)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{Field as ArrowField, Schema as ArrowSchema, TimeUnit};
    
    #[test]
    fn test_simple_schema_conversion() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("id", DataType::Int64, false),
            ArrowField::new("name", DataType::Utf8, true),
            ArrowField::new("amount", DataType::Float64, false),
            ArrowField::new("active", DataType::Boolean, true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        
        // Verify the schema was converted correctly
        let fields = iceberg_schema.as_struct().fields();
        assert_eq!(fields.len(), 4);
        
        assert_eq!(fields[0].name, "id");
        assert!(fields[0].required); // not nullable in Arrow = required in Iceberg
        
        assert_eq!(fields[1].name, "name");
        assert!(!fields[1].required); // nullable in Arrow = optional in Iceberg
        
        assert_eq!(fields[2].name, "amount");
        assert!(fields[2].required); // not nullable in Arrow = required in Iceberg
        
        assert_eq!(fields[3].name, "active");
        assert!(!fields[3].required); // nullable in Arrow = optional in Iceberg
    }
    
    #[test]
    fn test_decimal_conversion() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("price", DataType::Decimal128(10, 2), false),
            ArrowField::new("large_value", DataType::Decimal256(50, 4), true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        assert_eq!(fields.len(), 2);
        
        // Check decimal precision/scale
        match &*fields[0].field_type {
            Type::Primitive(PrimitiveType::Decimal { precision, scale }) => {
                assert_eq!(*precision, 10);
                assert_eq!(*scale, 2);
            }
            _ => panic!("Expected decimal type"),
        }
        
        // Check that Decimal256 is capped at 38 precision
        match &*fields[1].field_type {
            Type::Primitive(PrimitiveType::Decimal { precision, scale }) => {
                assert_eq!(*precision, 38); // Capped from 50
                assert_eq!(*scale, 4);
            }
            _ => panic!("Expected decimal type"),
        }
    }
    
    #[test]
    fn test_timestamp_conversion() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("created_at", DataType::Timestamp(TimeUnit::Microsecond, None), false),
            ArrowField::new("updated_at", DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())), true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        // Timestamp without timezone
        match &*fields[0].field_type {
            Type::Primitive(PrimitiveType::Timestamp) => {}
            _ => panic!("Expected timestamp type"),
        }
        
        // Timestamp with timezone
        match &*fields[1].field_type {
            Type::Primitive(PrimitiveType::Timestamptz) => {}
            _ => panic!("Expected timestamptz type"),
        }
    }
    
    #[test]
    fn test_all_primitive_types() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("bool_col", DataType::Boolean, false),
            ArrowField::new("int8_col", DataType::Int8, true),
            ArrowField::new("int16_col", DataType::Int16, false),
            ArrowField::new("int32_col", DataType::Int32, true),
            ArrowField::new("int64_col", DataType::Int64, false),
            ArrowField::new("uint8_col", DataType::UInt8, true),
            ArrowField::new("uint16_col", DataType::UInt16, false),
            ArrowField::new("uint32_col", DataType::UInt32, true),
            ArrowField::new("uint64_col", DataType::UInt64, false),
            ArrowField::new("float32_col", DataType::Float32, true),
            ArrowField::new("float64_col", DataType::Float64, false),
            ArrowField::new("string_col", DataType::Utf8, true),
            ArrowField::new("large_string_col", DataType::LargeUtf8, false),
            ArrowField::new("binary_col", DataType::Binary, true),
            ArrowField::new("large_binary_col", DataType::LargeBinary, false),
            ArrowField::new("fixed_binary_col", DataType::FixedSizeBinary(16), true),
            ArrowField::new("date32_col", DataType::Date32, false),
            ArrowField::new("date64_col", DataType::Date64, true),
            ArrowField::new("time32_sec", DataType::Time32(TimeUnit::Second), false),
            ArrowField::new("time64_micro", DataType::Time64(TimeUnit::Microsecond), true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        assert_eq!(fields.len(), 20);
        
        // Check a few conversions
        assert_eq!(fields[0].name, "bool_col");
        assert!(fields[0].required); // not nullable in arrow = required in iceberg
        
        assert_eq!(fields[1].name, "int8_col");
        assert!(!fields[1].required); // nullable in arrow = optional in iceberg
        
        // Check that unsigned types are converted to signed
        match &*fields[5].field_type {
            Type::Primitive(PrimitiveType::Long) => {}, // UInt8 -> Long
            _ => panic!("Expected Long type for UInt8"),
        }
    }
    
    #[test]
    fn test_nested_types() {
        use std::sync::Arc;
        
        let inner_struct = DataType::Struct(vec![
            ArrowField::new("x", DataType::Float32, false),
            ArrowField::new("y", DataType::Float32, false),
        ].into());
        
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("id", DataType::Int64, false),
            ArrowField::new("coordinates", inner_struct, true),
            ArrowField::new("tags", DataType::List(Arc::new(ArrowField::new("item", DataType::Utf8, true))), true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        assert_eq!(fields.len(), 3);
        
        // Check struct type
        match &*fields[1].field_type {
            Type::Struct(struct_type) => {
                assert_eq!(struct_type.fields().len(), 2);
                assert_eq!(struct_type.fields()[0].name, "x");
                assert_eq!(struct_type.fields()[1].name, "y");
            }
            _ => panic!("Expected struct type"),
        }
        
        // Check list type
        match &*fields[2].field_type {
            Type::List(_) => {},
            _ => panic!("Expected list type"),
        }
    }
    
    #[test]
    fn test_tpch_lineitem_schema() {
        // Simulate TPC-H lineitem table schema
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("l_orderkey", DataType::Int64, false),
            ArrowField::new("l_partkey", DataType::Int64, false),
            ArrowField::new("l_suppkey", DataType::Int64, false),
            ArrowField::new("l_linenumber", DataType::Int32, false),
            ArrowField::new("l_quantity", DataType::Decimal128(15, 2), false),
            ArrowField::new("l_extendedprice", DataType::Decimal128(15, 2), false),
            ArrowField::new("l_discount", DataType::Decimal128(15, 2), false),
            ArrowField::new("l_tax", DataType::Decimal128(15, 2), false),
            ArrowField::new("l_returnflag", DataType::Utf8, false),
            ArrowField::new("l_linestatus", DataType::Utf8, false),
            ArrowField::new("l_shipdate", DataType::Date32, false),
            ArrowField::new("l_commitdate", DataType::Date32, false),
            ArrowField::new("l_receiptdate", DataType::Date32, false),
            ArrowField::new("l_shipinstruct", DataType::Utf8, false),
            ArrowField::new("l_shipmode", DataType::Utf8, false),
            ArrowField::new("l_comment", DataType::Utf8, false),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        assert_eq!(fields.len(), 16);
        
        // Verify decimal conversions
        for i in 4..=7 {
            match &*fields[i].field_type {
                Type::Primitive(PrimitiveType::Decimal { precision, scale }) => {
                    assert_eq!(*precision, 15);
                    assert_eq!(*scale, 2);
                }
                _ => panic!("Expected decimal type for field {}", fields[i].name),
            }
        }
        
        // Verify date conversions
        for i in 10..=12 {
            match &*fields[i].field_type {
                Type::Primitive(PrimitiveType::Date) => {},
                _ => panic!("Expected date type for field {}", fields[i].name),
            }
        }
    }
    
    #[test]
    fn test_unsupported_types() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("duration", DataType::Duration(TimeUnit::Second), false),
        ]);
        
        let result = SchemaConverter::arrow_to_iceberg(&arrow_schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported Arrow type"));
    }
    
    #[test]
    fn test_field_nullability() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("required_field", DataType::Int64, false),
            ArrowField::new("optional_field", DataType::Int64, true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        // In Iceberg, required=true means required, required=false means optional
        assert!(fields[0].required);  // Arrow not nullable -> Iceberg required
        assert!(!fields[1].required); // Arrow nullable -> Iceberg optional
    }
    
    #[test]
    fn test_decimal_precision_capping() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("huge_decimal", DataType::Decimal256(50, 10), false),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        match &*fields[0].field_type {
            Type::Primitive(PrimitiveType::Decimal { precision, scale }) => {
                assert_eq!(*precision, 38); // Capped from 50
                assert_eq!(*scale, 10);
            }
            _ => panic!("Expected decimal type"),
        }
    }
}
