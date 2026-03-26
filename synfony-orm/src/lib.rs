//! # Synfony ORM
//!
//! Doctrine-inspired ORM integration for Synfony, backed by SeaORM.

pub mod connection;
mod entity_manager;
mod error;
mod repository;

pub use connection::{DatabaseConfig, connect};
pub use entity_manager::EntityManager;
pub use error::OrmError;
pub use repository::Repository;

// Re-export SeaORM essentials
pub use sea_orm::{
    self, ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection,
    DbErr, DeriveEntityModel, DerivePrimaryKey, DeriveRelation, EntityTrait, EnumIter,
    IntoActiveModel, ModelTrait, PaginatorTrait, PrimaryKeyTrait, QueryFilter, QueryOrder,
    QuerySelect, Related, RelationTrait, Set,
};
pub use sea_orm_migration::{self, MigratorTrait};
