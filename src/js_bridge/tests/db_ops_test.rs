#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, Number};
    use crate::js_bridge::ops::db_ops::DynamicRow;

    #[test]
    fn test_dynamic_row_serialization() {
        let mut map = Map::new();
        map.insert("name".to_string(), Value::String("test".to_string()));
        map.insert(
            "age".to_string(),
            Value::Number(serde_json::Number::from(30)),
        );

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["name"], "test");
        assert_eq!(json["age"], 30);
    }

    #[test]
    fn test_dynamic_row_with_integer() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(42)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 42);
    }

    #[test]
    fn test_dynamic_row_with_bigint() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(9223372036854775807i64)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 9223372036854775807i64);
    }

    #[test]
    fn test_dynamic_row_with_negative_integer() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(-42)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], -42);
    }

    #[test]
    fn test_dynamic_row_with_double() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(3.14159).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 3.14159);
    }

    #[test]
    fn test_dynamic_row_with_negative_double() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(-2.5).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], -2.5);
    }

    #[test]
    fn test_dynamic_row_with_zero() {
        let mut map = Map::new();
        map.insert("int_zero".to_string(), Value::Number(Number::from(0)));
        map.insert("float_zero".to_string(), Value::Number(Number::from_f64(0.0).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["int_zero"], 0);
        assert_eq!(json["float_zero"], 0.0);
    }

    #[test]
    fn test_dynamic_row_with_boolean_true() {
        let mut map = Map::new();
        map.insert("active".to_string(), Value::Bool(true));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["active"], true);
    }

    #[test]
    fn test_dynamic_row_with_boolean_false() {
        let mut map = Map::new();
        map.insert("active".to_string(), Value::Bool(false));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["active"], false);
    }

    #[test]
    fn test_dynamic_row_with_null() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Null);

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], Value::Null);
    }

    #[test]
    fn test_dynamic_row_with_empty_string() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String("".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], "");
    }

    #[test]
    fn test_dynamic_row_with_unicode_string() {
        let mut map = Map::new();
        map.insert("chinese".to_string(), Value::String("你好世界".to_string()));
        map.insert("emoji".to_string(), Value::String("🎉🚀".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["chinese"], "你好世界");
        assert_eq!(json["emoji"], "🎉🚀");
    }

    #[test]
    fn test_dynamic_row_with_special_characters() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String("Hello\nWorld\t!\"\\'".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], "Hello\nWorld\t!\"\\'");
    }

    #[test]
    fn test_dynamic_row_with_multiple_fields() {
        let mut map = Map::new();
        map.insert("id".to_string(), Value::Number(Number::from(1)));
        map.insert("name".to_string(), Value::String("Alice".to_string()));
        map.insert("age".to_string(), Value::Number(Number::from(30)));
        map.insert("active".to_string(), Value::Bool(true));
        map.insert("score".to_string(), Value::Number(Number::from_f64(95.5).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["id"], 1);
        assert_eq!(json["name"], "Alice");
        assert_eq!(json["age"], 30);
        assert_eq!(json["active"], true);
        assert_eq!(json["score"], 95.5);
    }

    #[test]
    fn test_dynamic_row_with_very_large_number() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(2147483647i32)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 2147483647i32);
    }

    #[test]
    fn test_dynamic_row_with_very_small_number() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(-2147483648i32)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], -2147483648i32);
    }

    #[test]
    fn test_dynamic_row_with_scientific_notation() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(1.23e10).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 1.23e10);
    }

    #[test]
    fn test_dynamic_row_with_very_small_double() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(0.000001).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 0.000001);
    }

    #[test]
    fn test_dynamic_row_with_mixed_null_and_values() {
        let mut map = Map::new();
        map.insert("id".to_string(), Value::Number(Number::from(1)));
        map.insert("name".to_string(), Value::Null);
        map.insert("age".to_string(), Value::Number(Number::from(30)));
        map.insert("email".to_string(), Value::Null);

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["id"], 1);
        assert_eq!(json["name"], Value::Null);
        assert_eq!(json["age"], 30);
        assert_eq!(json["email"], Value::Null);
    }

    #[test]
    fn test_dynamic_row_with_long_string() {
        let long_string = "a".repeat(10000);
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String(long_string.clone()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], long_string);
    }

    #[test]
    fn test_dynamic_row_with_json_string() {
        let json_str = r#"{"key": "value", "number": 42}"#;
        let mut map = Map::new();
        map.insert("data".to_string(), Value::String(json_str.to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["data"], json_str);
    }

    #[test]
    fn test_dynamic_row_with_timestamp_string() {
        let timestamp = "2024-01-01 12:00:00";
        let mut map = Map::new();
        map.insert("created_at".to_string(), Value::String(timestamp.to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["created_at"], timestamp);
    }

    #[test]
    fn test_dynamic_row_with_email_string() {
        let email = "user@example.com";
        let mut map = Map::new();
        map.insert("email".to_string(), Value::String(email.to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["email"], email);
    }

    #[test]
    fn test_dynamic_row_with_url_string() {
        let url = "https://example.com/path?query=value";
        let mut map = Map::new();
        map.insert("url".to_string(), Value::String(url.to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["url"], url);
    }

    #[test]
    fn test_dynamic_row_with_number_as_string() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String("12345".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], "12345");
    }

    #[test]
    fn test_dynamic_row_with_boolean_as_string() {
        let mut map = Map::new();
        map.insert("value1".to_string(), Value::String("true".to_string()));
        map.insert("value2".to_string(), Value::String("false".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value1"], "true");
        assert_eq!(json["value2"], "false");
    }

    #[test]
    fn test_dynamic_row_with_infinity() {
        let mut map = Map::new();
        // JSON 不支持 infinity，所以会转换为 null
        let value = Number::from_f64(f64::INFINITY);
        if value.is_some() {
            map.insert("value".to_string(), Value::Number(value.unwrap()));
        }

        if !map.is_empty() {
            let row = DynamicRow(map);
            let json = serde_json::to_value(&row).unwrap();
            // Infinity 在 JSON 中通常表示为 null
            assert_eq!(json["value"], Value::Null);
        }
    }

    #[test]
    fn test_dynamic_row_with_nan() {
        let mut map = Map::new();
        // JSON 不支持 NaN，所以会转换为 null
        let value = Number::from_f64(f64::NAN);
        if value.is_some() {
            map.insert("value".to_string(), Value::Number(value.unwrap()));
        }

        if !map.is_empty() {
            let row = DynamicRow(map);
            let json = serde_json::to_value(&row).unwrap();
            // NaN 在 JSON 中通常表示为 null
            assert_eq!(json["value"], Value::Null);
        }
    }

    #[test]
    fn test_dynamic_round_trip_serialization() {
        let mut original_map = Map::new();
        original_map.insert("id".to_string(), Value::Number(Number::from(1)));
        original_map.insert("name".to_string(), Value::String("Test".to_string()));
        original_map.insert("active".to_string(), Value::Bool(true));
        original_map.insert("score".to_string(), Value::Number(Number::from_f64(95.5).unwrap()));

        let row = DynamicRow(original_map.clone());
        let json = serde_json::to_value(&row).unwrap();
        let deserialized: DynamicRow = serde_json::from_value(json).unwrap();

        assert_eq!(deserialized.0["id"], original_map["id"]);
        assert_eq!(deserialized.0["name"], original_map["name"]);
        assert_eq!(deserialized.0["active"], original_map["active"]);
        assert_eq!(deserialized.0["score"], original_map["score"]);
    }

    #[test]
    fn test_dynamic_row_empty() {
        let map = Map::new();
        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json, Value::Object(Map::new()));
    }

    #[test]
    fn test_dynamic_row_with_hundred_fields() {
        let mut map = Map::new();
        for i in 0..100 {
            map.insert(
                format!("field_{}", i),
                Value::Number(Number::from(i)),
            );
        }

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        // 验证所有字段都存在
        if let Value::Object(obj) = json {
            assert_eq!(obj.len(), 100);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_dynamic_row_with_duplicate_keys() {
        let mut map = Map::new();
        map.insert("key".to_string(), Value::Number(Number::from(1)));
        map.insert("key".to_string(), Value::Number(Number::from(2))); // 会覆盖

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        // 后面的值会覆盖前面的
        assert_eq!(json["key"], 2);
    }

    #[test]
    fn test_dynamic_row_with_numeric_string() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String("123.456".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        // 应该保持为字符串，而不是数字
        assert_eq!(json["value"], "123.456");
    }

    #[test]
    fn test_dynamic_row_with_boolean_string() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String("true".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        // 应该保持为字符串，而不是布尔值
        assert_eq!(json["value"], "true");
    }

    #[test]
    fn test_dynamic_row_with_null_string() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String("null".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        // 应该保持为字符串，而不是 null
        assert_eq!(json["value"], "null");
    }

    #[test]
    fn test_dynamic_row_with_escaped_characters() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String("Line 1\nLine 2\rTab\tQuote\"".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], "Line 1\nLine 2\rTab\tQuote\"");
    }

    #[test]
    fn test_dynamic_row_with_max_integer() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(i32::MAX)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], i32::MAX);
    }

    #[test]
    fn test_dynamic_row_with_min_integer() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(i32::MIN)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], i32::MIN);
    }

    #[test]
    fn test_dynamic_row_with_max_bigint() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(i64::MAX)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], i64::MAX);
    }

    #[test]
    fn test_dynamic_row_with_min_bigint() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from(i64::MIN)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], i64::MIN);
    }

    #[test]
    fn test_dynamic_row_with_max_double() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(f64::MAX).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], f64::MAX);
    }

    #[test]
    fn test_dynamic_row_with_min_double() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(f64::MIN).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], f64::MIN);
    }

    #[test]
    fn test_dynamic_row_with_negative_zero() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(-0.0).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], -0.0);
    }

    #[test]
    fn test_dynamic_row_with_very_long_integer_string() {
        let long_number = "1234567890123456789012345678901234567890";
        let mut map = Map::new();
        map.insert("value".to_string(), Value::String(long_number.to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], long_number);
    }

    #[test]
    fn test_dynamic_row_with_decimal_precision() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(3.141592653589793).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 3.141592653589793);
    }

    #[test]
    fn test_dynamic_row_with_exponential_notation() {
        let mut map = Map::new();
        map.insert("value".to_string(), Value::Number(Number::from_f64(1e-10).unwrap()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["value"], 1e-10);
    }

    #[test]
    fn test_dynamic_row_with_mixed_types() {
        let mut map = Map::new();
        map.insert("int_val".to_string(), Value::Number(Number::from(42)));
        map.insert("str_val".to_string(), Value::String("hello".to_string()));
        map.insert("bool_val".to_string(), Value::Bool(true));
        map.insert("null_val".to_string(), Value::Null);
        map.insert("float_val".to_string(), Value::Number(Number::from_f64(3.14).unwrap()));
        map.insert("bigint_val".to_string(), Value::Number(Number::from(9007199254740991i64)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["int_val"], 42);
        assert_eq!(json["str_val"], "hello");
        assert_eq!(json["bool_val"], true);
        assert_eq!(json["null_val"], Value::Null);
        assert_eq!(json["float_val"], 3.14);
        assert_eq!(json["bigint_val"], 9007199254740991i64);
    }

    #[test]
    fn test_dynamic_row_serialization_to_json_string() {
        let mut map = Map::new();
        map.insert("name".to_string(), Value::String("Alice".to_string()));
        map.insert("age".to_string(), Value::Number(Number::from(30)));

        let row = DynamicRow(map);
        let json_str = serde_json::to_string(&row).unwrap();

        assert!(json_str.contains("\"name\":\"Alice\""));
        assert!(json_str.contains("\"age\":30"));
    }

    #[test]
    fn test_dynamic_row_from_json_value() {
        let json_value = serde_json::json!({
            "id": 1,
            "name": "Alice",
            "active": true
        });

        let row: DynamicRow = serde_json::from_value(json_value).unwrap();

        assert_eq!(row.0["id"], Value::Number(Number::from(1)));
        assert_eq!(row.0["name"], Value::String("Alice".to_string()));
        assert_eq!(row.0["active"], Value::Bool(true));
    }

    #[test]
    fn test_dynamic_row_with_field_name_with_underscore() {
        let mut map = Map::new();
        map.insert("field_name".to_string(), Value::String("value".to_string()));
        map.insert("field_name_2".to_string(), Value::Number(Number::from(42)));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["field_name"], "value");
        assert_eq!(json["field_name_2"], 42);
    }

    #[test]
    fn test_dynamic_row_with_field_name_with_numbers() {
        let mut map = Map::new();
        map.insert("field1".to_string(), Value::String("value1".to_string()));
        map.insert("field2".to_string(), Value::String("value2".to_string()));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        assert_eq!(json["field1"], "value1");
        assert_eq!(json["field2"], "value2");
    }

    #[test]
    fn test_dynamic_row_preserves_type_information() {
        let mut map = Map::new();
        map.insert("int_field".to_string(), Value::Number(Number::from(42)));
        map.insert("str_field".to_string(), Value::String("42".to_string()));
        map.insert("bool_field".to_string(), Value::Bool(true));

        let row = DynamicRow(map);
        let json = serde_json::to_value(&row).unwrap();

        // 验证类型信息被保留
        assert!(matches!(json["int_field"], Value::Number(_)));
        assert!(matches!(json["str_field"], Value::String(_)));
        assert!(matches!(json["bool_field"], Value::Bool(_)));
    }
}
