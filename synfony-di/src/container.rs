use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The Synfony service container.
///
/// Stores singleton services keyed by TypeId. Services are resolved lazily
/// on first access and cached for subsequent requests.
///
/// This is the Rust equivalent of Symfony's compiled service container,
/// but using Arc<T> for shared ownership instead of PHP references.
#[derive(Clone)]
pub struct Container {
    services: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl Container {
    pub fn new() -> Self {
        Container {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a singleton service instance.
    pub fn set<T: 'static + Send + Sync>(&self, service: Arc<T>) {
        let mut services = self.services.write().unwrap();
        services.insert(TypeId::of::<T>(), Box::new(service));
    }

    /// Resolve a service from the container.
    ///
    /// # Panics
    /// Panics if the service is not registered. In production use,
    /// the `#[module]` macro validates all dependencies at compile time,
    /// so this should never panic.
    pub fn resolve<T: 'static + Send + Sync>(&self) -> Arc<T> {
        let services = self.services.read().unwrap();
        services
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "Service `{}` is not registered in the container. \
                     Did you forget to add #[service] or register it in your module?",
                    std::any::type_name::<T>()
                )
            })
    }

    /// Try to resolve a service, returning None if not registered.
    pub fn try_resolve<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        let services = self.services.read().unwrap();
        services
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Arc<T>>())
            .cloned()
    }

    /// Check if a service is registered.
    pub fn has<T: 'static + Send + Sync>(&self) -> bool {
        let services = self.services.read().unwrap();
        services.contains_key(&TypeId::of::<T>())
    }

    /// Returns a list of all registered service type names (for debug:container).
    pub fn registered_services(&self) -> Vec<&'static str> {
        // We'd need to store names alongside — this is a placeholder
        // that will be enhanced when we add the debug infrastructure.
        Vec::new()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
