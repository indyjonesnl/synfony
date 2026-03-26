use serde::{Deserialize, Serialize};
use synfony_serializer::SerializeGroups;
use validator::Validate;

/// User response DTO with serialization groups.
///
/// Symfony equivalent:
/// ```php
/// class UserDto {
///     #[Groups(["list", "detail"])]
///     public int $id;
///     #[Groups(["list", "detail"])]
///     public string $name;
///     #[Groups(["detail"])]
///     public string $email;
///     #[Groups(["admin"])]
///     public string $role;
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, SerializeGroups)]
pub struct UserDto {
    #[groups("list", "detail", "admin")]
    pub id: i32,
    #[groups("list", "detail", "admin")]
    pub name: String,
    #[groups("detail", "admin")]
    pub email: String,
    #[groups("admin")]
    pub role: String,
}

/// Request DTO for creating a user — with validation constraints.
///
/// Symfony equivalent:
/// ```php
/// class CreateUserDto {
///     #[Assert\NotBlank]
///     #[Assert\Length(min: 2, max: 100)]
///     public string $name;
///
///     #[Assert\NotBlank]
///     #[Assert\Email]
///     public string $email;
/// }
/// ```
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,
}

impl UserDto {
    pub fn from_model(model: &crate::entity::user::Model) -> Self {
        UserDto {
            id: model.id,
            name: model.name.clone(),
            email: model.email.clone(),
            role: model.role.clone(),
        }
    }
}
