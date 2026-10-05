//! Private author metadata is validated before projecting the public document.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EditorialStatus {
    Draft,
    Reviewed,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Editorial {
    pub status: EditorialStatus,
    pub note: String,
}

/// This schema belongs to author tooling; never export it to the Web contract package.
pub fn schema() -> serde_json::Value {
    #[derive(schemars::JsonSchema)]
    #[schemars(rename_all = "camelCase", deny_unknown_fields)]
    #[allow(dead_code)]
    struct AuthorLesson {
        #[schemars(flatten)]
        lesson: brioche_course_contract::PublicLesson,
        editorial: Editorial,
        server_only: crate::grading::PrivateRules,
        #[schemars(default)]
        asset_refs: Vec<crate::media::AssetRef>,
        #[schemars(default)]
        audio_refs: Vec<crate::media::AssetRef>,
    }
    serde_json::to_value(schemars::schema_for!(AuthorLesson)).expect("schema serialization")
}

pub fn editorial(source: &serde_json::Value) -> Result<Editorial> {
    let value = source
        .get("editorial")
        .context("/editorial: required author metadata missing")?;
    let metadata: Editorial = crate::author_json::from_value(value.clone(), "/editorial")?;
    ensure!(
        !metadata.note.trim().is_empty()
            && metadata.note.len() <= 8000
            && !metadata
                .note
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t')),
        "/editorial/note: expected nonempty text of at most 8000 bytes without control characters"
    );
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn requires_explicit_valid_author_metadata() {
        for source in [
            json!({}),
            json!({"editorial":null}),
            json!({"editorial":{"status":"published","note":"review"}}),
            json!({"editorial":{"status":"draft","note":" "}}),
            json!({"editorial":{"status":"draft","note":"review","ignored":true}}),
        ] {
            assert!(editorial(&source).is_err());
        }
        assert!(matches!(
            editorial(&json!({"editorial":{"status":"draft","note":"Needs review"}}))
                .unwrap()
                .status,
            EditorialStatus::Draft
        ));
        assert!(matches!(
            editorial(
                &json!({"editorial":{"status":"reviewed","note":"Explicit author assertion"}})
            )
            .unwrap()
            .status,
            EditorialStatus::Reviewed
        ));
    }

    #[test]
    fn schema_includes_private_contract_only_for_author_tooling() {
        let schema = schema();
        let required = schema["required"].as_array().unwrap();
        for field in ["id", "blocks", "serverOnly", "editorial"] {
            assert!(required.contains(&serde_json::json!(field)), "{field}");
        }
        assert!(!required.contains(&serde_json::json!("assetRefs")));
        assert!(!required.contains(&serde_json::json!("audioRefs")));
        assert!(schema["properties"]["audioRefs"].is_object());
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["$defs"]["Editorial"]["additionalProperties"], false);
        assert!(schema.to_string().contains("correctOptionId"));
        let public =
            serde_json::to_value(schemars::schema_for!(brioche_course_contract::PublicLesson))
                .unwrap();
        for private in [
            "correctOptionId",
            "accepted",
            "correctTokenIds",
            "serverOnly",
            "editorial",
            "audioRefs",
        ] {
            assert!(!public.to_string().contains(private));
        }
    }
}
