//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use anyhow::anyhow;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sea_orm::entity::prelude::*;
use tokio::task::spawn_blocking;
use tracing::error;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text", unique)]
    pub username: String,
    #[sea_orm(column_type = "Text")]
    pub password: String,
    pub access_level: i32,
    pub ban_duration: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ban_ip: Option<String>,
}

impl Model {
    pub async fn verify_password(&self, password: &str) -> bool {
        let pwd = password.to_owned();
        let pwd_hash = self.password.clone();
        let res = spawn_blocking(move || {
            let Ok(parsed_hash) = PasswordHash::new(&pwd_hash) else {
                error!("Can not generate a hash for password");
                return false;
            };
            Argon2::default()
                .verify_password(pwd.as_bytes(), &parsed_hash)
                .is_ok()
        })
        .await;
        res.unwrap_or_else(|err| {
            error!("Failed to spawn blocking thread to generate hash: {err}");
            false
        })
    }
    ///
    ///
    /// # Arguments
    ///
    /// * `db_pool`:
    /// * `username`:
    ///
    /// returns: Result<Option<Model>, Error>
    /// # Errors
    pub async fn find_some_by_username(
        db_pool: &DatabaseConnection,
        username: &str,
    ) -> anyhow::Result<Option<Model>> {
        Ok(Entity::find()
            .filter(Column::Username.contains(username))
            .one(db_pool)
            .await?)
    }
    ///
    ///
    /// # Arguments
    ///
    /// * `db_pool`:
    /// * `username`:
    ///
    /// returns: Result<Option<Model>, Error>
    /// # Errors
    pub async fn find_by_username(
        db_pool: &DatabaseConnection,
        username: &str,
    ) -> anyhow::Result<Model> {
        Entity::find()
            .filter(Column::Username.contains(username))
            .one(db_pool)
            .await?
            .ok_or_else(|| anyhow!("User not found {username}"))
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::character::Entity")]
    Character,
}

impl Related<super::character::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Character.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}