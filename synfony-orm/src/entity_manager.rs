use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel};
use std::sync::Arc;

use crate::OrmError;

/// The EntityManager — Doctrine's EntityManagerInterface equivalent.
///
/// Provides persist/remove operations. In SeaORM, each operation
/// is immediately executed (no unit-of-work flush).
#[derive(Clone)]
pub struct EntityManager {
    db: Arc<DatabaseConnection>,
}

impl EntityManager {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        EntityManager { db }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Persist (insert) an active model.
    ///
    /// Equivalent to `$em->persist($entity); $em->flush();`
    pub async fn persist<A>(&self, model: A) -> Result<<A::Entity as EntityTrait>::Model, OrmError>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send,
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A> + Send,
    {
        model
            .insert(&*self.db)
            .await
            .map_err(OrmError::Database)
    }

    /// Update an existing active model.
    pub async fn update<A>(&self, model: A) -> Result<<A::Entity as EntityTrait>::Model, OrmError>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send,
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A> + Send,
    {
        model
            .update(&*self.db)
            .await
            .map_err(OrmError::Database)
    }

    /// Remove (delete) a model.
    pub async fn remove<A, M>(&self, model: M) -> Result<(), OrmError>
    where
        A: ActiveModelTrait + ActiveModelBehavior + Send,
        M: IntoActiveModel<A> + Send,
    {
        let active = model.into_active_model();
        active.delete(&*self.db).await.map_err(OrmError::Database)?;
        Ok(())
    }
}
