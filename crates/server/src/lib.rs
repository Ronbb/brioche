pub mod entity;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use brioche_course_contract::{ApiError, Catalog, Level, PublicLesson, Unit};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
        let rows = entity::Entity::find()
            .filter(entity::Column::Published.eq(true))
            .all(db)
            .await
            .map_err(|_| AppError::Unavailable)?;
        let mut latest = BTreeMap::new();
        for row in rows {
            let lesson: PublicLesson =
                serde_json::from_value(row.public_document).map_err(|_| AppError::Unavailable)?;
            lesson.validate().map_err(|_| AppError::Unavailable)?;
            if latest
                .get(&lesson.id)
                .is_none_or(|old: &PublicLesson| old.revision < lesson.revision)
            {
                latest.insert(lesson.id.clone(), lesson);
            }
        }
        Ok(latest.into_values().collect())
    }
}
pub enum AppError {
    NotFound,
    Unavailable,
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
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
        .fallback(|| async { AppError::NotFound })
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}
async fn ready(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(db) = &state.db {
        db.execute_unprepared("SELECT 1 FROM lesson_revisions LIMIT 0")
            .await
            .map_err(|_| AppError::Unavailable)?;
    } else if state.fixture.is_none() {
        return Err(AppError::Unavailable);
    }
    Ok(Json(serde_json::json!({"status":"ready"})))
}
async fn catalog(State(state): State<Arc<AppState>>) -> Result<Json<Catalog>, AppError> {
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
async fn lesson(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PublicLesson>, AppError> {
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
    let lesson: PublicLesson = serde_json::from_value(source)?;
    lesson.validate().map_err(anyhow::Error::msg)?;
    Ok(lesson)
}
pub fn development_fixture() -> anyhow::Result<PublicLesson> {
    project_source(serde_json::from_str(include_str!(
        "../../../docs/examples/a1-bakery.lesson.json"
    ))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
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
}
