//! Answer keys stay in this crate. Validation precedes import and grading.
use brioche_course_contract::{Block, Exercise, ExerciseAnswer, GradeResult, PublicLesson};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum Rule {
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
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateRules {
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
        let rules: PrivateRules = serde_json::from_value(
            source
                .get("serverOnly")
                .cloned()
                .ok_or(GradeError::InvalidContent)?,
        )
        .map_err(|_| GradeError::InvalidContent)?;
        let exercises: Vec<_> = lesson
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::Exercise { id, exercise } => Some((id, exercise)),
                _ => None,
            })
            .collect();
        if exercises.len() != rules.grading.len() {
            return Err(GradeError::InvalidContent);
        }
        for (id, exercise) in exercises {
            let valid = match (exercise, rules.grading.get(id)) {
                (
                    Exercise::SingleChoice { options, .. },
                    Some(Rule::Choice {
                        correct_option_id,
                        feedback_zh,
                    }),
                ) => {
                    !feedback_zh.trim().is_empty()
                        && options.iter().any(|o| &o.id == correct_option_id)
                }
                (
                    Exercise::FillBlank { .. },
                    Some(Rule::Text {
                        accepted,
                        case_sensitive,
                        feedback_zh,
                    }),
                ) => {
                    !feedback_zh.trim().is_empty()
                        && !accepted.is_empty()
                        && accepted.iter().all(|a| {
                            !normalize_text(a, *case_sensitive).is_empty() && a.len() <= 4096
                        })
                }
                (
                    Exercise::Order { tokens, .. },
                    Some(Rule::Order {
                        correct_token_ids,
                        feedback_zh,
                    }),
                ) => {
                    let ids: HashSet<_> = correct_token_ids.iter().collect();
                    !feedback_zh.trim().is_empty()
                        && ids.len() == tokens.len()
                        && ids.len() == correct_token_ids.len()
                        && tokens.iter().all(|t| ids.contains(&t.id))
                }
                _ => false,
            };
            if !valid {
                return Err(GradeError::InvalidContent);
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
}
