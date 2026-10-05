//! Local, content-addressed visual assets and immutable character snapshots.
use crate::{
    AppError,
    learning::{exec, field, hash, one, random_id},
};
use anyhow::{Context, Result, bail, ensure};
use brioche_course_contract::{Block, Character, MediaAsset, PublicLesson};
use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
const MAX_BYTES: u64 = 32 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetBundle {
    pub schema_version: String,
    pub assets: Vec<AssetSpec>,
    pub characters: Vec<CharacterSpec>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetSpec {
    pub asset_id: String,
    pub revision: u32,
    pub sha256: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub alt_zh: String,
    pub credit_zh: String,
    pub file: String,
    pub status: String,
    pub source: String,
    pub license: String,
    pub creator: String,
    pub rights_confirmed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharacterSpec {
    pub snapshot: Character,
    pub avatar_revision: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetRef {
    pub asset_id: String,
    pub revision: u32,
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 2000 && !value.chars().any(char::is_control)
}
fn extension(mime: &str) -> Result<&'static str> {
    match mime {
        "image/svg+xml" => Ok("svg"),
        "image/png" => Ok("png"),
        "image/jpeg" => Ok("jpg"),
        "image/webp" => Ok("webp"),
        _ => bail!("unsupported visual MIME type"),
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read_file(path: &Path) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).context("asset file unavailable")?;
    ensure!(file.metadata()?.is_file(), "asset must be a regular file");
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    ensure!(
        !bytes.is_empty() && bytes.len() as u64 <= MAX_BYTES,
        "asset size must be 1..32 MiB"
    );
    Ok(bytes)
}
fn svg_dimensions(bytes: &[u8]) -> Result<(u32, u32)> {
    use quick_xml::{Reader, XmlVersion, events::Event};
    let mut reader = Reader::from_str(std::str::from_utf8(bytes)?);
    let (mut depth, mut roots, mut dimensions) = (0usize, 0usize, None);
    let mut namespace = false;
    reader.config_mut().check_comments = true;
    loop {
        match reader.read_event()? {
            event @ (Event::Start(_) | Event::Empty(_)) => {
                let empty = matches!(event, Event::Empty(_));
                let element = match event {
                    Event::Start(e) | Event::Empty(e) => e,
                    _ => unreachable!(),
                };
                let name = element.name();
                let name = name.as_ref();
                ensure!(
                    matches!(
                        name,
                        "svg"
                            | "g"
                            | "path"
                            | "rect"
                            | "circle"
                            | "ellipse"
                            | "line"
                            | "polyline"
                            | "polygon"
                            | "defs"
                            | "linearGradient"
                            | "radialGradient"
                            | "stop"
                            | "title"
                            | "desc"
                            | "clipPath"
                            | "mask"
                    ),
                    "SVG contains unsupported or active element"
                );
                if depth == 0 {
                    ensure!(name == "svg" && roots == 0, "SVG requires one root");
                    roots += 1;
                }
                for attribute in element.attributes() {
                    let attribute = attribute?;
                    let key = attribute.key.as_ref();
                    ensure!(
                        matches!(
                            key,
                            "xmlns"
                                | "viewBox"
                                | "width"
                                | "height"
                                | "x"
                                | "y"
                                | "x1"
                                | "x2"
                                | "y1"
                                | "y2"
                                | "cx"
                                | "cy"
                                | "r"
                                | "rx"
                                | "ry"
                                | "d"
                                | "points"
                                | "fill"
                                | "stroke"
                                | "stroke-width"
                                | "stroke-linecap"
                                | "stroke-linejoin"
                                | "stroke-dasharray"
                                | "fill-rule"
                                | "clip-rule"
                                | "opacity"
                                | "fill-opacity"
                                | "stroke-opacity"
                                | "transform"
                                | "id"
                                | "role"
                                | "aria-labelledby"
                                | "aria-hidden"
                                | "offset"
                                | "stop-color"
                                | "stop-opacity"
                                | "gradientUnits"
                                | "gradientTransform"
                                | "clip-path"
                                | "mask"
                                | "preserveAspectRatio"
                        ),
                        "SVG contains unsupported attribute"
                    );
                    let value = attribute.normalized_value(XmlVersion::Implicit1_0)?;
                    if key == "xmlns" {
                        namespace = true;
                        ensure!(
                            depth == 0 && value == "http://www.w3.org/2000/svg",
                            "SVG namespace must be local and standard"
                        );
                    }
                    let lower = value.to_ascii_lowercase();
                    if matches!(key, "fill" | "stroke" | "clip-path" | "mask")
                        && lower.contains("url")
                    {
                        ensure!(
                            value.starts_with("url(#")
                                && value.ends_with(')')
                                && valid_id(&value[5..value.len() - 1]),
                            "SVG external reference rejected"
                        );
                    }
                    if depth == 0 && key == "viewBox" {
                        let parts = value
                            .split(|c: char| c.is_ascii_whitespace() || c == ',')
                            .filter(|v| !v.is_empty())
                            .map(str::parse::<f64>)
                            .collect::<std::result::Result<Vec<_>, _>>()?;
                        ensure!(
                            parts.len() == 4
                                && parts.iter().all(|v| v.is_finite())
                                && parts[2] > 0.0
                                && parts[3] > 0.0
                                && parts[2].fract() == 0.0
                                && parts[3].fract() == 0.0
                                && parts[2] <= 8192.0
                                && parts[3] <= 8192.0,
                            "SVG viewBox dimensions invalid"
                        );
                        dimensions = Some((parts[2] as u32, parts[3] as u32));
                    }
                }
                if !empty {
                    depth += 1;
                    ensure!(depth <= 128, "SVG nesting too deep");
                }
            }
            Event::End(_) => {
                ensure!(depth > 0, "unexpected SVG closing element");
                depth -= 1;
            }
            Event::DocType(_) | Event::PI(_) | Event::CData(_) | Event::GeneralRef(_) => {
                bail!("SVG entities or executable content rejected")
            }
            Event::Eof => break,
            Event::Text(text) if depth == 0 => {
                ensure!(
                    text.xml_content(XmlVersion::Implicit1_0).trim().is_empty(),
                    "text outside SVG root"
                );
            }
            _ => {}
        }
    }
    ensure!(
        roots == 1 && depth == 0 && namespace,
        "incomplete SVG document or missing SVG namespace"
    );
    dimensions.context("SVG requires viewBox")
}
fn dimensions(bytes: &[u8], mime: &str) -> Result<(u32, u32)> {
    if mime == "image/svg+xml" {
        return svg_dimensions(bytes);
    }
    use image::{GenericImageView, ImageFormat, ImageReader, Limits};
    let format = image::guess_format(bytes)?;
    ensure!(
        matches!(
            (format, mime),
            (ImageFormat::Png, "image/png")
                | (ImageFormat::Jpeg, "image/jpeg")
                | (ImageFormat::WebP, "image/webp")
        ),
        "MIME does not match file bytes"
    );
    let mut reader = ImageReader::with_format(std::io::Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    Ok(reader.decode()?.dimensions())
}
pub fn media_root() -> PathBuf {
    std::env::var_os("MEDIA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".local/media"))
}
fn store_file(root: &Path, bytes: &[u8], sha: &str, ext: &str) -> Result<()> {
    std::fs::create_dir_all(root)?;
    let final_path = root.join(format!("{sha}.{ext}"));
    if final_path.exists() {
        ensure!(
            digest(&stored_bytes(root, sha, ext)?) == sha,
            "existing media object is corrupt"
        );
        return Ok(());
    }
    let temp = root.join(format!(
        "{sha}-{}.tmp",
        random_id().map_err(anyhow::Error::msg)?
    ));
    let result = (|| -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        match std::fs::hard_link(&temp, &final_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => ensure!(
                digest(&stored_bytes(root, sha, ext)?) == sha,
                "concurrent object corrupt"
            ),
            Err(error) => return Err(error.into()),
        };
        Ok(())
    })();
    let _ = std::fs::remove_file(&temp);
    result
}
pub async fn import_bundle(
    db: &DatabaseConnection,
    bundle: AssetBundle,
    source_root: &Path,
    store: &Path,
    actor: &str,
) -> Result<()> {
    ensure!(
        bundle.schema_version == "1.0"
            && bundle.assets.len() <= 500
            && bundle.characters.len() <= 500
            && text(actor),
        "invalid asset bundle"
    );
    let bundle_hash = hash(&bundle).map_err(anyhow::Error::msg)?;
    let source_root = source_root.canonicalize()?;
    let store = store.to_path_buf();
    let assets = bundle.assets.clone();
    // Parsing, hashing, image decoding and filesystem writes never run on the async executor.
    let descriptors =
        tokio::task::spawn_blocking(move || -> Result<Vec<(MediaAsset, String, usize)>> {
            let mut ids = BTreeSet::new();
            let mut descriptors = Vec::new();
            for asset in &assets {
                ensure!(
                    valid_id(&asset.asset_id)
                        && asset.revision > 0
                        && asset.revision <= i32::MAX as u32
                        && ids.insert((&asset.asset_id, asset.revision)),
                    "duplicate or invalid asset identity"
                );
                ensure!(
                    asset.status == "ready"
                        && asset.rights_confirmed
                        && text(&asset.source)
                        && text(&asset.license)
                        && text(&asset.creator)
                        && text(&asset.alt_zh)
                        && text(&asset.credit_zh),
                    "asset needs ready status, rights, credit and alt text"
                );
                ensure!(
                    asset.sha256.len() == 64
                        && asset
                            .sha256
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                    "SHA-256 must be lowercase hex"
                );
                let relative = Path::new(&asset.file);
                ensure!(
                    relative
                        .components()
                        .all(|c| matches!(c, Component::Normal(_))),
                    "asset path must be relative, without traversal"
                );
                let path = source_root.join(relative).canonicalize()?;
                ensure!(
                    path.starts_with(&source_root),
                    "asset escapes source directory"
                );
                let bytes = read_file(&path)?;
                ensure!(digest(&bytes) == asset.sha256, "asset hash mismatch");
                ensure!(
                    dimensions(&bytes, &asset.mime_type)? == (asset.width, asset.height)
                        && asset.width > 0
                        && asset.height > 0,
                    "declared dimensions do not match file"
                );
                let ext = extension(&asset.mime_type)?;
                store_file(&store, &bytes, &asset.sha256, ext)?;
                descriptors.push((
                    MediaAsset {
                        asset_id: asset.asset_id.clone(),
                        revision: asset.revision,
                        sha256: asset.sha256.clone(),
                        mime_type: asset.mime_type.clone(),
                        width: asset.width,
                        height: asset.height,
                        alt_zh: asset.alt_zh.clone(),
                        credit_zh: asset.credit_zh.clone(),
                        url: format!("/api/media/{}.{}", asset.sha256, ext),
                    },
                    ext.to_owned(),
                    bytes.len(),
                ));
            }
            Ok(descriptors)
        })
        .await??;
    let tx = db.begin().await?;
    one(
        &tx,
        "SELECT generation FROM content_state WHERE singleton FOR UPDATE",
        vec![],
    )
    .await
    .map_err(anyhow::Error::msg)?
    .context("content state missing")?;
    for (spec, (descriptor, ext, size)) in bundle.assets.iter().zip(&descriptors) {
        exec(&tx,"INSERT INTO media_assets(asset_id,revision,descriptor,provenance,sha256,extension,byte_size) VALUES($1,$2,$3,$4,$5,$6,$7)",vec![spec.asset_id.clone().into(),(spec.revision as i32).into(),serde_json::to_value(descriptor)?.into(),serde_json::to_value(spec)?.into(),spec.sha256.clone().into(),ext.clone().into(),(*size as i64).into()]).await.map_err(anyhow::Error::msg)?;
    }
    let mut ids = BTreeSet::new();
    for character in &bundle.characters {
        let snapshot = &character.snapshot;
        ensure!(
            valid_id(&snapshot.character_id)
                && snapshot.revision > 0
                && snapshot.revision <= i32::MAX as u32
                && text(&snapshot.display_name)
                && snapshot.speech_locale == "fr-FR"
                && character.avatar_revision > 0
                && character.avatar_revision <= i32::MAX as u32
                && ids.insert((&snapshot.character_id, snapshot.revision)),
            "invalid or duplicate character"
        );
        let asset = one(
            &tx,
            "SELECT descriptor FROM media_assets WHERE asset_id=$1 AND revision=$2",
            vec![
                snapshot.avatar_id.clone().into(),
                (character.avatar_revision as i32).into(),
            ],
        )
        .await
        .map_err(anyhow::Error::msg)?
        .context("character avatar is missing")?;
        let descriptor: MediaAsset =
            serde_json::from_value(field(&asset, "descriptor").map_err(anyhow::Error::msg)?)?;
        ensure!(
            descriptor.width == descriptor.height,
            "avatar must be square"
        );
        exec(&tx,"INSERT INTO character_revisions(character_id,revision,snapshot,avatar_id,avatar_revision) VALUES($1,$2,$3,$4,$5)",vec![snapshot.character_id.clone().into(),(snapshot.revision as i32).into(),serde_json::to_value(snapshot)?.into(),snapshot.avatar_id.clone().into(),(character.avatar_revision as i32).into()]).await.map_err(anyhow::Error::msg)?;
    }
    exec(&tx,"INSERT INTO asset_import_audit(actor,bundle_hash,asset_count,character_count) VALUES($1,$2,$3,$4)",vec![actor.into(),bundle_hash.into(),(bundle.assets.len() as i32).into(),(bundle.characters.len() as i32).into()]).await.map_err(anyhow::Error::msg)?;
    tx.commit().await?;
    Ok(())
}
pub fn source_asset_refs(source: &serde_json::Value) -> Result<Vec<AssetRef>> {
    let Some(refs) = source.get("assetRefs") else {
        return Ok(Vec::new());
    };
    let refs: Vec<AssetRef> = serde_path_to_error::deserialize(refs.clone())
        .context("assetRefs: invalid reference structure")?;
    ensure!(refs.len() <= 500, "assetRefs: too many asset references");
    let mut ids = BTreeSet::new();
    for (index, reference) in refs.iter().enumerate() {
        ensure!(
            valid_id(&reference.asset_id)
                && reference.revision > 0
                && reference.revision <= i32::MAX as u32
                && ids.insert(reference.asset_id.clone()),
            "assetRefs/{index}: invalid or duplicate asset reference"
        );
    }
    Ok(refs)
}

pub async fn hydrate_source<C: ConnectionTrait>(
    db: &C,
    mut source: serde_json::Value,
) -> Result<serde_json::Value> {
    if source.get("assetRefs").is_none() {
        return Ok(source);
    }
    let refs = source_asset_refs(&source)?;
    let mut descriptors = Vec::new();
    for reference in refs {
        let row = one(
            db,
            "SELECT descriptor FROM media_assets WHERE asset_id=$1 AND revision=$2",
            vec![
                reference.asset_id.into(),
                (reference.revision as i32).into(),
            ],
        )
        .await
        .map_err(anyhow::Error::msg)?
        .context("registered asset revision missing")?;
        descriptors
            .push(field::<serde_json::Value>(&row, "descriptor").map_err(anyhow::Error::msg)?);
    }
    source["media"] = serde_json::Value::Array(descriptors);
    Ok(source)
}
pub async fn validate_lesson<C: ConnectionTrait>(
    db: &C,
    lesson: &PublicLesson,
    root: &Path,
) -> Result<(), AppError> {
    let mut ids = BTreeSet::new();
    for asset in &lesson.media {
        if !ids.insert(&asset.asset_id) || asset.revision == 0 || asset.revision > i32::MAX as u32 {
            return Err(AppError::InvalidInput);
        }
        let row = one(
            db,
            "SELECT descriptor FROM media_assets WHERE asset_id=$1 AND revision=$2",
            vec![
                asset.asset_id.clone().into(),
                (asset.revision as i32).into(),
            ],
        )
        .await?
        .ok_or(AppError::InvalidInput)?;
        if field::<serde_json::Value>(&row, "descriptor")?
            != serde_json::to_value(asset).map_err(|_| AppError::Unavailable)?
        {
            return Err(AppError::InvalidInput);
        }
        let root = root.to_path_buf();
        let asset = asset.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let ext = extension(&asset.mime_type)?;
            let bytes = stored_bytes(&root, &asset.sha256, ext)?;
            ensure!(digest(&bytes) == asset.sha256, "stored media hash mismatch");
            Ok(())
        })
        .await
        .map_err(|_| AppError::Unavailable)?
        .map_err(|_| AppError::InvalidInput)?;
    }
    for block in &lesson.blocks {
        if let Block::Scene {
            illustration_id, ..
        } = block
            && !ids.contains(illustration_id)
        {
            return Err(AppError::InvalidInput);
        }
    }
    for character in &lesson.cast {
        if character.revision == 0 || character.revision > i32::MAX as u32 {
            return Err(AppError::InvalidInput);
        }
        let row=one(db,"SELECT snapshot,avatar_revision FROM character_revisions WHERE character_id=$1 AND revision=$2",vec![character.character_id.clone().into(),(character.revision as i32).into()]).await?.ok_or(AppError::InvalidInput)?;
        let avatar_revision = field::<i32>(&row, "avatar_revision")?;
        if field::<serde_json::Value>(&row, "snapshot")?
            != serde_json::to_value(character).map_err(|_| AppError::Unavailable)?
            || !lesson.media.iter().any(|asset| {
                asset.asset_id == character.avatar_id && asset.revision == avatar_revision as u32
            })
        {
            return Err(AppError::InvalidInput);
        }
    }
    Ok(())
}
fn stored_bytes(root: &Path, sha: &str, ext: &str) -> Result<Vec<u8>> {
    let root = root.canonicalize()?;
    let path = root.join(format!("{sha}.{ext}")).canonicalize()?;
    ensure!(path.starts_with(&root), "stored object escapes media root");
    read_file(&path)
}
#[derive(Clone)]
struct MediaState {
    db: DatabaseConnection,
    root: PathBuf,
    permits: std::sync::Arc<tokio::sync::Semaphore>,
}
pub fn router(db: DatabaseConnection, root: PathBuf) -> axum::Router {
    axum::Router::new()
        .route("/api/media/{name}", axum::routing::get(serve))
        .with_state(MediaState {
            db,
            root,
            permits: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
        })
}
async fn serve(
    axum::extract::State(state): axum::extract::State<MediaState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<axum::response::Response, AppError> {
    let Some((sha, ext)) = name.split_once('.') else {
        return Err(AppError::NotFound);
    };
    if sha.len() != 64
        || !sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !matches!(ext, "svg" | "png" | "jpg" | "webp")
    {
        return Err(AppError::NotFound);
    }
    // Staged or unreferenced media are never made public by registration alone.
    let row=one(&state.db,"SELECT descriptor FROM media_assets m WHERE sha256=$1 AND extension=$2 AND EXISTS(SELECT 1 FROM lesson_revisions r WHERE r.published AND r.public_document->'media' @> jsonb_build_array(jsonb_build_object('assetId',m.asset_id,'revision',m.revision))) LIMIT 1",vec![sha.into(),ext.into()]).await?.ok_or(AppError::NotFound)?;
    let descriptor: MediaAsset =
        serde_json::from_value(field(&row, "descriptor")?).map_err(|_| AppError::Unavailable)?;
    let permit = state
        .permits
        .try_acquire_owned()
        .map_err(|_| AppError::Unavailable)?;
    let sha = sha.to_owned();
    let ext = ext.to_owned();
    let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
        let _permit = permit;
        let bytes = stored_bytes(&state.root, &sha, &ext)?;
        ensure!(digest(&bytes) == sha, "media object corrupt");
        Ok(bytes)
    })
    .await
    .map_err(|_| AppError::Unavailable)?
    .map_err(|_| AppError::Unavailable)?;
    axum::response::Response::builder()
        .header("content-type", descriptor.mime_type)
        .header("cache-control", "no-store")
        .header("x-content-type-options", "nosniff")
        .header("cross-origin-resource-policy", "same-origin")
        .header("content-security-policy", "default-src 'none'; sandbox")
        .body(axum::body::Body::from(bytes))
        .map_err(|_| AppError::Unavailable)
}
#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageFormat;
    #[test]
    fn raster_validation_decodes_supported_formats_and_checks_mime() {
        for (format, mime) in [
            (ImageFormat::Png, "image/png"),
            (ImageFormat::Jpeg, "image/jpeg"),
            (ImageFormat::WebP, "image/webp"),
        ] {
            let image = image::DynamicImage::ImageRgb8(image::RgbImage::new(3, 2));
            let mut output = std::io::Cursor::new(Vec::new());
            image.write_to(&mut output, format).unwrap();
            assert_eq!(dimensions(output.get_ref(), mime).unwrap(), (3, 2));
            let wrong_mime = if mime == "image/png" {
                "image/jpeg"
            } else {
                "image/png"
            };
            assert!(dimensions(output.get_ref(), wrong_mime).is_err());
            assert!(dimensions(&output.get_ref()[..8], mime).is_err());
        }
    }
    #[test]
    fn graphics_validation_rejects_active_svg_and_bad_dimensions() {
        assert_eq!(
            svg_dimensions(include_bytes!("../../../apps/web/public/assets/bakery.svg")).unwrap(),
            (640, 470)
        );
        assert_eq!(
            svg_dimensions(include_bytes!(
                "../../../apps/web/public/assets/avatars/camille.svg"
            ))
            .unwrap(),
            (96, 96)
        );
        for bytes in [
            br#"<svg viewBox="0 0 10 10"><script>alert(1)</script></svg>"#.as_slice(),
            br#"<!DOCTYPE svg SYSTEM "http://evil.test"><svg viewBox="0 0 10 10"/>"#,
            br#"<svg viewBox="0 0 10 10"><path onload="alert(1)"/></svg>"#,
            br#"<svg viewBox="0 0 10 10"><path fill="url(https://evil.test)"/></svg>"#,
            br#"<svg viewBox="0 0 NaN 10"/>"#,
            br#"<svg viewBox="0 0 10 10"><g></svg>"#,
            br#"<svg viewBox="0 0 10 10"/><svg viewBox="0 0 10 10"/>"#,
        ] {
            assert!(svg_dimensions(bytes).is_err());
        }
        assert!(dimensions(b"not a PNG", "image/png").is_err());
    }
}
