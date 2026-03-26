use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The security token — represents an authenticated user session.
///
/// Equivalent to Symfony's `TokenInterface`. Contains the user identity,
/// roles, and optional custom attributes. Stored in Axum request extensions
/// after successful authentication.
///
/// # Example
/// ```ignore
/// let token = SecurityToken::new("user-123", "alice@example.com")
///     .with_role("ROLE_USER")
///     .with_role("ROLE_ADMIN")
///     .with_attribute("tenant_id", "acme");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityToken {
    /// Unique user identifier (e.g., database ID, UUID)
    user_id: String,

    /// User display identifier (e.g., email, username)
    user_identifier: String,

    /// Set of roles (e.g., ROLE_USER, ROLE_ADMIN)
    roles: HashSet<String>,

    /// Custom attributes attached to the token
    attributes: std::collections::HashMap<String, String>,
}

impl SecurityToken {
    pub fn new(user_id: impl Into<String>, user_identifier: impl Into<String>) -> Self {
        let mut roles = HashSet::new();
        roles.insert("ROLE_USER".to_string()); // Every authenticated user has ROLE_USER

        SecurityToken {
            user_id: user_id.into(),
            user_identifier: user_identifier.into(),
            roles,
            attributes: std::collections::HashMap::new(),
        }
    }

    /// Add a role to the token.
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.insert(role.into());
        self
    }

    /// Add multiple roles.
    pub fn with_roles(mut self, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for role in roles {
            self.roles.insert(role.into());
        }
        self
    }

    /// Add a custom attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Get the user ID.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Get the user identifier (email, username, etc.).
    pub fn user_identifier(&self) -> &str {
        &self.user_identifier
    }

    /// Get all roles.
    pub fn roles(&self) -> &HashSet<String> {
        &self.roles
    }

    /// Check if the token has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// Get a custom attribute.
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }

    /// Get all attributes.
    pub fn attributes(&self) -> &std::collections::HashMap<String, String> {
        &self.attributes
    }
}
