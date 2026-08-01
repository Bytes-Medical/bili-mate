//! Strict JSON parsing for clinical requests (SEC-006, API processing order
//! step 2): duplicate-key rejection first, then schema-strict deserialisation
//! with unknown properties denied and safe, pointer-addressed errors that
//! never reproduce the submitted value (API-017).

use std::collections::HashSet;
use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

use crate::problem::FieldError;

const DUPLICATE_MARKER: &str = "duplicate JSON object key";

/// Walks an entire JSON document, failing on any duplicated object key.
/// `serde_json` silently keeps the last duplicate by default, which is why
/// this pass exists.
struct DuplicateKeyCheck;

impl<'de> Deserialize<'de> for DuplicateKeyCheck {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DuplicateKeyVisitor)
    }
}

struct DuplicateKeyVisitor;

impl<'de> Visitor<'de> for DuplicateKeyVisitor {
    type Value = DuplicateKeyCheck;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any JSON value without duplicate object keys")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut seen = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key) {
                return Err(de::Error::custom(DUPLICATE_MARKER));
            }
            map.next_value::<DuplicateKeyCheck>()?;
        }
        Ok(DuplicateKeyCheck)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        while seq.next_element::<DuplicateKeyCheck>()?.is_some() {}
        Ok(DuplicateKeyCheck)
    }

    fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
        Ok(DuplicateKeyCheck)
    }
    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
        Ok(DuplicateKeyCheck)
    }
    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
        Ok(DuplicateKeyCheck)
    }
    fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
        Ok(DuplicateKeyCheck)
    }
    fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
        Ok(DuplicateKeyCheck)
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(DuplicateKeyCheck)
    }
}

pub enum ParseFailure {
    /// Not well-formed JSON.
    Malformed,
    /// Well-formed but contains a duplicate object key.
    DuplicateKey,
    /// Well-formed JSON that fails the schema; pointers identify fields
    /// without echoing values.
    Schema(Vec<FieldError>),
}

/// Parse a strict JSON body into `T` after the duplicate-key pass.
pub fn parse_strict<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, ParseFailure> {
    match serde_json::from_slice::<DuplicateKeyCheck>(body) {
        Ok(_) => {}
        Err(e) if e.to_string().contains(DUPLICATE_MARKER) => {
            return Err(ParseFailure::DuplicateKey)
        }
        Err(_) => return Err(ParseFailure::Malformed),
    }

    let deserializer = &mut serde_json::Deserializer::from_slice(body);
    match serde_path_to_error::deserialize::<_, T>(deserializer) {
        Ok(value) => Ok(value),
        Err(e) => {
            let pointer = path_to_pointer(e.path());
            Err(ParseFailure::Schema(vec![FieldError {
                pointer,
                code: "SCHEMA_INVALID".into(),
                message: "The value supplied for this field is missing, unknown or not permitted by the schema.".into(),
            }]))
        }
    }
}

/// Convert a serde path (`measurements[0].method`) to an RFC 6901 pointer
/// (`/measurements/0/method`).
fn path_to_pointer(path: &serde_path_to_error::Path) -> String {
    use serde_path_to_error::Segment;
    let mut pointer = String::new();
    for segment in path.iter() {
        match segment {
            Segment::Map { key } => {
                pointer.push('/');
                pointer.push_str(&key.replace('~', "~0").replace('/', "~1"));
            }
            Segment::Seq { index } => {
                pointer.push('/');
                pointer.push_str(&index.to_string());
            }
            Segment::Enum { variant } => {
                pointer.push('/');
                pointer.push_str(variant);
            }
            Segment::Unknown => {}
        }
    }
    if pointer.is_empty() {
        "/".into()
    } else {
        pointer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_unique_keys() {
        assert!(parse_strict::<serde_json::Value>(br#"{"a":1,"b":{"a":2},"c":[{"a":3}]}"#).is_ok());
    }

    #[test]
    fn rejects_top_level_duplicates() {
        assert!(matches!(
            parse_strict::<serde_json::Value>(br#"{"a":1,"a":2}"#),
            Err(ParseFailure::DuplicateKey)
        ));
    }

    #[test]
    fn rejects_nested_duplicates() {
        assert!(matches!(
            parse_strict::<serde_json::Value>(br#"{"a":{"b":1,"b":2}}"#),
            Err(ParseFailure::DuplicateKey)
        ));
        assert!(matches!(
            parse_strict::<serde_json::Value>(br#"{"a":[{"b":1,"b":2}]}"#),
            Err(ParseFailure::DuplicateKey)
        ));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(matches!(
            parse_strict::<serde_json::Value>(b"{"),
            Err(ParseFailure::Malformed)
        ));
    }

    #[test]
    fn schema_errors_carry_pointers_without_values() {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Target {
            #[allow(dead_code)]
            wanted: u8,
        }
        match parse_strict::<Target>(br#"{"wanted": 300}"#) {
            Err(ParseFailure::Schema(errors)) => {
                assert_eq!(errors[0].pointer, "/wanted");
                assert!(
                    !errors[0].message.contains("300"),
                    "must not echo the value"
                );
            }
            _ => panic!("expected schema failure"),
        }
    }
}
