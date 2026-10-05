//! Answer keys stay in this crate. Validation precedes import and grading.
use brioche_course_contract::{Block, Exercise, ExerciseAnswer, GradeResult, PublicLesson};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum Rule {
    #[serde(rename_all = "camelCase")]
    Choice {
        correct_option_id: String,
        feedback_zh: String,
    },
    #[serde(rename_all = "camelCase")]
    Text {
        accepted: Vec<String>,
        case_sensitive: bool,
        feedback_zh: String,
    },
    #[serde(rename_all = "camelCase")]
    Order {
        correct_token_ids: Vec<String>,
        feedback_zh: String,
    },
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrivateRules {
    grading: BTreeMap<String, Rule>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GradeError {
    InvalidContent,
    UnknownExercise,
    InvalidAnswer,
}

/// NFC, whitespace and French apostrophe variants are equivalent; accents remain meaningful.
pub fn normalize_text(text: &str, case_sensitive: bool) -> String {
    let normalized: String = text
        .nfc()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{02bc}' => '\'',
            _ => c,
        })
        .collect();
    let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    if case_sensitive {
        normalized
    } else {
        normalized.to_lowercase()
    }
}

pub struct Grader {
    rules: BTreeMap<String, Rule>,
}
impl Grader {
    pub fn from_source(
        lesson: &PublicLesson,
        source: &serde_json::Value,
    ) -> Result<Self, GradeError> {
        Self::from_author_source(lesson, source).map_err(|_| GradeError::InvalidContent)
    }

