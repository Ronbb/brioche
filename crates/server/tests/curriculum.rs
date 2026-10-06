//! The authored pilot is data, checked with the same projection and grader as imports.
use brioche_course_contract::ExerciseAnswer;
use brioche_server::{author_json::Document, content::ReleaseManifest, grading::Grader};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

#[test]
fn scene_inventory_matches_actual_svg_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/content/a1");
    let document = Document::load(root.join("scene-assets.bundle.json")).unwrap();
    let bundle: brioche_server::media::AssetBundle =
        serde_json::from_value(document.value).unwrap();
    assert_eq!(bundle.schema_version, "1.0");
    assert_eq!(bundle.assets.len(), 3);
    for asset in bundle.assets {
        let info = brioche_server::media::inspect_file(
            &root.join("assets").join(&asset.file),
            &asset.mime_type,
        )
        .unwrap();
        assert_eq!(
            info.sha256, asset.sha256,
            "stale hash for {}",
            asset.asset_id
        );
        assert_eq!((info.width, info.height), (asset.width, asset.height));
    }
}

#[test]
fn pilot_sources_match_catalog_and_shared_knowledge_and_grade_all_exercises() {
    check_catalog(
        "catalog.release.json",
        &[
            "a1-first-conversations",
            "a1-breakfast-bakery",
            "a1-city-travel",
        ],
    );
}

#[test]
fn extended_sources_match_catalog_and_shared_knowledge_and_grade_all_exercises() {
    check_catalog(
        "catalog.extended.release.json",
        &[
            "a1-first-conversations",
            "a1-breakfast-bakery",
            "a1-city-travel",
            "a1-home-routine",
        ],
    );
}

fn check_catalog(file: &str, expected_units: &[&str]) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let document = Document::load(root.join("content/a1").join(file)).unwrap();
    let manifest: ReleaseManifest = serde_json::from_value(document.value.clone()).unwrap();
    manifest.validate_author().unwrap();
    let units = document.value["levels"][0]["units"].as_array().unwrap();
    assert_eq!(
        units
            .iter()
            .map(|unit| unit["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        expected_units
    );
    let mut knowledge = BTreeMap::<String, Value>::new();
    let mut checked = 0;
    for unit in units {
        let lessons = unit["lessons"].as_array().unwrap();
        assert_eq!(lessons.len(), 4);
        for reference in lessons {
            let id = reference["lessonId"].as_str().unwrap();
            let path = if id == "a1-bakery-buy-breakfast" {
                root.join("examples/a1-bakery.lesson.json")
            } else {
                root.join(format!("content/a1/{id}.lesson.json"))
            };
            let source = Document::load(path).unwrap().value;
            assert_eq!(source["id"], reference["lessonId"]);
            assert_eq!(source["revision"], reference["revision"]);
            assert_eq!(source["unitId"], unit["id"]);
            assert_eq!(source["levelId"], "a1");
            brioche_server::author_source::editorial(&source).unwrap();
            brioche_server::media::source_asset_refs(&source).unwrap();
            brioche_server::recording::source_audio_refs(&source).unwrap();
            let lesson = brioche_server::project_source(source.clone()).unwrap();
            let grader = Grader::from_author_source(&lesson, &source).unwrap();
            let public = serde_json::to_value(&lesson).unwrap();
            for private_field in ["serverOnly", "editorial", "assetRefs", "audioRefs"] {
                assert!(public.get(private_field).is_none());
            }
            for group in ["vocabulary", "grammar"] {
                for entry in source["knowledge"][group].as_array().unwrap() {
                    let key = entry["id"].as_str().unwrap().to_owned();
                    if let Some(previous) = knowledge.insert(key.clone(), entry.clone()) {
                        assert_eq!(
                            previous, *entry,
                            "conflicting shared knowledge {key} in {id}"
                        );
                    }
                }
            }
            let rules = source["serverOnly"]["grading"].as_object().unwrap();
            assert_eq!(rules.len(), 3);
            for (exercise_id, rule) in rules {
                let answer = match rule["kind"].as_str().unwrap() {
                    "choice" => ExerciseAnswer::Choice {
                        option_id: rule["correctOptionId"].as_str().unwrap().into(),
                    },
                    "text" => ExerciseAnswer::Text {
                        text: rule["accepted"][0].as_str().unwrap().into(),
                    },
                    "order" => ExerciseAnswer::Order {
                        token_ids: serde_json::from_value(rule["correctTokenIds"].clone()).unwrap(),
                    },
                    kind => panic!("unknown rule {kind}"),
                };
                assert!(grader.grade(&lesson, exercise_id, &answer).unwrap().correct);
                let wrong = match answer {
                    ExerciseAnswer::Choice { option_id } => {
                        let block = source["blocks"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|block| block["id"] == *exercise_id)
                            .unwrap();
                        let alternative = block["options"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|option| option["id"] != option_id)
                            .unwrap();
                        ExerciseAnswer::Choice {
                            option_id: alternative["id"].as_str().unwrap().into(),
                        }
                    }
                    ExerciseAnswer::Text { .. } => ExerciseAnswer::Text {
                        text: "incorrect-answer".into(),
                    },
                    ExerciseAnswer::Order { mut token_ids } => {
                        token_ids.swap(0, 1);
                        ExerciseAnswer::Order { token_ids }
                    }
                };
                assert!(!grader.grade(&lesson, exercise_id, &wrong).unwrap().correct);
            }
            checked += 1;
        }
    }
    assert_eq!(checked, expected_units.len() * 4);
}
