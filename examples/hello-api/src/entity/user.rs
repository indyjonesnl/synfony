//! User entity — equivalent to a Doctrine Entity.
//!
//! Symfony equivalent:
//! ```php
//! #[ORM\Entity(repositoryClass: UserRepository::class)]
//! #[ORM\Table(name: "users")]
//! class User {
//!     #[ORM\Id]
//!     #[ORM\GeneratedValue]
//!     #[ORM\Column]
//!     private ?int $id = null;
//!
//!     #[ORM\Column(length: 100)]
//!     private string $name;
//!
//!     #[ORM\Column(length: 255, unique: true)]
//!     private string $email;
//! }
//! ```

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub role: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
