//! # Synfony Security
//!
//! Authentication and authorization component for the Synfony framework.
//!
//! Mirrors Symfony's Security component:
//! - **Authenticators**: Extract and verify credentials (JWT, API key, custom)
//! - **Firewalls**: Route-pattern-based middleware that applies authenticators
//! - **Voters**: Object-level authorization (can this user edit this post?)
//! - **SecurityToken**: The authenticated user context
//! - **CurrentUser**: Axum extractor for the logged-in user

mod authenticator;
mod current_user;
mod error;
pub mod firewall;
mod token;
mod voter;

pub mod jwt;

pub use authenticator::{Authenticator, AuthenticatorBox};
pub use current_user::{CurrentUser, OptionalUser};
pub use error::AuthError;
pub use firewall::{Firewall, FirewallConfig, FirewallLayer, SecurityConfig};
pub use token::SecurityToken;
pub use voter::{AccessDecisionManager, Vote, Voter};
