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

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminAccounts {
    pub items: Vec<AdminAccount>,
    pub next_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminAccount {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AdminTokenKind {
    Invite,
    Reset,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminTokenRequest {
    pub email: String,
    pub kind: AdminTokenKind,
    pub operator: bool,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminTokenResult {
    pub token: String,
    pub email: String,
    pub kind: AdminTokenKind,
    pub expires_in_seconds: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AdminAccountRole {
    Learner,
    Operator,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminRoleRequest {
    pub expected_role: AdminAccountRole,
    pub role: AdminAccountRole,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminSessions {
    pub account: AdminAccount,
    pub items: Vec<AdminSession>,
    pub next_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminSession {
    // A SHA-256 record identifier, never a session cookie or reusable login token.
    pub id: String,
    pub expires_at: String,
    pub current: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminRevokeSessionRequest {
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminRevokeSessionResult {
    pub current: bool,
}

/// Private authoring data, never embedded in public lesson snapshots.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterVoiceProfile {
    pub personality: String,
    pub speaking_style: String,
    pub default_emotion: String,
    pub provider: String,
    pub model: String,
    pub voice_id: String,
    pub voice_kind: String,
    pub locale: String,
    pub rate: f64,
    pub reference_audio: Option<CharacterVoiceReference>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterVoiceReference {
    pub asset_id: String,
    pub revision: u32,
    pub transcript: String,
    // Provenance and consent for cloning, separate from ordinary playback rights.
    pub cloning_permission: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminCharacterVoices {
    pub items: Vec<AdminCharacterVoice>,
    pub next_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminCharacterVoice {
    pub character: crate::Character,
    pub avatar_revision: u32,
    pub voice_revision: u32,
    pub profile: Option<CharacterVoiceProfile>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminCharacterVoiceRequest {
    pub character_id: String,
    pub character_revision: u32,
    pub expected_voice_revision: u32,
    pub profile: CharacterVoiceProfile,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminPendingTokens {
    pub items: Vec<AdminPendingToken>,
    pub next_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminPendingToken {
    // An opaque management identifier, neither a token nor its authentication hash.
    pub id: String,
    pub email: String,
    pub kind: AdminTokenKind,
    pub role: AdminAccountRole,
    pub expires_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminRevokeTokenRequest {
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminAssets {
    pub items: Vec<AdminAsset>,
    pub next: Option<AdminAssetCursor>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminAssetCursor {
    pub asset_id: String,
    pub revision: u32,
}
/// Explicit private projection: never return local import filenames or the raw provenance blob.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminAsset {
    pub asset: crate::MediaAsset,
    pub source: String,
    pub license: String,
    pub creator: String,
    pub rights_confirmed: bool,
    pub byte_size: u32,
}
