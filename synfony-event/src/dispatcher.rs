use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A type-erased async listener callback.
/// Takes a cloned event (Box<dyn Any>) and returns a future.
type BoxedListener =
    Arc<dyn Fn(Box<dyn Any + Send>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// A listener registration with priority.
struct ListenerEntry {
    priority: i32,
    callback: BoxedListener,
}

/// The event dispatcher — equivalent to Symfony's `EventDispatcherInterface`.
///
/// Events must implement `Clone` so each listener receives its own copy.
///
/// # Example
/// ```ignore
/// let dispatcher = EventDispatcher::new();
///
/// dispatcher.listen::<UserCreatedEvent>(10, |event: UserCreatedEvent| async move {
///     println!("New user: {}", event.email);
/// });
///
/// dispatcher.dispatch(UserCreatedEvent { user_id: 1, email: "a@b.com".into() }).await;
/// ```
#[derive(Clone)]
pub struct EventDispatcher {
    listeners: Arc<std::sync::RwLock<HashMap<TypeId, Vec<ListenerEntry>>>>,
}

/// Trait for event listeners — any async fn that takes an owned event.
pub trait Listener<E: 'static + Send>: Send + Sync + 'static {
    fn handle(&self, event: E) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

/// Implement Listener for closures returning futures.
impl<E, F, Fut> Listener<E> for F
where
    E: 'static + Send,
    F: Fn(E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn handle(&self, event: E) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin((self)(event))
    }
}

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher {
            listeners: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Register a listener for an event type with a priority.
    ///
    /// Higher priority = invoked first (like Symfony).
    pub fn listen<E>(&self, priority: i32, listener: impl Listener<E>)
    where
        E: Clone + Send + Sync + 'static,
    {
        let listener = Arc::new(listener);
        let callback: BoxedListener = Arc::new(move |any: Box<dyn Any + Send>| {
            let event = *any.downcast::<E>().expect("Event type mismatch");
            listener.handle(event)
        });

        let mut map = self.listeners.write().unwrap();
        let entries = map.entry(TypeId::of::<E>()).or_default();
        entries.push(ListenerEntry {
            priority,
            callback,
        });
        entries.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Dispatch an event to all registered listeners.
    ///
    /// Each listener receives a clone of the event.
    pub async fn dispatch<E>(&self, event: E)
    where
        E: Clone + Send + Sync + 'static,
    {
        let callbacks: Vec<BoxedListener> = {
            let map = self.listeners.read().unwrap();
            match map.get(&TypeId::of::<E>()) {
                Some(entries) => entries.iter().map(|e| e.callback.clone()).collect(),
                None => return,
            }
        };

        for callback in callbacks {
            callback(Box::new(event.clone())).await;
        }
    }

    /// Check if any listeners are registered for an event type.
    pub fn has_listeners<E: 'static>(&self) -> bool {
        let map = self.listeners.read().unwrap();
        map.get(&TypeId::of::<E>())
            .is_some_and(|v| !v.is_empty())
    }

    /// Get the number of listeners for an event type.
    pub fn listener_count<E: 'static>(&self) -> usize {
        let map = self.listeners.read().unwrap();
        map.get(&TypeId::of::<E>()).map_or(0, |v| v.len())
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
