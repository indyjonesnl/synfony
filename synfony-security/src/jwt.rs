//! JWT authentication support.
//!
//! Provides a ready-to-use JWT authenticator and token generation utilities,
//! similar to `lexik/jwt-authentication-bundle` in Symfony.

use axum::http::request::Parts;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::authenticator::Authenticator;
use crate::error::AuthError;
use crate::token::SecurityToken;

/// JWT claims payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User identifier (email, username)
    pub identifier: String,
    /// Roles
    pub roles: Vec<String>,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp)
    pub exp: i64,
    /// Custom attributes
    #[serde(default, flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Configuration for the JWT authenticator.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for HMAC signing
    pub secret: String,
    /// Token time-to-live in seconds (default: 3600 = 1 hour)
    pub ttl: i64,
    /// Algorithm (default: HS256)
    pub algorithm: jsonwebtoken::Algorithm,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        JwtConfig {
            secret: secret.into(),
            ttl: 3600,
            algorithm: jsonwebtoken::Algorithm::HS256,
        }
    }

    pub fn with_ttl(mut self, ttl_seconds: i64) -> Self {
        self.ttl = ttl_seconds;
        self
    }
}

/// JWT token manager — generates and validates JWT tokens.
///
/// Equivalent to `Lexik\Bundle\JWTAuthenticationBundle\Services\JWTTokenManagerInterface`.
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
        JwtManager {
            config,
            encoding_key,
            decoding_key,
        }
    }

    /// Generate a JWT token for the given security token.
    pub fn generate(&self, token: &SecurityToken) -> Result<String, AuthError> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: token.user_id().to_string(),
            identifier: token.user_identifier().to_string(),
            roles: token.roles().iter().cloned().collect(),
            iat: now,
            exp: now + self.config.ttl,
            extra: token
                .attributes()
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        };

        encode(
            &Header::new(self.config.algorithm),
            &claims,
            &self.encoding_key,
        )
        .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    /// Validate a JWT token string and return the claims.
    pub fn validate(&self, token_str: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(self.config.algorithm);
        validation.set_required_spec_claims(&["sub", "exp", "iat"]);

        decode::<Claims>(token_str, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(e.to_string()),
            })
    }
}

/// JWT Authenticator — extracts and validates JWT tokens from the Authorization header.
///
/// Expects: `Authorization: Bearer <token>`
///
/// Equivalent to Symfony's JWT authenticator from `lexik/jwt-authentication-bundle`.
pub struct JwtAuthenticator {
    manager: JwtManager,
}

impl JwtAuthenticator {
    pub fn new(config: JwtConfig) -> Self {
        JwtAuthenticator {
            manager: JwtManager::new(config),
        }
    }

    /// Get a reference to the JWT manager (useful for generating tokens in login endpoints).
    pub fn manager(&self) -> &JwtManager {
        &self.manager
    }

    fn extract_bearer_token(parts: &Parts) -> Option<&str> {
        parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
    }
}

#[async_trait::async_trait]
impl Authenticator for JwtAuthenticator {
    fn supports(&self, parts: &Parts) -> bool {
        Self::extract_bearer_token(parts).is_some()
    }

    async fn authenticate(&self, parts: &Parts) -> Result<SecurityToken, AuthError> {
        let token_str = Self::extract_bearer_token(parts)
            .ok_or(AuthError::Unauthenticated)?;

        let claims = self.manager.validate(token_str)?;

        let mut token = SecurityToken::new(&claims.sub, &claims.identifier);
        token = token.with_roles(claims.roles);

        for (key, value) in &claims.extra {
            if let Some(s) = value.as_str() {
                token = token.with_attribute(key, s);
            }
        }

        Ok(token)
    }
}

/// Simple API key authenticator — validates against a set of known keys.
///
/// Expects: `X-API-Key: <key>` header.
///
/// Useful for service-to-service authentication.
pub struct ApiKeyAuthenticator {
    valid_keys: HashSet<String>,
    /// Role to assign to API key users
    role: String,
}

impl ApiKeyAuthenticator {
    pub fn new(keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        ApiKeyAuthenticator {
            valid_keys: keys.into_iter().map(|k| k.into()).collect(),
            role: "ROLE_API".to_string(),
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }
}

#[async_trait::async_trait]
impl Authenticator for ApiKeyAuthenticator {
    fn supports(&self, parts: &Parts) -> bool {
        parts.headers.contains_key("X-API-Key")
    }

    async fn authenticate(&self, parts: &Parts) -> Result<SecurityToken, AuthError> {
        let key = parts
            .headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::Unauthenticated)?;

        if self.valid_keys.contains(key) {
            Ok(SecurityToken::new("api-key", key).with_role(&self.role))
        } else {
            Err(AuthError::InvalidCredentials("Invalid API key".into()))
        }
    }
}
