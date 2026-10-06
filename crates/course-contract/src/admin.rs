//! Operator responses deliberately exclude author sources and grading rules.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminOverview {
    pub generation: String,
    pub active_release: Option<String>,
    pub lessons: Vec<AdminLesson>,
    pub releases: Vec<AdminRelease>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminLesson {
    pub id: String,
    pub revision: u32,
    pub title: String,
    pub level: String,
    pub unit: String,
    pub published: bool,
    pub withdrawn: bool,
    pub approved: bool,
    pub review_version: u32,
    pub review_note: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminRelease {
    pub id: String,
    pub lesson_count: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminReviewRequest {
    pub version: u32,
    pub approved: bool,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminActivateRequest {
    pub release_id: String,
    pub generation: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminWithdrawRequest {
    pub generation: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminDocumentRequest {
    pub document: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminImportResult {
    pub lesson_id: String,
    pub revision: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminHistory {
    pub items: Vec<AdminHistoryItem>,
    pub next: Option<AdminHistoryCursor>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminHistoryCursor {
    pub before_time: String,
    pub before_key: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminHistoryItem {
    pub key: String,
    pub action: String,
    pub target: String,
    pub actor: String,
    pub reason: String,
    pub created_at: String,
}
