mod container;
mod inject;

pub use container::Container;
pub use inject::Inject;

/// Macro to submit a service for auto-discovery via inventory.
#[macro_export]
macro_rules! submit_service {
    ($ty:ty) => {
        ::inventory::submit! {
            $crate::ServiceRegistration::new::<$ty>()
        }
    };
}

/// A service registration entry for the inventory-based auto-discovery.
pub struct ServiceRegistration {
    pub name: &'static str,
    pub constructor: fn(&Container) -> Box<dyn std::any::Any + Send + Sync>,
}

impl ServiceRegistration {
    pub fn new<T: 'static + Send + Sync>() -> Self
    where
        T: FromContainer,
    {
        ServiceRegistration {
            name: std::any::type_name::<T>(),
            constructor: |container| Box::new(T::from_container(container)),
        }
    }
}

inventory::collect!(ServiceRegistration);

/// Trait for types that can be constructed from the DI container.
pub trait FromContainer: 'static + Send + Sync + Sized {
    fn from_container(container: &Container) -> std::sync::Arc<Self>;
}
