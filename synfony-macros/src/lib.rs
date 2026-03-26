extern crate proc_macro;

mod controller;
mod route;
mod service;

use proc_macro::TokenStream;

/// Marks an `impl` block as a controller with a route prefix.
///
/// # Example
/// ```ignore
/// #[controller("/api/users")]
/// impl UserController {
///     #[route(GET, "/")]
///     async fn list(&self) -> Json<Vec<User>> { ... }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller::controller_impl(attr, item)
}

/// Defines a route on a controller method.
///
/// # Example
/// ```ignore
/// #[route(GET, "/users/:id")]
/// async fn show(&self, Path(id): Path<i32>) -> Json<User> { ... }
/// ```
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::route_impl(attr, item)
}

/// Marks a struct as a service in the DI container.
///
/// # Example
/// ```ignore
/// #[service]
/// struct UserService {
///     repo: Inject<UserRepository>,
/// }
/// ```
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    service::service_impl(attr, item)
}
