//! Public lesson DTOs are a whitelist, independent of editorial and grading data.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ts_rs::TS;
mod audio;
mod validation;

macro_rules! dto {
    ($name:ident { $($(#[$meta:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
        #[serde(rename_all="camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$meta])* pub $field: $ty),* }
    };
}
dto!(Title {
    fr: String,
    zh: String
});
dto!(Vocabulary { id: String, lemma: String, part_of_speech: String, gender: Option<String>, meaning_zh: String, note_zh: String });
dto!(Grammar { id: String, title_zh: String, body_zh: String, examples: Vec<Title> });
dto!(Knowledge { vocabulary: Vec<Vocabulary>, grammar: Vec<Grammar> });
dto!(Character {
    character_id: String,
    revision: u32,
    display_name: String,
    avatar_id: String,
    speech_locale: String
});
dto!(MediaAsset {
    asset_id: String,
    revision: u32,
    sha256: String,
    mime_type: String,
    width: u32,
    height: u32,
    alt_zh: String,
    credit_zh: String,
    url: String
});
dto!(AudioAsset {
    asset_id: String,
    revision: u32,
    sha256: String,
    mime_type: String,
    duration_ms: u32,
    credit_zh: String,
    url: String
});
// Unicode scalar offsets within the referenced segment, never UTF-16 offsets.
dto!(AudioWordRange {
    start: u32,
    end: u32
});
dto!(AudioCue { entry_id: String, segment_id: Option<String>, word_range: Option<AudioWordRange>, start_ms: u32, end_ms: u32 });
dto!(AudioTrack { block_id: String, asset_id: String, cues: Vec<AudioCue> });
dto!(Speaker {
    id: String,
    label_zh: String,
    character_id: String,
    display_name: String,
    avatar_id: String
});
dto!(Segment { id: String, text: String, vocabulary_id: Option<String>, grammar_id: Option<String> });
dto!(Turn { id: String, speaker_id: String, segments: Vec<Segment>, translation_zh: String });
dto!(Paragraph { id: String, segments: Vec<Segment>, translation_zh: String });
dto!(Target {
    block_id: String,
    entry_id: String,
    segment_id: String
});
dto!(OptionItem {
    id: String,
    text: String
});
dto!(Step { id: String, kind: String, title_zh: String, block_ids: Vec<String> });
dto!(Completion { strategy: String, required_step_ids: Vec<String>, required_exercise_ids: Vec<String> });

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "exerciseType", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Exercise {
    #[serde(rename_all = "camelCase")]
    SingleChoice {
        prompt_zh: String,
        options: Vec<OptionItem>,
    },
    #[serde(rename_all = "camelCase")]
    FillBlank {
        prompt_zh: String,
        template_fr: String,
        hint_zh: String,
    },
    #[serde(rename_all = "camelCase")]
    Order {
        prompt_zh: String,
        tokens: Vec<OptionItem>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Block {
    #[serde(rename_all = "camelCase")]
    Scene {
        id: String,
        place_zh: String,
        situation_zh: String,
        illustration_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Dialogue {
        id: String,
        title_zh: String,
        speakers: Vec<Speaker>,
        turns: Vec<Turn>,
    },
    #[serde(rename_all = "camelCase")]
    Article {
        id: String,
        title_zh: String,
        narrator_id: String,
        paragraphs: Vec<Paragraph>,
    },
    #[serde(rename_all = "camelCase")]
    Explanation {
        id: String,
        title_zh: String,
        body_zh: String,
        targets: Vec<Target>,
    },
    #[serde(rename_all = "camelCase")]
    Culture {
        id: String,
        title_zh: String,
        body_zh: String,
        scope_zh: String,
    },
    #[serde(rename_all = "camelCase")]
    Vocabulary { id: String, entry_ids: Vec<String> },
    #[serde(rename_all = "camelCase")]
    Grammar { id: String, entry_ids: Vec<String> },
    Exercise {
        id: String,
        #[serde(flatten)]
        exercise: Exercise,
    },
    #[serde(rename_all = "camelCase")]
    Habit {
        id: String,
        task_zh: String,
        alternative_zh: String,
    },
    #[serde(rename_all = "camelCase")]
    Summary {
        id: String,
        takeaways_zh: Vec<String>,
    },
}
impl Block {
    pub fn id(&self) -> &str {
        match self {
            Self::Scene { id, .. }
            | Self::Dialogue { id, .. }
            | Self::Article { id, .. }
            | Self::Explanation { id, .. }
            | Self::Culture { id, .. }
            | Self::Vocabulary { id, .. }
            | Self::Grammar { id, .. }
            | Self::Exercise { id, .. }
            | Self::Habit { id, .. }
            | Self::Summary { id, .. } => id,
        }
    }
}
dto!(PublicLesson {
    schema_version: String, id: String, revision: u32, level_id: String, unit_id: String,
    title: Title, summary_zh: String, estimated_minutes: u32, objectives_zh: Vec<String>,
    knowledge: Knowledge, blocks: Vec<Block>, steps: Vec<Step>, completion: Completion,
    review_item_ids: Vec<String>, cast: Vec<Character>, #[serde(default)] media: Vec<MediaAsset>,
    #[serde(default, skip_serializing_if="Vec::is_empty")] audio: Vec<AudioAsset>,
    #[serde(default, skip_serializing_if="Vec::is_empty")] audio_tracks: Vec<AudioTrack>
});
dto!(LessonSummary {
    id: String,
    revision: u32,
    level_id: String,
    unit_id: String,
    title: Title,
    summary_zh: String,
    estimated_minutes: u32
});
dto!(Unit { id: String, title_zh: String, lessons: Vec<LessonSummary> });
dto!(Level { id: String, label: String, units: Vec<Unit> });
dto!(Catalog { levels: Vec<Level>, development_fixture: bool });
dto!(PreviewRelease { id: String, catalog: Catalog, withdrawn_lesson_ids: Vec<String> });
dto!(ApiError {
    code: String,
    message: String
});

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewRating {
    Again,
    Remembered,
    Familiar,
}
dto!(ReviewCard {
    id: String,
    knowledge_id: String,
    source_lesson_id: String,
    source_revision: u32,
    vocabulary: Vocabulary,
    stage: i16,
    due_at: String,
    version: u32,
    suspended: bool
});
dto!(ReviewPreferenceRequest {
    card_version: u32,
    idempotency_key: String,
    suspended: bool
});
dto!(ReviewEnrollmentRequest {
    knowledge_id: String,
    source_lesson_id: String,
    source_revision: u32,
    idempotency_key: String
});
dto!(SavedItem { id: String, knowledge_id: String, source_lesson_id: String, source_revision: u32, vocabulary: Option<Vocabulary>, saved: bool, withdrawn: bool, version: u32, created_at: String });
dto!(SavedWriteRequest {
    source_lesson_id: String,
    source_revision: u32,
    saved: bool,
    version: u32,
    idempotency_key: String
});
dto!(SavedPage { items: Vec<SavedItem>, next_cursor: Option<String> });
dto!(ReviewHistoryItem { id: String, card_id: String, vocabulary: Option<Vocabulary>, withdrawn: bool, rating: ReviewRating, old_stage: i16, new_stage: i16, reviewed_at: String, due_at: String, time_zone: String, algorithm_version: String });
dto!(ReviewHistoryPage { items: Vec<ReviewHistoryItem>, next_cursor: Option<String> });
dto!(ReviewQueue { items: Vec<ReviewCard>, due_count: u32, next_due_at: Option<String>, local_date: String, time_zone: String });
dto!(ReviewCardsPage { items: Vec<ReviewCard>, next_cursor: Option<String> });
dto!(ReviewAttemptRequest {
    card_version: u32,
    idempotency_key: String,
    rating: ReviewRating
});
dto!(ReviewAttemptResult {
    card: ReviewCard,
    reviewed_at: String,
    time_zone: String
});

/// Submitted values are IDs/text, never a client supplied score or answer key.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ExerciseAnswer {
    #[serde(rename_all = "camelCase")]
    Choice {
        option_id: String,
    },
    Text {
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    Order {
        token_ids: Vec<String>,
    },
}
dto!(GradeRequest {
    revision: u32,
    exercise_id: String,
    answer: ExerciseAnswer
});
dto!(GradeResult {
    exercise_id: String,
    correct: bool,
    feedback_zh: String
});
dto!(CsrfToken { csrf_token: String });
dto!(StartLearningRequest {
    lesson_id: String,
    schema_version: String,
    idempotency_key: String
});
dto!(LearningWriteRequest {
    version: u32,
    idempotency_key: String
});
dto!(SubmitAttemptRequest {
    version: u32,
    idempotency_key: String,
    exercise_id: String,
    answer: ExerciseAnswer
});
dto!(AttemptRecord {
    id: String,
    exercise_id: String,
    attempt_index: u32,
    answer: ExerciseAnswer,
    result: GradeResult,
    hint_used: bool
});
dto!(LearningState {
    id: String, lesson_id: String, revision: u32, version: u32, last_step_id: Option<String>,
    confirmed_step_ids: Vec<String>, hinted_exercise_ids: Vec<String>, attempts: Vec<AttemptRecord>,
    completed_at: Option<String>, first_completed_at: Option<String>
});
dto!(LearningSession {
    lesson: PublicLesson,
    progress: LearningState
});
dto!(AttemptResult {
    result: GradeResult,
    progress: LearningState
});
dto!(HintResult {
    hint_zh: String,
    progress: LearningState
});
dto!(LearningOverviewItem {
    session_id: String, lesson_id: String, revision: u32, title: Title,
    last_step_id: Option<String>, completed_at: Option<String>, first_completed_at: Option<String>, updated_at: String
});
dto!(LearningOverview { items: Vec<LearningOverviewItem>, next_cursor: Option<String>, completed_lessons: u32 });
dto!(StudyDay {
    local_date: String,
    confirmed_steps: u32,
    exercise_attempts: u32,
    review_attempts: u32,
    completed_lessons: u32,
    active: bool
});
dto!(StudyDashboard { local_date: String, time_zone: String, week_start: String, days: Vec<StudyDay>, active_days: u8, weekly_goal_days: u8, daily_goal_minutes: u8, due_reviews: u32, next_review_at: Option<String>, completed_lessons: u32, resume: Option<LearningOverviewItem>, recommended_lesson: Option<LessonSummary>, all_available_completed: bool, course_states: Vec<LearningOverviewItem>, catalog: Catalog });
dto!(UserProfile {
    id: String,
    email: String,
    display_name: String,
    role: String,
    settings: UserSettings,
    version: u32
});
dto!(UserSettings {
    time_zone: String,
    weekly_days: u8,
    daily_minutes: u8,
    show_translation: bool,
    speech_rate: f64
});
impl Default for UserSettings {
    fn default() -> Self {
        Self {
            time_zone: "Asia/Shanghai".into(),
            weekly_days: 5,
            daily_minutes: 10,
            show_translation: false,
            speech_rate: 1.0,
        }
    }
}
dto!(UpdateProfileRequest {
    version: u32,
    display_name: Option<String>,
    time_zone: Option<String>,
    weekly_days: Option<u8>,
    daily_minutes: Option<u8>,
    show_translation: Option<bool>,
    speech_rate: Option<f64>
});
dto!(AuthResult {
    user: UserProfile,
    csrf_token: String
});
macro_rules! secret_dto {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
        #[serde(rename_all="camelCase", deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty),* }
    };
}
secret_dto!(LoginRequest {
    email: String,
    password: String
});
secret_dto!(AcceptInviteRequest {
    token: String,
    email: String,
    password: String,
    display_name: String
});
secret_dto!(ResetPasswordRequest {
    token: String,
    password: String
});

impl PublicLesson {
    pub fn summary(&self) -> LessonSummary {
        LessonSummary {
            id: self.id.clone(),
            revision: self.revision,
            level_id: self.level_id.clone(),
            unit_id: self.unit_id.clone(),
            title: self.title.clone(),
            summary_zh: self.summary_zh.clone(),
            estimated_minutes: self.estimated_minutes,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != "1.0" || self.revision == 0 || self.blocks.is_empty() {
            return Err("unsupported version or empty lesson".into());
        }
        let mut ids = HashSet::new();
        let mut insert = |id: &str| {
            if id.is_empty() || !ids.insert(id.to_owned()) {
                Err(format!("duplicate/empty id: {id}"))
            } else {
                Ok(())
            }
        };
        for v in &self.knowledge.vocabulary {
            insert(&v.id)?;
        }
        for g in &self.knowledge.grammar {
            insert(&g.id)?;
        }
        for b in &self.blocks {
            insert(b.id())?;
        }
        for s in &self.steps {
            insert(&s.id)?;
        }
        let vocab: HashSet<_> = self
            .knowledge
            .vocabulary
            .iter()
            .map(|v| v.id.as_str())
            .collect();
        let grammar: HashSet<_> = self
            .knowledge
            .grammar
            .iter()
            .map(|v| v.id.as_str())
            .collect();
        let cast: HashSet<_> = self.cast.iter().map(|v| v.character_id.as_str()).collect();
        if cast.len() != self.cast.len() {
            return Err("duplicate cast character".into());
        }
        let check_segments = |segments: &[Segment]| -> Result<(), String> {
            for s in segments {
                if s.vocabulary_id
                    .as_deref()
                    .is_some_and(|id| !vocab.contains(id))
                    || s.grammar_id
                        .as_deref()
                        .is_some_and(|id| !grammar.contains(id))
                {
                    return Err(format!("unknown anchor at {}", s.id));
                }
            }
            Ok(())
        };
        for b in &self.blocks {
            match b {
                Block::Dialogue {
                    speakers, turns, ..
                } => {
                    let speaker_ids: HashSet<_> = speakers.iter().map(|s| s.id.as_str()).collect();
                    if speaker_ids.len() != speakers.len() {
                        return Err("duplicate speaker".into());
                    }
                    for s in speakers {
                        if !cast.contains(s.character_id.as_str()) {
                            return Err("unknown cast member".into());
                        }
                    }
                    for t in turns {
                        if !speaker_ids.contains(t.speaker_id.as_str()) {
                            return Err("unknown speaker".into());
                        }
                        check_segments(&t.segments)?;
                    }
                }
                Block::Article {
                    narrator_id,
                    paragraphs,
                    ..
                } => {
                    if !cast.contains(narrator_id.as_str()) {
                        return Err("unknown narrator".into());
                    }
                    for p in paragraphs {
                        check_segments(&p.segments)?;
                    }
                }
                Block::Vocabulary { entry_ids, .. } => {
                    if entry_ids.iter().any(|id| !vocab.contains(id.as_str())) {
                        return Err("unknown vocabulary".into());
                    }
                }
                Block::Grammar { entry_ids, .. }
                    if entry_ids.iter().any(|id| !grammar.contains(id.as_str())) =>
                {
                    return Err("unknown grammar".into());
                }
                _ => {}
            }
        }
        for s in &self.steps {
            if s.block_ids
                .iter()
                .any(|id| !self.blocks.iter().any(|b| b.id() == id))
            {
                return Err("unknown step block".into());
            }
        }
        if self
            .review_item_ids
            .iter()
            .any(|id| !vocab.contains(id.as_str()))
        {
            return Err("unknown review item".into());
        }
        if self
            .completion
            .required_step_ids
            .iter()
            .any(|id| !self.steps.iter().any(|s| &s.id == id))
            || self.completion.required_exercise_ids.iter().any(|id| {
                !self
                    .blocks
                    .iter()
                    .any(|b| matches!(b,Block::Exercise{id:bid,..} if bid==id))
            })
        {
            return Err("unknown completion reference".into());
        }
        self.validate_flow()?;
        self.validate_audio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(crate) fn fixture() -> PublicLesson {
        let mut source: serde_json::Value =
            serde_json::from_str(include_str!("../../../docs/examples/a1-bakery.lesson.json"))
                .unwrap();
        source.as_object_mut().unwrap().remove("serverOnly");
        source.as_object_mut().unwrap().remove("editorial");
        serde_json::from_value(source).unwrap()
    }
    #[test]
    fn valid_sample() {
        fixture().validate().unwrap();
    }
    #[test]
    fn invalid_anchor() {
        let mut lesson = fixture();
        lesson.knowledge.vocabulary.clear();
        assert!(lesson.validate().is_err());
    }
    #[test]
    fn unknown_block_rejected() {
        assert!(serde_json::from_str::<Block>(r#"{"type":"script","id":"x"}"#).is_err());
    }
    #[test]
    fn dto_excludes_private_fields() {
        let json = serde_json::to_string(&fixture()).unwrap();
        for secret in [
            "serverOnly",
            "correctOptionId",
            "correctTokenIds",
            "accepted",
            "editorial",
        ] {
            assert!(!json.contains(secret));
        }
    }
}
