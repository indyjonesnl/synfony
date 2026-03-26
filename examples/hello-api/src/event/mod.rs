/// Event dispatched when a new user is created.
///
/// Symfony equivalent:
/// ```php
/// class UserCreatedEvent {
///     public function __construct(
///         public readonly int $userId,
///         public readonly string $email,
///     ) {}
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UserCreatedEvent {
    pub user_id: i32,
    pub email: String,
}

/// Event dispatched when a user is deleted.
#[derive(Debug, Clone)]
pub struct UserDeletedEvent {
    pub user_id: i32,
}
