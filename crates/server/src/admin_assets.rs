//! Private, paginated visual registry; registration does not publish a file.
use crate::{
    AppError,
    identity::{AuthSession, Backend, require_operator},
    learning::{field, one},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use brioche_course_contract::{AdminAsset, AdminAssetCursor, AdminAssets, MediaAsset};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

pub fn router() -> Router<Backend> {
    Router::new()
        .route("/api/v1/operator/assets", get(list))
        .route("/api/v1/operator/assets/{id}/{revision}/file", get(file))
}
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssetQuery {
    after_id: Option<String>,
    after_revision: Option<u32>,
    q: Option<String>,
}
impl AssetQuery {
    fn validate(&self) -> Result<(), AppError> {
        match (&self.after_id, self.after_revision) {
            (None, None) => {}
            (Some(id), Some(revision)) if valid(id, revision) => {}
            _ => return Err(AppError::InvalidInput),
        }
        if self
            .q
            .as_ref()
            .is_some_and(|q| q.len() > 200 || q.chars().any(char::is_control))
        {
            return Err(AppError::InvalidInput);
        }
        Ok(())
    }
}
fn valid(id: &str, revision: u32) -> bool {
    brioche_course_contract::valid_content_id(id)
        && brioche_course_contract::valid_content_revision(revision)
}
async fn list(
    auth: AuthSession,
    State(backend): State<Backend>,
    Query(query): Query<AssetQuery>,
) -> Result<Json<AdminAssets>, AppError> {
    require_operator(&auth)?;
    query.validate()?;
    let rows = backend.db.query_all_raw(Statement::from_sql_and_values(DbBackend::Postgres, r#"
        SELECT descriptor,provenance->>'source' AS source,provenance->>'license' AS license,
        provenance->>'creator' AS creator,(provenance->>'rightsConfirmed')::boolean AS rights_confirmed,byte_size
        FROM media_assets
        WHERE (asset_id,revision)>($1,$2)
        AND ($3='' OR strpos(lower(asset_id),lower($3))>0 OR strpos(lower(descriptor->>'altZh'),lower($3))>0)
        ORDER BY asset_id,revision LIMIT 21
    "#, vec![query.after_id.unwrap_or_default().into(), (query.after_revision.unwrap_or(0) as i32).into(),query.q.unwrap_or_default().trim().to_owned().into()])).await.map_err(|_|AppError::Unavailable)?;
    let more = rows.len() > 20;
    let items = rows
        .into_iter()
        .take(20)
        .map(|row| -> Result<AdminAsset, AppError> {
            let mut asset: MediaAsset = serde_json::from_value(field(&row, "descriptor")?)
                .map_err(|_| AppError::Unavailable)?;
            asset.url = format!(
                "/api/v1/operator/assets/{}/{}/file",
                asset.asset_id, asset.revision
            );
            Ok(AdminAsset {
                asset,
                source: field(&row, "source")?,
                license: field(&row, "license")?,
                creator: field(&row, "creator")?,
                rights_confirmed: field(&row, "rights_confirmed")?,
                byte_size: u32::try_from(field::<i64>(&row, "byte_size")?)
                    .map_err(|_| AppError::Unavailable)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let next = if more {
        items.last().map(|i| AdminAssetCursor {
            asset_id: i.asset.asset_id.clone(),
            revision: i.asset.revision,
        })
    } else {
        None
    };
    Ok(Json(AdminAssets { items, next }))
}
async fn file(
    auth: AuthSession,
    State(backend): State<Backend>,
    Path((id, revision)): Path<(String, u32)>,
    axum::Extension(root): axum::Extension<std::path::PathBuf>,
    axum::Extension(permits): axum::Extension<std::sync::Arc<tokio::sync::Semaphore>>,
) -> Result<axum::response::Response, AppError> {
    require_operator(&auth)?;
    if !valid(&id, revision) {
        return Err(AppError::InvalidInput);
    }
    let row = one(
        &backend.db,
        "SELECT descriptor FROM media_assets WHERE asset_id=$1 AND revision=$2",
        vec![id.into(), (revision as i32).into()],
    )
    .await?
    .ok_or(AppError::NotFound)?;
    let descriptor =
        serde_json::from_value(field(&row, "descriptor")?).map_err(|_| AppError::Unavailable)?;
    crate::media::asset_response(root, descriptor, permits).await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursors_are_pairs_with_bounded_search() {
        assert!(AssetQuery::default().validate().is_ok());
        assert!(
            AssetQuery {
                after_id: Some("art-bakery".into()),
                after_revision: Some(1),
                q: Some("%_'".into())
            }
            .validate()
            .is_ok()
        );
        for q in [
            AssetQuery {
                after_id: Some("art-bakery".into()),
                ..Default::default()
            },
            AssetQuery {
                after_revision: Some(1),
                ..Default::default()
            },
            AssetQuery {
                after_id: Some("../file".into()),
                after_revision: Some(1),
                ..Default::default()
            },
            AssetQuery {
                q: Some("x".repeat(201)),
                ..Default::default()
            },
            AssetQuery {
                q: Some("\n".into()),
                ..Default::default()
            },
        ] {
            assert!(q.validate().is_err());
        }
    }
}
