//! # Synfony Messenger
//!
//! Message bus for the Synfony framework — equivalent to Symfony Messenger.
//!
//! Provides:
//! - **MessageBus**: Dispatch messages to handlers (sync or async)
//! - **Message handlers**: Async functions that process a specific message type
//! - **Transports**: In-process (tokio::spawn) for background processing
//!
//! ## Example
//! ```ignore
//! // Define a message
//! struct SendWelcomeEmail { user_id: i32, email: String }
//!
//! // Register a handler
//! let mut bus = MessageBus::new();
//! bus.register_handler::<SendWelcomeEmail>(|msg| Box::pin(async move {
//!     println!("Sending welcome email to {}", msg.email);
//!     Ok(())
//! }));
//!
//! // Dispatch synchronously (wait for handler)
//! bus.dispatch(SendWelcomeEmail { user_id: 1, email: "a@b.com".into() }).await?;
//!
//! // Dispatch asynchronously (fire and forget via tokio::spawn)
//! bus.dispatch_async(SendWelcomeEmail { user_id: 2, email: "c@d.com".into() });
//! ```

mod bus;
mod handler;

pub use bus::MessageBus;
pub use handler::MessageHandler;

#[derive(Debug, thiserror::Error)]
pub enum MessengerError {
    #[error("No handler registered for message type: {0}")]
    NoHandler(String),

    #[error("Handler failed: {0}")]
    HandlerFailed(String),
}
