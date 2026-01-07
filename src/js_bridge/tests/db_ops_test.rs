#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};
    use crate::js_bridge::ops::db_ops::DynamicRow;
    use super::*;

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
}
