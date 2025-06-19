use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Represents Helm chart values
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Values {
    /// The underlying values map
    #[serde(flatten)]
    pub data: IndexMap<String, JsonValue>,
}

impl Values {
    /// Create new empty values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create values from a JSON value
    pub fn from_json(value: JsonValue) -> crate::Result<Self> {
        match value {
            JsonValue::Object(map) => {
                let data = map.into_iter().collect();
                Ok(Self { data })
            }
            _ => Err(crate::Error::InvalidValues(
                "Values must be a JSON object".to_string()
            ))
        }
    }

    /// Create values from a YAML string
    pub fn from_yaml(yaml: &str) -> crate::Result<Self> {
        let value: JsonValue = serde_yaml::from_str(yaml)?;
        Self::from_json(value)
    }

    /// Set a value using a dot-separated path
    pub fn set(&mut self, path: &str, value: JsonValue) {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return;
        }

        // For single-level paths, set directly
        if parts.len() == 1 {
            self.data.insert(parts[0].to_string(), value);
            return;
        }

        // For nested paths, we need to build the structure
        let key = parts[0].to_string();
        let rest_path = parts[1..].join(".");
        
        // Get or create the nested object
        let entry = self.data
            .entry(key)
            .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
        
        match entry {
            JsonValue::Object(map) => {
                // Convert to Values, set the nested value, then convert back
                let mut nested = Values::from(map.clone());
                nested.set(&rest_path, value);
                *map = nested.data.into_iter()
                    .map(|(k, v)| (k, v))
                    .collect();
            }
            _ => {
                // If the existing value is not an object, replace it
                let mut nested = Values::new();
                nested.set(&rest_path, value);
                *entry = JsonValue::Object(
                    nested.data.into_iter()
                        .map(|(k, v)| (k, v))
                        .collect()
                );
            }
        }
    }

    /// Get a value using a dot-separated path
    pub fn get(&self, path: &str) -> Option<JsonValue> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // Convert to JSON for easier traversal
        let json = self.to_json();
        let mut current = &json;
        
        for part in parts {
            match current {
                JsonValue::Object(map) => {
                    current = map.get(part)?;
                }
                _ => return None,
            }
        }
        
        Some(current.clone())
    }

    /// Merge another Values object into this one
    pub fn merge(&mut self, other: Values) {
        for (key, value) in other.data {
            match (self.data.get_mut(&key), value) {
                (Some(JsonValue::Object(existing)), JsonValue::Object(new)) => {
                    // Deep merge objects
                    Self::merge_objects(existing, new);
                }
                (_, value) => {
                    // Replace value
                    self.data.insert(key, value);
                }
            }
        }
    }

    /// Deep merge two JSON objects
    fn merge_objects(target: &mut serde_json::Map<String, JsonValue>, source: serde_json::Map<String, JsonValue>) {
        for (key, value) in source {
            match (target.get_mut(&key), value) {
                (Some(JsonValue::Object(existing)), JsonValue::Object(new)) => {
                    Self::merge_objects(existing, new);
                }
                (_, value) => {
                    target.insert(key, value);
                }
            }
        }
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> JsonValue {
        JsonValue::Object(
            self.data.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        )
    }

    /// Convert to YAML string
    pub fn to_yaml(&self) -> crate::Result<String> {
        let yaml = serde_yaml::to_string(&self.to_json())?;
        Ok(yaml)
    }

    /// Check if values are empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all values
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl From<IndexMap<String, JsonValue>> for Values {
    fn from(data: IndexMap<String, JsonValue>) -> Self {
        Self { data }
    }
}

impl From<serde_json::Map<String, JsonValue>> for Values {
    fn from(map: serde_json::Map<String, JsonValue>) -> Self {
        let data = map.into_iter().collect();
        Self { data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get_values() {
        let mut values = Values::new();
        values.set("postgresql.auth.password", JsonValue::String("secret".to_string()));
        
        assert_eq!(
            values.get("postgresql.auth.password"),
            Some(JsonValue::String("secret".to_string()))
        );
    }

    #[test]
    fn test_merge_values() {
        let mut values1 = Values::new();
        values1.set("app.name", JsonValue::String("myapp".to_string()));
        values1.set("app.port", JsonValue::Number(8080.into()));

        let mut values2 = Values::new();
        values2.set("app.port", JsonValue::Number(9090.into()));
        values2.set("app.debug", JsonValue::Bool(true));

        values1.merge(values2);

        assert_eq!(
            values1.get("app.name"),
            Some(JsonValue::String("myapp".to_string()))
        );
        assert_eq!(
            values1.get("app.port"),
            Some(JsonValue::Number(9090.into()))
        );
        assert_eq!(
            values1.get("app.debug"),
            Some(JsonValue::Bool(true))
        );
    }
}