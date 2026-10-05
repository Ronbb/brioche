use sea_orm_migration::prelude::*;
mod m20261006_000001_course_revisions;
mod m20261006_000002_sessions;
mod m20261006_000003_identity;
mod m20261006_000004_profile;
mod m20261006_000005_learning;
pub struct Migrator;
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261006_000001_course_revisions::Migration),
            Box::new(m20261006_000002_sessions::Migration),
            Box::new(m20261006_000003_identity::Migration),
            Box::new(m20261006_000004_profile::Migration),
            Box::new(m20261006_000005_learning::Migration),
        ]
    }
}
