//! Unit tests for schema converter

#[cfg(test)]
mod schema_converter_tests {
    use super::super::*;
    use arrow::datatypes::{DataType, Field, Schema as ArrowSchema, TimeUnit};
    
    #[test]
    fn test_all_primitive_types() {
        let arrow_schema = ArrowSchema::new(vec![
            Field::new("bool_col", DataType::Boolean, false),
            Field::new("int8_col", DataType::Int8, true),
            Field::new("int16_col", DataType::Int16, false),
            Field::new("int32_col", DataType::Int32, true),
            Field::new("int64_col", DataType::Int64, false),
            Field::new("uint8_col", DataType::UInt8, true),
            Field::new("uint16_col", DataType::UInt16, false),
            Field::new("uint32_col", DataType::UInt32, true),
            Field::new("uint64_col", DataType::UInt64, false),
            Field::new("float32_col", DataType::Float32, true),
            Field::new("float64_col", DataType::Float64, false),
            Field::new("string_col", DataType::Utf8, true),
            Field::new("large_string_col", DataType::LargeUtf8, false),
            Field::new("binary_col", DataType::Binary, true),
            Field::new("large_binary_col", DataType::LargeBinary, false),
            Field::new("fixed_binary_col", DataType::FixedSizeBinary(16), true),
            Field::new("date32_col", DataType::Date32, false),
            Field::new("date64_col", DataType::Date64, true),
            Field::new("time32_sec", DataType::Time32(TimeUnit::Second), false),
            Field::new("time64_micro", DataType::Time64(TimeUnit::Microsecond), true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        assert_eq!(fields.len(), 20);
        
        // Check a few conversions
        assert_eq!(fields[0].name, "bool_col");
        assert!(!fields[0].required); // not nullable in arrow = required in iceberg
        
        assert_eq!(fields[1].name, "int8_col");
        assert!(fields[1].required); // nullable in arrow = optional in iceberg
        
        // Check that unsigned types are converted to signed
        match &fields[5].field_type {
            Type::Primitive(PrimitiveType::Long) => {}, // UInt8 -> Long
            _ => panic!("Expected Long type for UInt8"),
        }
    }
    
    #[test]
    fn test_nested_types() {
        let inner_struct = DataType::Struct(vec![
            Field::new("x", DataType::Float32, false),
            Field::new("y", DataType::Float32, false),
        ].into());
        
        let arrow_schema = ArrowSchema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("coordinates", inner_struct, true),
            Field::new("tags", DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))), true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        assert_eq!(fields.len(), 3);
        
        // Check struct type
        match &fields[1].field_type {
            Type::Struct(struct_type) => {
                assert_eq!(struct_type.fields().len(), 2);
                assert_eq!(struct_type.fields()[0].name, "x");
                assert_eq!(struct_type.fields()[1].name, "y");
            }
            _ => panic!("Expected struct type"),
        }
        
        // Check list type
        match &fields[2].field_type {
            Type::List(_) => {},
            _ => panic!("Expected list type"),
        }
    }
    
    #[test]
    fn test_tpch_lineitem_schema() {
        // Simulate TPC-H lineitem table schema
        let arrow_schema = ArrowSchema::new(vec![
            Field::new("l_orderkey", DataType::Int64, false),
            Field::new("l_partkey", DataType::Int64, false),
            Field::new("l_suppkey", DataType::Int64, false),
            Field::new("l_linenumber", DataType::Int32, false),
            Field::new("l_quantity", DataType::Decimal128(15, 2), false),
            Field::new("l_extendedprice", DataType::Decimal128(15, 2), false),
            Field::new("l_discount", DataType::Decimal128(15, 2), false),
            Field::new("l_tax", DataType::Decimal128(15, 2), false),
            Field::new("l_returnflag", DataType::Utf8, false),
            Field::new("l_linestatus", DataType::Utf8, false),
            Field::new("l_shipdate", DataType::Date32, false),
            Field::new("l_commitdate", DataType::Date32, false),
            Field::new("l_receiptdate", DataType::Date32, false),
            Field::new("l_shipinstruct", DataType::Utf8, false),
            Field::new("l_shipmode", DataType::Utf8, false),
            Field::new("l_comment", DataType::Utf8, false),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        assert_eq!(fields.len(), 16);
        
        // Verify decimal conversions
        for i in 4..=7 {
            match &fields[i].field_type {
                Type::Primitive(PrimitiveType::Decimal { precision, scale }) => {
                    assert_eq!(*precision, 15);
                    assert_eq!(*scale, 2);
                }
                _ => panic!("Expected decimal type for field {}", fields[i].name),
            }
        }
        
        // Verify date conversions
        for i in 10..=12 {
            match &fields[i].field_type {
                Type::Primitive(PrimitiveType::Date) => {},
                _ => panic!("Expected date type for field {}", fields[i].name),
            }
        }
    }
    
    #[test]
    fn test_unsupported_types() {
        let arrow_schema = ArrowSchema::new(vec![
            Field::new("duration", DataType::Duration(TimeUnit::Second), false),
        ]);
        
        let result = SchemaConverter::arrow_to_iceberg(&arrow_schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported Arrow type"));
    }
    
    #[test]
    fn test_field_nullability() {
        let arrow_schema = ArrowSchema::new(vec![
            Field::new("required_field", DataType::Int64, false),
            Field::new("optional_field", DataType::Int64, true),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        // In Iceberg, required=false means optional
        assert!(!fields[0].required); // Arrow not nullable -> Iceberg required
        assert!(fields[1].required);  // Arrow nullable -> Iceberg optional
    }
    
    #[test]
    fn test_decimal_precision_capping() {
        let arrow_schema = ArrowSchema::new(vec![
            Field::new("huge_decimal", DataType::Decimal256(50, 10), false),
        ]);
        
        let iceberg_schema = SchemaConverter::arrow_to_iceberg(&arrow_schema).unwrap();
        let fields = iceberg_schema.as_struct().fields();
        
        match &fields[0].field_type {
            Type::Primitive(PrimitiveType::Decimal { precision, scale }) => {
                assert_eq!(*precision, 38); // Capped from 50
                assert_eq!(*scale, 10);
            }
            _ => panic!("Expected decimal type"),
        }
    }
}