    /// Author-only diagnostics. HTTP handlers continue using the opaque GradeError.
    pub fn from_author_source(
        lesson: &PublicLesson,
        source: &serde_json::Value,
    ) -> anyhow::Result<Self> {
        use anyhow::{Context, bail, ensure};
        let rules: PrivateRules = crate::author_json::from_value(
            source
                .get("serverOnly")
                .cloned()
                .context("/serverOnly: missing private rules")?,
            "/serverOnly",
        )?;
        let mut exercises = BTreeMap::new();
        for (index, block) in lesson.blocks.iter().enumerate() {
            if let Block::Exercise { id, exercise } = block {
                ensure!(
                    exercises.insert(id, exercise).is_none(),
                    "/blocks/{index}/id: duplicate exercise ID"
                );
            }
        }
        let path = |id: &str| {
            format!(
                "/serverOnly/grading/{}",
                id.replace('~', "~0").replace('/', "~1")
            )
        };
        for id in rules.grading.keys() {
            ensure!(
                exercises.contains_key(id),
                "{}: rule references unknown exercise",
                path(id)
            );
        }
        for (id, exercise) in exercises {
            let pointer = path(id);
            let rule = rules
                .grading
                .get(id)
                .with_context(|| format!("{pointer}: missing grading rule"))?;
            let feedback = match rule {
                Rule::Choice { feedback_zh, .. }
                | Rule::Text { feedback_zh, .. }
                | Rule::Order { feedback_zh, .. } => feedback_zh,
            };
            ensure!(
                !feedback.trim().is_empty(),
                "{pointer}/feedbackZh: expected nonempty feedback"
            );
            match (exercise, rule) {
                (
                    Exercise::SingleChoice { options, .. },
                    Rule::Choice {
                        correct_option_id, ..
                    },
                ) => {
                    ensure!(
                        options.iter().any(|option| &option.id == correct_option_id),
                        "{pointer}/correctOptionId: unknown option reference"
                    );
                }
                (
                    Exercise::FillBlank { .. },
                    Rule::Text {
                        accepted,
                        case_sensitive,
                        ..
                    },
                ) => {
                    ensure!(
                        !accepted.is_empty(),
                        "{pointer}/accepted: at least one accepted answer is required"
                    );
                    for (index, answer) in accepted.iter().enumerate() {
                        ensure!(
                            !normalize_text(answer, *case_sensitive).is_empty()
                                && answer.len() <= 4096,
                            "{pointer}/accepted/{index}: expected nonempty normalized answer of at most 4096 bytes"
                        );
                    }
                }
                (
                    Exercise::Order { tokens, .. },
                    Rule::Order {
                        correct_token_ids, ..
                    },
                ) => {
                    ensure!(
                        correct_token_ids.len() == tokens.len(),
                        "{pointer}/correctTokenIds: expected every token exactly once"
                    );
                    let mut seen = HashSet::new();
                    let known: HashSet<_> = tokens.iter().map(|token| &token.id).collect();
                    for (index, token) in correct_token_ids.iter().enumerate() {
                        ensure!(
                            known.contains(token) && seen.insert(token),
                            "{pointer}/correctTokenIds/{index}: unknown or duplicate token reference"
                        );
                    }
                }
                _ => bail!("{pointer}/kind: grading kind does not match exercise kind"),
            }
        }
        Ok(Self {
            rules: rules.grading,
        })
    }
    pub fn grade(
        &self,
        lesson: &PublicLesson,
        id: &str,
        answer: &ExerciseAnswer,
    ) -> Result<GradeResult, GradeError> {
        let exercise = lesson
            .blocks
            .iter()
            .find_map(|b| match b {
                Block::Exercise {
                    id: block_id,
                    exercise,
                } if block_id == id => Some(exercise),
                _ => None,
            })
            .ok_or(GradeError::UnknownExercise)?;
        let rule = self.rules.get(id).ok_or(GradeError::InvalidContent)?;
        let (correct, feedback) = match (exercise, rule, answer) {
            (
                Exercise::SingleChoice { options, .. },
                Rule::Choice {
                    correct_option_id,
                    feedback_zh,
                },
                ExerciseAnswer::Choice { option_id },
            ) => {
                if !options.iter().any(|o| &o.id == option_id) {
                    return Err(GradeError::InvalidAnswer);
                }
                (option_id == correct_option_id, feedback_zh)
            }
            (
                Exercise::FillBlank { .. },
                Rule::Text {
                    accepted,
                    case_sensitive,
                    feedback_zh,
                },
                ExerciseAnswer::Text { text },
            ) => {
                if text.len() > 4096 || normalize_text(text, *case_sensitive).is_empty() {
                    return Err(GradeError::InvalidAnswer);
                }
                (
                    accepted.iter().any(|a| {
                        normalize_text(a, *case_sensitive) == normalize_text(text, *case_sensitive)
                    }),
                    feedback_zh,
                )
            }
            (
                Exercise::Order { tokens, .. },
                Rule::Order {
                    correct_token_ids,
                    feedback_zh,
                },
                ExerciseAnswer::Order { token_ids },
            ) => {
                let ids: HashSet<_> = token_ids.iter().collect();
                if ids.len() != tokens.len()
                    || ids.len() != token_ids.len()
                    || !tokens.iter().all(|t| ids.contains(&t.id))
                {
                    return Err(GradeError::InvalidAnswer);
                }
                (token_ids == correct_token_ids, feedback_zh)
            }
            _ => return Err(GradeError::InvalidAnswer),
        };
        Ok(GradeResult {
            exercise_id: id.into(),
            correct,
            feedback_zh: feedback.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (PublicLesson, serde_json::Value) {
        let source: serde_json::Value =
            serde_json::from_str(include_str!("../../../docs/examples/a1-bakery.lesson.json"))
                .unwrap();
        (crate::project_source(source.clone()).unwrap(), source)
    }
    #[test]
    fn normalizes_without_removing_accents() {
        assert_eq!(
            normalize_text("  S’il\u{00a0}vous\u{202f}plaît  ", false),
            "s'il vous plaît"
        );
        assert_eq!(normalize_text("cafe\u{301}", false), "café");
        assert_ne!(normalize_text("cafe", false), normalize_text("café", false));
        assert_ne!(normalize_text("Une", true), normalize_text("une", true));
    }
    #[test]
    fn grades_three_kinds_and_rejects_forged_inputs() {
        let (lesson, source) = fixture();
        let grader = Grader::from_source(&lesson, &source).unwrap();
        assert!(
            grader
                .grade(
                    &lesson,
                    "exercise-intention",
                    &ExerciseAnswer::Choice {
                        option_id: "request-bread".into()
                    }
                )
                .unwrap()
                .correct
        );
        assert!(
            !grader
                .grade(
                    &lesson,
                    "exercise-intention",
                    &ExerciseAnswer::Choice {
                        option_id: "ask-price".into()
                    }
                )
                .unwrap()
                .correct
        );
        assert!(
            grader
                .grade(
                    &lesson,
                    "exercise-article",
                    &ExerciseAnswer::Text {
                        text: " UNE\u{00a0}".into()
                    }
                )
                .unwrap()
                .correct
        );
        assert!(
            grader
                .grade(
                    &lesson,
                    "exercise-order",
                    &ExerciseAnswer::Order {
                        token_ids: vec!["request".into(), "bread".into(), "please".into()]
                    }
                )
                .unwrap()
                .correct
        );
        assert!(
            !grader
                .grade(
                    &lesson,
                    "exercise-order",
                    &ExerciseAnswer::Order {
                        token_ids: vec!["bread".into(), "request".into(), "please".into()]
                    }
                )
                .unwrap()
                .correct
        );
        for (id, answer) in [
            (
                "exercise-intention",
                ExerciseAnswer::Choice {
                    option_id: "forged".into(),
                },
            ),
            (
                "exercise-intention",
                ExerciseAnswer::Text {
                    text: "request-bread".into(),
                },
            ),
            (
                "exercise-order",
                ExerciseAnswer::Order {
                    token_ids: vec!["request".into(); 3],
                },
            ),
            (
                "exercise-article",
                ExerciseAnswer::Text { text: " ".into() },
            ),
        ] {
            assert_eq!(
                grader.grade(&lesson, id, &answer).unwrap_err(),
                GradeError::InvalidAnswer
            );
        }
    }
    #[test]
    fn rejects_missing_wrong_and_extra_rules() {
        let (lesson, source) = fixture();
        for pointer in [
            "/serverOnly/grading/exercise-intention/correctOptionId",
            "/serverOnly/grading/exercise-order/correctTokenIds",
            "/serverOnly/grading/exercise-article/accepted",
        ] {
            let mut invalid = source.clone();
            *invalid.pointer_mut(pointer).unwrap() = serde_json::json!(null);
            assert!(Grader::from_source(&lesson, &invalid).is_err());
        }
        let mut invalid = source;
        invalid["serverOnly"]["grading"]["extra"] =
            invalid["serverOnly"]["grading"]["exercise-intention"].clone();
        assert!(Grader::from_source(&lesson, &invalid).is_err());
    }

    #[test]
    fn author_diagnostics_locate_rules_and_runtime_errors_remain_opaque() {
        let (lesson, original) = fixture();
        for (pointer, value, expected) in [
            (
                "/serverOnly/grading/exercise-intention/correctOptionId",
                serde_json::json!("private-missing-option"),
                "/serverOnly/grading/exercise-intention/correctOptionId",
            ),
            (
                "/serverOnly/grading/exercise-article/accepted",
                serde_json::json!([]),
                "/serverOnly/grading/exercise-article/accepted",
            ),
            (
                "/serverOnly/grading/exercise-article/accepted/0",
                serde_json::json!(" "),
                "/serverOnly/grading/exercise-article/accepted/0",
            ),
            (
                "/serverOnly/grading/exercise-order/correctTokenIds/1",
                serde_json::json!("private-missing-token"),
                "/serverOnly/grading/exercise-order/correctTokenIds/1",
            ),
            (
                "/serverOnly/grading/exercise-order/correctTokenIds/1",
                serde_json::json!("request"),
                "/serverOnly/grading/exercise-order/correctTokenIds/1",
            ),
            (
                "/serverOnly/grading/exercise-order/correctTokenIds",
                serde_json::json!([]),
                "/serverOnly/grading/exercise-order/correctTokenIds",
            ),
            (
                "/serverOnly/grading/exercise-intention/feedbackZh",
                serde_json::json!(""),
                "/serverOnly/grading/exercise-intention/feedbackZh",
            ),
        ] {
            let mut source = original.clone();
            *source.pointer_mut(pointer).unwrap() = value;
            let error = Grader::from_author_source(&lesson, &source)
                .err()
                .unwrap()
                .to_string();
            assert!(error.starts_with(&format!("{expected}: ")), "{error}");
            assert!(!error.contains("private-missing"));
            assert!(matches!(
                Grader::from_source(&lesson, &source),
                Err(GradeError::InvalidContent)
            ));
        }
        let mut missing = original.clone();
        missing["serverOnly"]["grading"]
            .as_object_mut()
            .unwrap()
            .remove("exercise-intention");
        assert!(
            Grader::from_author_source(&lesson, &missing)
                .err()
                .unwrap()
                .to_string()
                .starts_with("/serverOnly/grading/exercise-intention:")
        );
        let mut extra = original.clone();
        extra["serverOnly"]["grading"]["a/b~c"] =
            original["serverOnly"]["grading"]["exercise-intention"].clone();
        assert!(
            Grader::from_author_source(&lesson, &extra)
                .err()
                .unwrap()
                .to_string()
                .starts_with("/serverOnly/grading/a~1b~0c:")
        );
        let mut mismatch = original.clone();
        mismatch["serverOnly"]["grading"]["exercise-intention"] =
            original["serverOnly"]["grading"]["exercise-article"].clone();
        assert!(
            Grader::from_author_source(&lesson, &mismatch)
                .err()
                .unwrap()
                .to_string()
                .starts_with("/serverOnly/grading/exercise-intention/kind:")
        );
        let mut duplicate = lesson.clone();
        duplicate.blocks.push(
            lesson
                .blocks
                .iter()
                .find(|b| matches!(b, Block::Exercise { .. }))
                .unwrap()
                .clone(),
        );
        assert!(matches!(
            Grader::from_source(&duplicate, &original),
            Err(GradeError::InvalidContent)
        ));
        assert!(Grader::from_author_source(&lesson, &original).is_ok());
    }
}
