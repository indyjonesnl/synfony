//! # Synfony Serializer
//!
//! Serialization groups for the Synfony framework.
//!
//! Provides Symfony-style `#[Groups]` support for serde, allowing you to
//! control which fields are included in the response based on context.
//!
//! ## Example
//! ```ignore
//! #[derive(Serialize, SerializeGroups)]
//! struct UserDto {
//!     #[groups("list", "detail")]
//!     id: i32,
//!     #[groups("list", "detail")]
//!     name: String,
//!     #[groups("detail")]
//!     email: String,
//!     #[groups("admin")]
//!     internal_notes: String,
//! }
//!
//! // In a handler:
//! async fn list() -> GroupedJson<Vec<UserDto>> {
//!     let users = get_users();
//!     GroupedJson::new(users, "list")
//! }
//! ```

mod grouped_json;

pub use grouped_json::GroupedJson;
pub use synfony_serializer_macros::SerializeGroups;
