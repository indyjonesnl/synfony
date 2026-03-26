/// Async message: send a welcome email to a new user.
///
/// Symfony equivalent:
/// ```php
/// #[AsMessage]
/// class SendWelcomeEmail {
///     public function __construct(
///         public readonly int $userId,
///         public readonly string $email,
///     ) {}
/// }
/// ```
#[derive(Debug)]
pub struct SendWelcomeEmail {
    pub user_id: i32,
    pub email: String,
}

/// Async message: notify admins of new user registration.
#[derive(Debug)]
pub struct NotifyAdminsOfNewUser {
    pub user_id: i32,
    pub user_name: String,
}
