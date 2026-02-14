use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// OdooString handles Odoo's dynamic typing where empty text fields are returned as boolean `false`.
/// Mirrors Go's `OdooString` type from `internal/models/odoo_types.go`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct OdooString(pub String);

impl<'de> Deserialize<'de> for OdooString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrBool {
            String(String),
            Bool(bool),
        }

        match StringOrBool::deserialize(deserializer)? {
            StringOrBool::String(s) => Ok(OdooString(s)),
            StringOrBool::Bool(b) => {
                if !b {
                    Ok(OdooString(String::new()))
                } else {
                    Ok(OdooString("true".to_string()))
                }
            }
        }
    }
}

impl Serialize for OdooString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl fmt::Display for OdooString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// sea-orm integration: Value conversion
impl From<OdooString> for sea_orm::Value {
    fn from(os: OdooString) -> Self {
        sea_orm::Value::String(Some(Box::new(os.0)))
    }
}

impl sea_orm::TryGetable for OdooString {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let val: String = res.try_get_by(index)?;
        Ok(OdooString(val))
    }
}

impl sea_orm::sea_query::ValueType for OdooString {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(x)) => Ok(OdooString(*x)),
            sea_orm::Value::String(None) => Ok(OdooString(String::new())),
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(OdooString).to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None)
    }
}

impl sea_orm::sea_query::Nullable for OdooString {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}
