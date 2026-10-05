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
    brioche_course_contract::UserProfile::export_all(&config)?;
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
