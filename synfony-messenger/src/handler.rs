use std::future::Future;
use std::pin::Pin;

use crate::MessengerError;

/// Trait for message handlers.
///
/// Equivalent to Symfony's `#[AsMessageHandler]` classes.
pub trait MessageHandler<M: Send + 'static>: Send + Sync + 'static {
    fn handle(&self, message: M) -> Pin<Box<dyn Future<Output = Result<(), MessengerError>> + Send>>;
}

/// Implement MessageHandler for closures.
impl<M, F, Fut> MessageHandler<M> for F
where
    M: Send + 'static,
    F: Fn(M) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), MessengerError>> + Send + 'static,
{
    fn handle(&self, message: M) -> Pin<Box<dyn Future<Output = Result<(), MessengerError>> + Send>> {
        Box::pin((self)(message))
    }
}
