use sea_orm::DatabaseConnection;
pub async fn fixture_assets(db: &DatabaseConnection, name: &str) -> std::path::PathBuf {
    use brioche_server::media::{AssetBundle, AssetSpec, CharacterSpec};
    use sha2::{Digest, Sha256};
    let root = std::env::temp_dir().join(format!("brioche-media-{name}"));
    std::fs::create_dir(&root).unwrap();
    let source =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/public/assets");
    let entries = [
        ("art-bakery-morning", "bakery.svg", 640, 470),
        ("avatar-camille-v1", "avatars/camille.svg", 96, 96),
        ("avatar-luc-v1", "avatars/luc.svg", 96, 96),
        ("avatar-lea-v1", "avatars/lea.svg", 96, 96),
    ];
    let assets = entries
        .into_iter()
        .map(|(id, file, width, height)| AssetSpec {
            asset_id: id.into(),
            revision: 1,
            sha256: format!(
                "{:x}",
                Sha256::digest(std::fs::read(source.join(file)).unwrap())
            ),
            mime_type: "image/svg+xml".into(),
            width,
            height,
            alt_zh: "测试素材".into(),
            credit_zh: "仅隔离协议测试".into(),
            file: file.into(),
            status: "ready".into(),
            source: "test:repository-svg".into(),
            license: "LicenseRef-TestOnly".into(),
            creator: "test fixture".into(),
            rights_confirmed: true,
        })
        .collect();
    let lesson = brioche_server::development_fixture().unwrap();
    let characters = lesson
        .cast
        .into_iter()
        .map(|snapshot| CharacterSpec {
            snapshot,
            avatar_revision: 1,
        })
        .collect();
    brioche_server::media::import_bundle(
        db,
        AssetBundle {
            schema_version: "1.0".into(),
            assets,
            characters,
        },
        &source,
        &root,
        "protocol-test",
    )
    .await
    .unwrap();
    root
}
pub fn fixture_refs() -> serde_json::Value {
    serde_json::json!([{"assetId":"art-bakery-morning","revision":1},{"assetId":"avatar-camille-v1","revision":1},{"assetId":"avatar-luc-v1","revision":1},{"assetId":"avatar-lea-v1","revision":1}])
}
