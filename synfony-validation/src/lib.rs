//! # Synfony Validation
//!
//! Auto-validating request extractors for Synfony.
//!
//! Provides:
//! - `JsonBody<T>` — Deserialize + validate JSON body (like Symfony's `#[MapRequestPayload]`)
//! - `QueryParams<T>` — Deserialize + validate query parameters (like Symfony's `#[MapQueryString]`)
//! - `ValidatedForm<T>` — Deserialize + validate form data
//!
//! Returns 422 Unprocessable Entity with structured validation errors on failure.

mod error;
mod json_body;
mod query_params;

pub use error::ValidationError;
pub use json_body::JsonBody;
pub use query_params::QueryParams;

// Re-export validator for convenience
pub use validator::{self, Validate};
