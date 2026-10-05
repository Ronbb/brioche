use brioche_course_contract::{
    ApiError, Catalog, CsrfToken, GradeRequest, GradeResult, PublicLesson,
};
use std::{fs, path::Path};
use ts_rs::TS;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/contracts/src/generated");
    fs::create_dir_all(&out)?;
    let config = ts_rs::Config::default().with_out_dir(&out);
    PublicLesson::export_all(&config)?;
    Catalog::export_all(&config)?;
    ApiError::export_all(&config)?;
    GradeRequest::export_all(&config)?;
    GradeResult::export_all(&config)?;
    CsrfToken::export_all(&config)?;
    brioche_course_contract::ReviewQueue::export_all(&config)?;
    brioche_course_contract::ReviewCardsPage::export_all(&config)?;
    brioche_course_contract::ReviewPreferenceRequest::export_all(&config)?;
    brioche_course_contract::ReviewEnrollmentRequest::export_all(&config)?;
    brioche_course_contract::SavedPage::export_all(&config)?;
    brioche_course_contract::SavedWriteRequest::export_all(&config)?;
    brioche_course_contract::ReviewHistoryPage::export_all(&config)?;
    brioche_course_contract::ReviewAttemptRequest::export_all(&config)?;
    brioche_course_contract::ReviewAttemptResult::export_all(&config)?;
    brioche_course_contract::StartLearningRequest::export_all(&config)?;
    brioche_course_contract::LearningWriteRequest::export_all(&config)?;
    brioche_course_contract::SubmitAttemptRequest::export_all(&config)?;
    brioche_course_contract::LearningSession::export_all(&config)?;
    brioche_course_contract::AttemptResult::export_all(&config)?;
    brioche_course_contract::HintResult::export_all(&config)?;
    brioche_course_contract::LearningOverview::export_all(&config)?;
    brioche_course_contract::StudyDashboard::export_all(&config)?;
    brioche_course_contract::UserProfile::export_all(&config)?;
    brioche_course_contract::UpdateProfileRequest::export_all(&config)?;
    brioche_course_contract::AuthResult::export_all(&config)?;
    brioche_course_contract::LoginRequest::export_all(&config)?;
    brioche_course_contract::AcceptInviteRequest::export_all(&config)?;
    brioche_course_contract::ResetPasswordRequest::export_all(&config)?;
    fs::write(
        out.join("public-lesson.schema.json"),
        serde_json::to_string_pretty(&schemars::schema_for!(PublicLesson))?,
    )?;
    Ok(())
}
