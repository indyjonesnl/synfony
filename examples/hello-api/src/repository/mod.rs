use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use synfony_orm::{OrmError, Repository};

use crate::dto::CreateUserDto;
use crate::entity::user;

/// User repository — Doctrine-style repository for User entities.
pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Arc<Self> {
        Arc::new(UserRepository { db })
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<user::Model>, OrmError> {
        user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&*self.db)
            .await
            .map_err(OrmError::Database)
    }

    pub async fn create(&self, dto: &CreateUserDto) -> Result<user::Model, OrmError> {
        let model = user::ActiveModel {
            name: Set(dto.name.clone()),
            email: Set(dto.email.clone()),
            role: Set("ROLE_USER".to_string()),
            ..Default::default()
        };

        model
            .insert(&*self.db)
            .await
            .map_err(OrmError::Database)
    }
}

#[async_trait::async_trait]
impl Repository<user::Entity> for UserRepository {
    fn connection(&self) -> &DatabaseConnection {
        &self.db
    }
}
