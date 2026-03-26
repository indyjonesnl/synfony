use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::handler::MessageHandler;
use crate::MessengerError;

/// Type-erased handler that accepts a Box<dyn Any> and returns a future.
type BoxedHandler = Arc<
    dyn Fn(Box<dyn Any + Send>) -> Pin<Box<dyn Future<Output = Result<(), MessengerError>> + Send>>
        + Send
        + Sync,
>;

/// The message bus — equivalent to Symfony's `MessageBusInterface`.
///
/// Routes messages to their registered handlers. Supports:
/// - **Sync dispatch**: `dispatch()` — waits for the handler to complete
/// - **Async dispatch**: `dispatch_async()` — spawns a tokio task (fire-and-forget)
///
/// ## Symfony Comparison
/// ```php
/// // Symfony
/// $bus->dispatch(new SendWelcomeEmail($userId));
///
/// // Synfony (sync)
/// bus.dispatch(SendWelcomeEmail { user_id }).await?;
///
/// // Synfony (async — like using an async transport)
/// bus.dispatch_async(SendWelcomeEmail { user_id });
/// ```
#[derive(Clone)]
pub struct MessageBus {
    handlers: Arc<std::sync::RwLock<HashMap<TypeId, BoxedHandler>>>,
}

impl MessageBus {
    pub fn new() -> Self {
        MessageBus {
            handlers: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Register a handler for a message type.
    ///
    /// Only one handler per message type (like Symfony's single handler routing).
    /// Registering a second handler for the same type replaces the first.
    pub fn register_handler<M>(&self, handler: impl MessageHandler<M>)
    where
        M: Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        let boxed: BoxedHandler = Arc::new(move |any: Box<dyn Any + Send>| {
            let msg = *any.downcast::<M>().expect("Message type mismatch");
            handler.handle(msg)
        });

        let mut map = self.handlers.write().unwrap();
        map.insert(TypeId::of::<M>(), boxed);
    }

    /// Dispatch a message synchronously — waits for the handler to complete.
    ///
    /// Equivalent to Symfony Messenger with a sync transport.
    pub async fn dispatch<M>(&self, message: M) -> Result<(), MessengerError>
    where
        M: Send + Sync + 'static,
    {
        let handler = {
            let map = self.handlers.read().unwrap();
            map.get(&TypeId::of::<M>()).cloned()
        };

        match handler {
            Some(h) => h(Box::new(message)).await,
            None => Err(MessengerError::NoHandler(
                std::any::type_name::<M>().to_string(),
            )),
        }
    }

    /// Dispatch a message asynchronously — spawns a tokio task.
    ///
    /// Equivalent to Symfony Messenger with an async transport (Redis, AMQP, etc).
    /// The message is processed in the background; errors are logged but not returned.
    pub fn dispatch_async<M>(&self, message: M)
    where
        M: Send + Sync + 'static,
    {
        let handler = {
            let map = self.handlers.read().unwrap();
            map.get(&TypeId::of::<M>()).cloned()
        };

        if let Some(h) = handler {
            tokio::spawn(async move {
                if let Err(e) = h(Box::new(message)).await {
                    tracing::error!("Async message handler failed: {}", e);
                }
            });
        } else {
            tracing::warn!(
                "No handler registered for async message: {}",
                std::any::type_name::<M>()
            );
        }
    }

    /// Check if a handler is registered for a message type.
    pub fn has_handler<M: 'static>(&self) -> bool {
        let map = self.handlers.read().unwrap();
        map.contains_key(&TypeId::of::<M>())
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}
