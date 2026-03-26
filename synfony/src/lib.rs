//! # Synfony
//!
//! A Symfony-inspired web framework for Rust.
//!
//! Synfony brings the familiar patterns and conventions of PHP's Symfony framework
//! to the Rust ecosystem, giving Symfony developers a natural path to high-performance
//! Rust web applications.

// Re-export framework crates
pub use synfony_core as core;
pub use synfony_di as di;
pub use synfony_config as config;
pub use synfony_console as console;
pub use synfony_macros as macros;
pub use synfony_security as security;
pub use synfony_orm as orm;
pub use synfony_validation as validation;
pub use synfony_serializer as serializer;
pub use synfony_event as event;
pub use synfony_messenger as messenger;

// Re-export key types at the top level for ergonomics
pub use synfony_core::{ApiError, Application, AppState, Controller, ControllerRegistration, ErrorResponse, Kernel, RouteDefinition, RouteRegistry, RoutingError, UrlGenerator};

// Re-export inventory so the #[controller] macro can use it
pub use inventory;
pub use synfony_di::{Container, Inject};
pub use synfony_config::SynfonyConfig;
pub use synfony_console::ConsoleIO;
pub use synfony_security::{
    Authenticator, AuthError, CurrentUser, FirewallLayer, SecurityConfig, SecurityToken, Vote,
    Voter,
};
pub use synfony_orm::{DatabaseConnection, EntityManager, OrmError, Repository};
pub use synfony_validation::{JsonBody, QueryParams, ValidationError};
pub use synfony_serializer::{GroupedJson, SerializeGroups};
pub use synfony_event::EventDispatcher;
pub use synfony_messenger::MessageBus;

// Re-export proc macros
pub use synfony_macros::{controller, route, service};

// Re-export commonly used dependencies so users don't need to add them
pub use axum;
pub use serde;
pub use serde_json;
pub use tokio;
pub use tower;
pub use tower_http;
pub use tracing;
pub use validator;

/// The prelude — import everything you need with `use synfony::prelude::*`.
pub mod prelude {
    pub use crate::{
        ApiError, Application, AppState, Container, ConsoleIO, ErrorResponse, Inject, Kernel,
        SynfonyConfig,
    };
    pub use crate::{controller, route, service};
    pub use crate::{AuthError, CurrentUser, SecurityToken, Vote, Voter};
    pub use crate::{DatabaseConnection, EntityManager, EventDispatcher, MessageBus, OrmError, Repository};
    pub use crate::{GroupedJson, JsonBody, QueryParams, RouteDefinition, SerializeGroups, UrlGenerator, ValidationError};

    // Common axum types used in handlers
    pub use axum::extract::{Json, Path, Query, State};
    pub use axum::response::IntoResponse;
    pub use axum::Router;

    // Serde
    pub use serde::{Deserialize, Serialize};

    // Validation
    pub use validator::Validate;

    // Async
    pub use async_trait::async_trait;
}
