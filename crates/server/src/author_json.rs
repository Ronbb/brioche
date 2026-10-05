//! Bounded author files; duplicate object members must never silently overwrite content.
use anyhow::{Context, Result, bail};
use serde::{
    Deserializer,
    de::{self, DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{fmt, fs::File, io::Read, path::Path};

const LIMIT: usize = 2 * 1024 * 1024;

pub fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let file =
        File::open(path).with_context(|| format!("{}: cannot open author file", path.display()))?;
    let mut bytes = Vec::new();
    file.take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .with_context(|| format!("{}: cannot read author file", path.display()))?;
    if bytes.len() > LIMIT {
        bail!("{}: author JSON exceeds 2 MiB", path.display());
    }
    parse(&bytes).with_context(|| format!("{}: invalid author JSON", path.display()))?;
    let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
    serde_path_to_error::deserialize(&mut deserializer)
        .with_context(|| format!("{}: invalid author document", path.display()))
}

fn parse(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = Node(String::new()).deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

struct Node(String);
impl<'de> DeserializeSeed<'de> for Node {
    type Value = Value;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Node {
    type Value = Value;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON value")
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite number"))
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) =
            seq.next_element_seed(Node(format!("{}/{}", self.0, values.len())))?
        {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            let pointer = format!("{}/{}", self.0, key.replace('~', "~0").replace('/', "~1"));
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON field at {pointer}"
                )));
            }
            values.insert(key, map.next_value_seed(Node(pointer))?);
        }
        Ok(Value::Object(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_duplicate_members_with_pointer_and_location() {
        let error = parse(br#"{"steps":[{"a/b~c":1,"a/b~c":2}]}"#).unwrap_err();
        assert!(error.to_string().contains("/steps/0/a~1b~0c"));
        assert_eq!(error.line(), 1);
        assert!(error.column() > 1);
        assert!(parse(br#"{"id":1,"\u0069d":2}"#).is_err());
        assert!(parse(br#"{"a":{"id":1},"b":{"id":2}}"#).is_ok());
    }
    #[test]
    fn rejects_trailing_invalid_and_deep_documents() {
        for bytes in [b"{} {}".as_slice(), b"{", &[0xff]] {
            assert!(parse(bytes).is_err());
        }
        let deep = format!("{}0{}", "[".repeat(150), "]".repeat(150));
        assert!(parse(deep.as_bytes()).is_err());
        assert_eq!(
            parse(br#"[null,true,-1,18446744073709551615,1.25,"bonjour"]"#).unwrap(),
            serde_json::from_slice::<Value>(
                br#"[null,true,-1,18446744073709551615,1.25,"bonjour"]"#
            )
            .unwrap()
        );
    }
    #[test]
    fn bounds_actual_file_reads_and_names_the_file() {
        let path = std::env::temp_dir().join(format!(
            "brioche-author-{}.json",
            crate::learning::random_id().unwrap()
        ));
        std::fs::write(&path, vec![b' '; LIMIT + 1]).unwrap();
        let error = load::<Value>(&path).unwrap_err().to_string();
        assert!(error.contains("2 MiB"));
        assert!(error.contains(path.file_name().unwrap().to_str().unwrap()));
        std::fs::write(&path, br#"{"id":1,"id":2}"#).unwrap();
        assert!(
            format!("{:#}", load::<Value>(&path).unwrap_err())
                .contains("duplicate JSON field at /id")
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn typed_errors_preserve_nested_field_and_original_line() {
        let path = std::env::temp_dir().join(format!(
            "brioche-author-{}.json",
            crate::learning::random_id().unwrap()
        ));
        std::fs::write(
            &path,
            br#"{
          "id":"test", "schemaVersion":"1.0",
          "levels":[{"id":"a1", "label":"A1", "units":[{
            "id":"unit", "titleZh":"Breakfast",
            "lessons":[{"lessonId":"lesson", "revision":"bad"}]
          }]}]
        }"#,
        )
        .unwrap();
        let error = format!(
            "{:#}",
            load::<crate::content::ReleaseManifest>(&path).unwrap_err()
        );
        std::fs::remove_file(path).unwrap();
        assert!(
            error.contains("levels[0].units[0].lessons[0].revision"),
            "{error}"
        );
        assert!(error.contains("line 5 column"), "{error}");
    }

    #[test]
    fn current_author_examples_remain_readable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples");
        load::<Value>(root.join("a1-bakery.lesson.json")).unwrap();
        load::<crate::content::ReleaseManifest>(root.join("catalog.release.json")).unwrap();
        load::<crate::media::AssetBundle>(root.join("asset-bundle.json")).unwrap();
    }
}
