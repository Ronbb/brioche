use crate::{Block, Exercise, PublicLesson};
use std::collections::HashSet;

fn unique<'a>(values: impl Iterator<Item = &'a str>, path: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for value in values {
        if value.trim().is_empty() || !seen.insert(value) {
            return Err(format!("{path}: empty or duplicate ID {value}"));
        }
    }
    Ok(())
}
impl PublicLesson {
    pub(crate) fn validate_flow(&self) -> Result<(), String> {
        if self.id.trim().is_empty()
            || self.level_id.trim().is_empty()
            || self.unit_id.trim().is_empty()
            || self.title.fr.trim().is_empty()
            || self.title.zh.trim().is_empty()
            || self.steps.is_empty()
            || self.completion.strategy != "attempt-all"
            || self.completion.required_step_ids.is_empty()
        {
            return Err("/: missing lesson metadata or unsupported completion policy".into());
        }
        unique(
            self.review_item_ids.iter().map(String::as_str),
            "/reviewItemIds",
        )?;
        unique(
            self.completion.required_step_ids.iter().map(String::as_str),
            "/completion/requiredStepIds",
        )?;
        unique(
            self.completion
                .required_exercise_ids
                .iter()
                .map(String::as_str),
            "/completion/requiredExerciseIds",
        )?;
        let mut anchors = HashSet::new();
        for (bi, block) in self.blocks.iter().enumerate() {
            let path = format!("/blocks/{bi}");
            let entries: Vec<_> = match block {
                Block::Dialogue {
                    turns, speakers, ..
                } => {
                    if turns.is_empty() || speakers.is_empty() {
                        return Err(format!("{path}: empty dialogue"));
                    }
                    for speaker in speakers {
                        let cast = self
                            .cast
                            .iter()
                            .find(|c| c.character_id == speaker.character_id)
                            .ok_or_else(|| format!("{path}: unknown character"))?;
                        if cast.display_name != speaker.display_name
                            || cast.avatar_id != speaker.avatar_id
                        {
                            return Err(format!("{path}: speaker differs from pinned character"));
                        }
                    }
                    turns.iter().map(|e| (&e.id, &e.segments)).collect()
                }
                Block::Article { paragraphs, .. } => {
                    if paragraphs.is_empty() {
                        return Err(format!("{path}: empty article"));
                    }
                    paragraphs.iter().map(|e| (&e.id, &e.segments)).collect()
                }
                Block::Exercise { exercise, .. } => {
                    match exercise {
                        Exercise::SingleChoice { prompt_zh, options } => {
                            if prompt_zh.trim().is_empty()
                                || options.len() < 2
                                || options.iter().any(|o| o.text.trim().is_empty())
                            {
                                return Err(format!("{path}: invalid choice"));
                            }
                            unique(options.iter().map(|o| o.id.as_str()), &path)?;
                        }
                        Exercise::Order { prompt_zh, tokens } => {
                            if prompt_zh.trim().is_empty()
                                || tokens.len() < 2
                                || tokens.iter().any(|o| o.text.trim().is_empty())
                            {
                                return Err(format!("{path}: invalid order"));
                            }
                            unique(tokens.iter().map(|o| o.id.as_str()), &path)?;
                        }
                        Exercise::FillBlank {
                            prompt_zh,
                            template_fr,
                            ..
                        } => {
                            if prompt_zh.trim().is_empty()
                                || template_fr.matches("___").count() != 1
                            {
                                return Err(format!("{path}: fill-blank needs one blank"));
                            }
                        }
                    }
                    vec![]
                }
                Block::Vocabulary { entry_ids, .. } | Block::Grammar { entry_ids, .. } => {
                    unique(entry_ids.iter().map(String::as_str), &path)?;
                    vec![]
                }
                _ => vec![],
            };
            unique(entries.iter().map(|(id, _)| id.as_str()), &path)?;
            for (id, segments) in entries {
                if segments.is_empty()
                    || segments
                        .iter()
                        .map(|s| s.text.as_str())
                        .collect::<String>()
                        .trim()
                        .is_empty()
                {
                    return Err(format!("{path}: empty sentence"));
                }
                unique(segments.iter().map(|s| s.id.as_str()), &path)?;
                for segment in segments {
                    anchors.insert((block.id(), id.as_str(), segment.id.as_str()));
                }
            }
        }
        for (bi, block) in self.blocks.iter().enumerate() {
            if let Block::Explanation { targets, .. } = block {
                for (ti, target) in targets.iter().enumerate() {
                    if !anchors.contains(&(
                        target.block_id.as_str(),
                        target.entry_id.as_str(),
                        target.segment_id.as_str(),
                    )) {
                        return Err(format!("/blocks/{bi}/targets/{ti}: unknown reading anchor"));
                    }
                }
            }
        }
        let mut reachable = HashSet::new();
        for (si, step) in self.steps.iter().enumerate() {
            if !matches!(
                step.kind.as_str(),
                "discover" | "read" | "explore" | "practice" | "apply" | "recap"
            ) || step.title_zh.trim().is_empty()
                || step.block_ids.is_empty()
            {
                return Err(format!("/steps/{si}: invalid step"));
            }
            unique(
                step.block_ids.iter().map(String::as_str),
                &format!("/steps/{si}/blockIds"),
            )?;
            reachable.extend(step.block_ids.iter().map(String::as_str));
        }
        if self.blocks.iter().any(|b| !reachable.contains(b.id())) {
            return Err("/steps: unreachable teaching block".into());
        }
        if self.cast.iter().any(|c| {
            c.revision == 0
                || c.display_name.trim().is_empty()
                || c.avatar_id.trim().is_empty()
                || !c.speech_locale.starts_with("fr")
        }) {
            return Err("/cast: invalid character snapshot".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    fn fixture() -> PublicLesson {
        super::super::tests::fixture()
    }
    #[test]
    fn rejects_broken_targets_duplicate_tokens_and_unknown_steps() {
        let mut lesson = fixture();
        if let Block::Explanation { targets, .. } = &mut lesson.blocks[3] {
            targets[0].segment_id = "missing".into();
        }
        assert!(lesson.validate().unwrap_err().contains("targets/0"));
        let mut lesson = fixture();
        lesson.steps[0].kind = "execute-script".into();
        assert!(lesson.validate().is_err());
        let mut lesson = fixture();
        if let Block::Exercise {
            exercise: Exercise::Order { tokens, .. },
            ..
        } = &mut lesson.blocks[9]
        {
            tokens[1].id = tokens[0].id.clone();
        }
        assert!(lesson.validate().is_err());
    }
    #[test]
    fn rejects_unreachable_required_content() {
        let mut lesson = fixture();
        lesson.steps.retain(|s| s.kind != "explore");
        assert!(lesson.validate().unwrap_err().contains("unreachable"));
    }
}
