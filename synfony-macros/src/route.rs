use proc_macro::TokenStream;

/// The #[route] attribute is processed by the #[controller] macro.
/// When used standalone (outside a controller), it's a no-op passthrough
/// that preserves the function for the controller macro to pick up.
pub fn route_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
