//! # Hello API — Synfony Example Application
//!
//! A complete API demonstrating all Synfony framework features.
//! Controllers are auto-discovered — no manual route registration needed.
//!
//! ## Demo Credentials
//! - Admin: admin@example.com / admin
//! - User:  user@example.com / user

// Controllers are auto-discovered by the #[controller] macro.
// Just declaring the modules is enough — no registration calls needed.
mod controller;
mod dto;
mod entity;
mod event;
mod message;
mod repository;
mod service;

use std::collections::HashMap;
use std::sync::Arc;

use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use synfony::Application;
use synfony_event::EventDispatcher;
use synfony_messenger::MessageBus;
use synfony_orm::connect;
use synfony_orm::connection::DatabaseConfig;
use synfony_security::firewall::{
    AccessControlEntry, FirewallConfig, FirewallLayer, SecurityConfig,
};
use synfony_security::jwt::{JwtAuthenticator, JwtConfig, JwtManager};

use event::{UserCreatedEvent, UserDeletedEvent};
use message::{NotifyAdminsOfNewUser, SendWelcomeEmail};
use repository::UserRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Application::new()?;

    // --- Database ---
    let db_config = DatabaseConfig {
        url: "sqlite:./hello-api.db?mode=rwc".to_string(),
        ..Default::default()
    };
    let db = Arc::new(connect(&db_config).await?);

    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL DEFAULT 'ROLE_USER'
        )
        "#
        .to_string(),
    ))
    .await?;

    let count: Option<serde_json::Value> = db
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(*) as c FROM users".to_string(),
        ))
        .await?
        .map(|r| {
            let val: i32 = r.try_get_by_index(0).unwrap_or(0);
            serde_json::json!(val)
        });

    if count == Some(serde_json::json!(0)) {
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "INSERT INTO users (name, email, role) VALUES ('Alice', 'alice@example.com', 'ROLE_USER'), ('Bob', 'bob@example.com', 'ROLE_USER'), ('Admin', 'admin@example.com', 'ROLE_ADMIN')".to_string(),
        ))
        .await?;
        tracing::info!("Seeded 3 demo users");
    }

    // --- Security (equivalent to security.yaml) ---
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-dev-secret".to_string());
    let jwt_config = JwtConfig::new(&jwt_secret).with_ttl(3600);
    let jwt_authenticator = Arc::new(JwtAuthenticator::new(jwt_config.clone()));
    let jwt_manager = Arc::new(JwtManager::new(jwt_config));

    app.register_service(jwt_manager);

    let security_config = SecurityConfig {
        firewalls: HashMap::from([
            (
                "public".to_string(),
                FirewallConfig {
                    pattern: "/api/health".to_string(),
                    authenticator: None,
                    anonymous: true,
                },
            ),
            (
                "login".to_string(),
                FirewallConfig {
                    pattern: "/api/login".to_string(),
                    authenticator: None,
                    anonymous: true,
                },
            ),
            (
                "api".to_string(),
                FirewallConfig {
                    pattern: "/api/*".to_string(),
                    authenticator: Some("jwt".to_string()),
                    anonymous: false,
                },
            ),
        ]),
        access_control: vec![
            AccessControlEntry {
                path: "/api/admin/*".to_string(),
                roles: vec!["ROLE_ADMIN".to_string()],
            },
            AccessControlEntry {
                path: "/api/*".to_string(),
                roles: vec!["ROLE_USER".to_string()],
            },
        ],
    };

    let authenticators: HashMap<String, synfony_security::AuthenticatorBox> =
        HashMap::from([(
            "jwt".to_string(),
            jwt_authenticator as synfony_security::AuthenticatorBox,
        )]);

    app.set_firewall(FirewallLayer::from_config(security_config, authenticators));

    // --- Event Dispatcher ---
    let dispatcher = EventDispatcher::new();

    dispatcher.listen::<UserCreatedEvent>(20, |event: UserCreatedEvent| async move {
        tracing::info!(user_id = event.user_id, email = %event.email, "EVENT [UserCreatedEvent] New user registered");
    });

    dispatcher.listen::<UserCreatedEvent>(10, |event: UserCreatedEvent| async move {
        tracing::info!(user_id = event.user_id, "EVENT [UserCreatedEvent] Audit: user creation recorded");
    });

    dispatcher.listen::<UserDeletedEvent>(10, |event: UserDeletedEvent| async move {
        tracing::info!(user_id = event.user_id, "EVENT [UserDeletedEvent] User removed from system");
    });

    app.register_service(Arc::new(dispatcher));

    // --- Message Bus ---
    let bus = MessageBus::new();

    bus.register_handler::<SendWelcomeEmail>(|msg: SendWelcomeEmail| async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        tracing::info!(user_id = msg.user_id, email = %msg.email, "MESSENGER [SendWelcomeEmail] Welcome email sent");
        Ok(())
    });

    bus.register_handler::<NotifyAdminsOfNewUser>(|msg: NotifyAdminsOfNewUser| async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        tracing::info!(user_id = msg.user_id, user_name = %msg.user_name, "MESSENGER [NotifyAdminsOfNewUser] Admin notification sent");
        Ok(())
    });

    app.register_service(Arc::new(bus));

    // --- Services ---
    let user_repo = UserRepository::new(db.clone());
    app.register_service(user_repo);
    app.register_service(db);

    // --- Run ---
    // Controllers are auto-discovered from #[controller] macros.
    // No register_routes() calls needed!
    app.run().await?;
    Ok(())
}
