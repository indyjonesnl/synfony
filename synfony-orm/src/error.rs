use sea_orm::DbErr;

#[derive(Debug, thiserror::Error)]
pub enum OrmError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Migration error: {0}")]
    Migration(String),
}
