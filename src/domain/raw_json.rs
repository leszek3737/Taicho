use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type JsonMap = serde_json::Map<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawJson {
    value: Value,
}

impl RawJson {
    pub fn from_serialize<T: Serialize>(val: &T) -> Result<Self, serde_json::Error> {
        serde_json::to_value(val).map(|value| Self { value })
    }

    pub fn from_hash_map(map: std::collections::HashMap<String, Value>) -> Self {
        Self {
            value: Value::Object(map.into_iter().collect()),
        }
    }

    pub fn empty() -> Self {
        Self {
            value: Value::Object(Default::default()),
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn into_value(self) -> Value {
        self.value
    }

    pub fn to_hash_map(&self) -> Option<std::collections::HashMap<String, Value>> {
        match &self.value {
            Value::Object(map) => Some(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
            _ => None,
        }
    }

    pub fn to_json_map(&self) -> Option<JsonMap> {
        match &self.value {
            Value::Object(_) => Some(self.value.as_object().cloned().unwrap_or_default()),
            _ => None,
        }
    }
}

impl Default for RawJson {
    fn default() -> Self {
        Self::empty()
    }
}
