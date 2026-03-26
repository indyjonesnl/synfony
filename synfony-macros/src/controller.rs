use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl};

pub fn controller_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let prefix_lit = syn::parse::<syn::LitStr>(attr);
    let prefix = match prefix_lit {
        Ok(lit) => lit.value(),
        Err(e) => return e.to_compile_error().into(),
    };

    let input = syn::parse::<ItemImpl>(item);
    let impl_block = match input {
        Ok(item) => item,
        Err(e) => return e.to_compile_error().into(),
    };

    let self_ty = &impl_block.self_ty;

    // Collect route information from methods
    let mut route_registrations = Vec::new();

    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            for attr in &method.attrs {
                if attr.path().is_ident("route") {
                    if let Some((http_method, path)) = parse_route_attr(attr) {
                        let fn_name = &method.sig.ident;
                        let full_path = format!("{}{}", prefix, path);
                        let method_ident = syn::Ident::new(
                            &http_method.to_lowercase(),
                            proc_macro2::Span::call_site(),
                        );

                        route_registrations.push(quote! {
                            .route(#full_path, ::synfony::axum::routing::#method_ident(Self::#fn_name))
                        });
                    }
                }
            }
        }
    }

    // Strip #[route(...)] attributes from methods for the output impl block
    let mut clean_impl = impl_block.clone();
    for item in &mut clean_impl.items {
        if let ImplItem::Fn(method) = item {
            method.attrs.retain(|attr| !attr.path().is_ident("route"));
        }
    }

    let expanded = quote! {
        #clean_impl

        impl #self_ty {
            /// Returns an Axum router with all routes registered for this controller.
            pub fn routes() -> ::synfony::axum::Router<::synfony::AppState> {
                ::synfony::axum::Router::new()
                    #(#route_registrations)*
            }
        }
    };

    expanded.into()
}

fn parse_route_attr(attr: &syn::Attribute) -> Option<(String, String)> {
    let tokens = attr.meta.require_list().ok()?;
    let args: proc_macro2::TokenStream = tokens.tokens.clone();
    let args_str = args.to_string();

    // Parse "GET , "/path""
    let parts: Vec<&str> = args_str.splitn(2, ',').collect();
    if parts.len() != 2 {
        return None;
    }

    let method = parts[0].trim().to_uppercase();
    let path = parts[1].trim().trim_matches('"').trim().to_string();

    Some((method, path))
}
