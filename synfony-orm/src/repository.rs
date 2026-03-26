use sea_orm::{DatabaseConnection, EntityTrait};

use crate::OrmError;

/// The Repository trait — Doctrine's EntityRepository equivalent.
///
/// Provides a standard interface for data access. Each entity gets a repository
/// that can be injected via the DI container.
///
/// # Example
/// ```ignore
/// struct UserRepository {
///     db: Arc<DatabaseConnection>,
/// }
///
/// #[async_trait]
/// impl Repository<user::Entity> for UserRepository {
///     fn connection(&self) -> &DatabaseConnection {
///         &self.db
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Repository<E>: Send + Sync
where
    E: EntityTrait,
{
    /// Get the database connection.
    fn connection(&self) -> &DatabaseConnection;

    /// Find all entities.
    ///
    /// Equivalent to `$repository->findAll()`.
    async fn find_all(&self) -> Result<Vec<E::Model>, OrmError>
    where
        E::Model: Send,
    {
        E::find()
            .all(self.connection())
            .await
            .map_err(OrmError::Database)
    }

    /// Find an entity by its primary key.
    ///
    /// Equivalent to `$repository->find($id)`.
    async fn find_by_id<V>(&self, id: V) -> Result<Option<E::Model>, OrmError>
    where
        V: Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType> + Send,
        E::Model: Send,
    {
        E::find_by_id(id)
            .one(self.connection())
            .await
            .map_err(OrmError::Database)
    }

    /// Find an entity by primary key, returning an error if not found.
    async fn find_or_fail<V>(&self, id: V) -> Result<E::Model, OrmError>
    where
        V: Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType> + Send + std::fmt::Debug + Copy,
        E::Model: Send,
    {
        self.find_by_id(id)
            .await?
            .ok_or_else(|| OrmError::NotFound(format!("{:?}", id)))
    }

    /// Delete an entity by primary key. Returns true if deleted.
    async fn delete_by_id<V>(&self, id: V) -> Result<bool, OrmError>
    where
        V: Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType> + Send,
    {
        let result = E::delete_by_id(id)
            .exec(self.connection())
            .await
            .map_err(OrmError::Database)?;
        Ok(result.rows_affected > 0)
    }
}
