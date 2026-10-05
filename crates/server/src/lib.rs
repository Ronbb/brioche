pub mod author_json;
pub mod content;
pub mod csrf;
pub mod dashboard;
pub mod entity;
pub mod grading;
pub mod identity;
pub mod learning;
pub mod library;
pub mod media;
pub mod observability;
pub mod password;
pub mod reviews;
pub mod session_store;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use brioche_course_contract::{
    ApiError, Catalog, GradeRequest, GradeResult, Level, PublicLesson, Unit,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use std::{collections::BTreeMap, sync::Arc};

pub struct AppState {
    pub db: Option<DatabaseConnection>,
    pub fixture: Option<PublicLesson>,
}
impl AppState {
    async fn lessons(&self) -> Result<Vec<PublicLesson>, AppError> {
        if let Some(lesson) = &self.fixture {
            return Ok(vec![lesson.clone()]);
        }
        let db = self.db.as_ref().ok_or(AppError::Unavailable)?;
        let rows = db.query_all_raw(Statement::from_string(DbBackend::Postgres,
            "SELECT r.public_document FROM content_state s JOIN release_entries e ON e.release_id=s.active_release JOIN lesson_revisions r ON (r.lesson_id,r.revision)=(e.lesson_id,e.revision) WHERE s.singleton AND r.published ORDER BY e.position"
        )).await.map_err(|_| AppError::Unavailable)?;
        let mut lessons = Vec::new();
        for row in rows {
            let lesson: PublicLesson =
                serde_json::from_value(learning::field(&row, "public_document")?)
                    .map_err(|_| AppError::Unavailable)?;
            lesson.validate().map_err(|_| AppError::Unavailable)?;
            lessons.push(lesson);
        }
        Ok(lessons)
    }
}
#[derive(Debug)]
pub enum AppError {
    InvalidInput,
    Unauthorized,
    RateLimited,
    NotFound,
    Unavailable,
    InvalidAnswer,
    Forbidden,
    Conflict,
    Gone,
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request failed")
    }
}
impl std::error::Error for AppError {}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Gone => (
                StatusCode::GONE,
                "content_withdrawn",
                "课程已撤回，暂时无法继续学习",
            ),
            Self::InvalidInput => (StatusCode::BAD_REQUEST, "invalid_input", "请检查填写的信息"),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "登录信息无效或已过期",
            ),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "请求较多，请稍后重试",
            ),
            Self::InvalidAnswer => (
                StatusCode::BAD_REQUEST,
                "invalid_answer",
                "请检查答案后重试",
            ),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "请求来源无法验证"),
            Self::Conflict => (
                StatusCode::CONFLICT,
                "revision_conflict",
                "课程版本已变化，请重新打开课程",
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "课程不存在"),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                "服务暂时不可用",
            ),
        };
        (
            status,
            [("Cache-Control", "no-store")],
            Json(ApiError {
                code: code.into(),
                message: message.into(),
            }),
        )
            .into_response()
    }
}
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/ready", get(ready))
        .route("/api/catalog", get(catalog))
        .route("/api/lessons/{id}", get(lesson))
        .route("/api/demo/lessons/{id}/grade", post(demo_grade))
        .fallback(|| async { AppError::NotFound })
        .layer(axum::extract::DefaultBodyLimit::max(16 * 1024))
        .with_state(Arc::new(state))
}
/// Development only, stateless grading. Production learning submissions require authenticated sessions.
async fn demo_grade(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<GradeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let lesson = state
        .fixture
        .as_ref()
        .filter(|l| l.id == id)
        .ok_or(AppError::NotFound)?;
    let origin = headers
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .and_then(|origin| origin.parse::<axum::http::Uri>().ok())
        .ok_or(AppError::Forbidden)?;
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Forbidden)?;
    if !matches!(origin.scheme_str(), Some("http" | "https"))
        || origin.authority().map(|a| a.as_str()) != Some(host)
        || origin.path() != "/"
    {
        return Err(AppError::Forbidden);
    }
    if request.revision != lesson.revision {
        return Err(AppError::Conflict);
    }
    let source = development_source().map_err(|_| AppError::Unavailable)?;
    let grader =
        grading::Grader::from_source(lesson, &source).map_err(|_| AppError::Unavailable)?;
    let result: GradeResult = grader
        .grade(lesson, &request.exercise_id, &request.answer)
        .map_err(|error| match error {
            grading::GradeError::UnknownExercise => AppError::NotFound,
            grading::GradeError::InvalidAnswer => AppError::InvalidAnswer,
            grading::GradeError::InvalidContent => AppError::Unavailable,
        })?;
    Ok(([("Cache-Control", "no-store")], Json(result)))
}
async fn ready(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(db) = &state.db {
        db.execute_unprepared("SELECT users.profile_version FROM lesson_revisions, users, browser_sessions, identity_tokens, auth_throttle, learning_sessions, review_cards, review_attempts, saved_items, content_state, content_releases, content_withdrawals, media_assets, character_revisions, asset_import_audit LIMIT 0")
            .await
            .map_err(|_| AppError::Unavailable)?;
    } else if state.fixture.is_none() {
        return Err(AppError::Unavailable);
    }
    Ok(Json(serde_json::json!({"status":"ready"})))
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogQuery {
    q: Option<String>,
}
async fn catalog(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<Catalog>, AppError> {
    let query = content::search_terms(query.q.as_deref().unwrap_or(""))?;
    let Json(catalog) = catalog_all(State(state)).await?;
    Ok(Json(content::search_catalog(catalog, &query)))
}
async fn catalog_all(State(state): State<Arc<AppState>>) -> Result<Json<Catalog>, AppError> {
    if let Some(db) = &state.db {
        return content::catalog(db).await.map(Json);
    }
    let mut levels: BTreeMap<String, BTreeMap<String, Vec<_>>> = BTreeMap::new();
    for lesson in state.lessons().await? {
        levels
            .entry(lesson.level_id.clone())
            .or_default()
            .entry(lesson.unit_id.clone())
            .or_default()
            .push(lesson.summary());
    }
    Ok(Json(Catalog {
        development_fixture: state.fixture.is_some(),
        levels: levels
            .into_iter()
            .map(|(id, units)| Level {
                label: id.to_uppercase(),
                id,
                units: units
                    .into_iter()
                    .map(|(id, lessons)| Unit {
                        title_zh: if id == "a1-breakfast-bakery" {
                            "早餐与面包店".into()
                        } else {
                            id.clone()
                        },
                        id,
                        lessons,
                    })
                    .collect(),
            })
            .collect(),
    }))
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LessonQuery {
    revision: Option<u32>,
}
async fn lesson(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<LessonQuery>,
) -> Result<Json<PublicLesson>, AppError> {
    if let Some(revision) = query.revision {
        if revision == 0 || revision > i32::MAX as u32 {
            return Err(AppError::InvalidInput);
        }
        if let Some(fixture) = &state.fixture {
            return if fixture.id == id && fixture.revision == revision {
                Ok(Json(fixture.clone()))
            } else {
                Err(AppError::NotFound)
            };
        }
        let db = state.db.as_ref().ok_or(AppError::Unavailable)?;
        let row=learning::one(db,"SELECT public_document,published,EXISTS(SELECT 1 FROM content_withdrawals w WHERE (w.lesson_id,w.revision)=(r.lesson_id,r.revision)) AS withdrawn FROM lesson_revisions r WHERE lesson_id=$1 AND revision=$2",vec![id.into(),(revision as i32).into()]).await?.ok_or(AppError::NotFound)?;
        if !learning::field::<bool>(&row, "published")? {
            return Err(if learning::field::<bool>(&row, "withdrawn")? {
                AppError::Gone
            } else {
                AppError::NotFound
            });
        }
        let lesson: PublicLesson =
            serde_json::from_value(learning::field(&row, "public_document")?)
                .map_err(|_| AppError::Unavailable)?;
        lesson.validate().map_err(|_| AppError::Unavailable)?;
        return Ok(Json(lesson));
    }
    state
        .lessons()
        .await?
        .into_iter()
        .find(|lesson| lesson.id == id)
        .map(Json)
        .ok_or(AppError::NotFound)
}
pub fn project_source(mut source: serde_json::Value) -> anyhow::Result<PublicLesson> {
    let object = source
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("lesson must be an object"))?;
    object.remove("serverOnly");
    object.remove("editorial");
    object.remove("assetRefs");
    let lesson: PublicLesson = serde_json::from_value(source)?;
    lesson.validate().map_err(anyhow::Error::msg)?;
    Ok(lesson)
}
pub fn development_source() -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::from_str(include_str!(
        "../../../docs/examples/a1-bakery.lesson.json"
    ))?)
}
pub fn development_fixture() -> anyhow::Result<PublicLesson> {
    project_source(development_source()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    #[tokio::test]
    async fn catalog_search_matches_scenes_normalizes_french_and_bounds_queries() {
        let app = router(AppState {
            db: None,
            fixture: Some(development_fixture().unwrap()),
        });
        for (query, status, count) in [
            ("", StatusCode::OK, 1),
            ("?q=%E9%9D%A2%E5%8C%85%E5%BA%97", StatusCode::OK, 1),
            ("?q=BOULANGERIE%20matin", StatusCode::OK, 1),
            (
                "?q=%EF%BD%82%EF%BD%8F%EF%BD%95%EF%BD%8C%EF%BD%81%EF%BD%8E%EF%BD%87%EF%BD%85%EF%BD%92%EF%BD%89%EF%BD%85",
                StatusCode::OK,
                1,
            ),
            ("?q=boulangerie%20introuvable", StatusCode::OK, 0),
            ("?q=%25", StatusCode::OK, 0),
            ("?q=%00", StatusCode::BAD_REQUEST, 0),
        ] {
            let response = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .uri(format!("/api/catalog{query}"))
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status);
            if status == StatusCode::OK {
                let catalog: Catalog = serde_json::from_slice(
                    &response.into_body().collect().await.unwrap().to_bytes(),
                )
                .unwrap();
                assert_eq!(
                    catalog
                        .levels
                        .iter()
                        .flat_map(|l| &l.units)
                        .flat_map(|u| &u.lessons)
                        .count(),
                    count
                );
                assert!(catalog.development_fixture);
            }
        }
        assert!(content::search_terms(&"a".repeat(121)).is_err());
        assert_eq!(
            content::search_terms(" CAFÉ  café\u{301} ").unwrap(),
            vec!["cafe", "cafe"]
        );
    }
    #[tokio::test]
    async fn public_api_no_answers() {
        let app = router(AppState {
            db: None,
            fixture: Some(development_fixture().unwrap()),
        });
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/lessons/a1-bakery-buy-breakfast")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        for key in [
            "serverOnly",
            "correctOptionId",
            "correctTokenIds",
            "editorial",
        ] {
            assert!(!text.contains(key));
        }
    }
    #[tokio::test]
    async fn readiness_without_backend_fails() {
        let response = router(AppState {
            db: None,
            fixture: None,
        })
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/ready")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn demo_grading_checks_origin_version_and_payload_without_exposing_keys() {
        for (origin, revision, answer, expected) in [
            (
                Some("http://localhost:5173"),
                1,
                serde_json::json!({"kind":"choice","optionId":"request-bread"}),
                StatusCode::OK,
            ),
            (
                Some("https://evil.example"),
                1,
                serde_json::json!({"kind":"choice","optionId":"request-bread"}),
                StatusCode::FORBIDDEN,
            ),
            (
                None,
                1,
                serde_json::json!({"kind":"choice","optionId":"request-bread"}),
                StatusCode::FORBIDDEN,
            ),
            (
                Some("http://localhost:5173"),
                2,
                serde_json::json!({"kind":"choice","optionId":"request-bread"}),
                StatusCode::CONFLICT,
            ),
            (
                Some("http://localhost:5173"),
                1,
                serde_json::json!({"kind":"choice","optionId":"missing"}),
                StatusCode::BAD_REQUEST,
            ),
            (
                Some("http://localhost:5173"),
                1,
                serde_json::json!({"kind":"choice","optionId":"request-bread","score":100}),
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
        ] {
            let mut request = axum::http::Request::builder()
                .method("POST")
                .uri("/api/demo/lessons/a1-bakery-buy-breakfast/grade")
                .header("host", "localhost:5173")
                .header("content-type", "application/json");
            if let Some(origin) = origin {
                request = request.header("origin", origin);
            }
            let response = router(AppState {
                db: None,
                fixture: Some(development_fixture().unwrap()),
            })
            .oneshot(
                request
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&serde_json::json!({
                            "revision":revision,"exerciseId":"exercise-intention","answer":answer
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
            assert_eq!(response.status(), expected);
            if expected == StatusCode::OK {
                assert_eq!(response.headers()["cache-control"], "no-store");
                let bytes = response.into_body().collect().await.unwrap().to_bytes();
                let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(json["correct"], true);
                assert!(json.get("correctOptionId").is_none());
            }
        }
        let response = router(AppState { db: None, fixture: None }).oneshot(
            axum::http::Request::builder().method("POST").uri("/api/demo/lessons/a1-bakery-buy-breakfast/grade")
                .header("content-type","application/json")
                .body(axum::body::Body::from(r#"{"revision":1,"exerciseId":"exercise-intention","answer":{"kind":"choice","optionId":"request-bread"}}"#)).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
