//! # Synfony Event
//!
//! Typed event dispatcher for the Synfony framework.
//!
//! Mirrors Symfony's EventDispatcher component:
//! - Events are plain structs
//! - Listeners are async functions registered with a priority
//! - Dispatch sends an event to all matching listeners in priority order
//!
//! ## Example
//! ```ignore
//! use synfony_event::{EventDispatcher, Listener};
//!
//! struct UserCreatedEvent { user_id: i32, email: String }
//!
//! let mut dispatcher = EventDispatcher::new();
//! dispatcher.listen::<UserCreatedEvent>(10, |event| Box::pin(async move {
//!     println!("User created: {}", event.email);
//! }));
//!
//! dispatcher.dispatch(UserCreatedEvent { user_id: 1, email: "alice@example.com".into() }).await;
//! ```

mod dispatcher;

pub use dispatcher::{EventDispatcher, Listener};

/// Marker trait for events. Any `Send + Sync + 'static` type can be an event.
pub trait Event: Send + Sync + 'static {}

// Blanket implementation — every compatible type is an Event.
impl<T: Send + Sync + 'static> Event for T {}
