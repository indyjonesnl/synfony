use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl};

struct ParsedRoute {
    method: String,
    path: String,
    name: Option<String>,
}

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
    let mut metadata_entries = Vec::new();

    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            for attr in &method.attrs {
                if attr.path().is_ident("route") {
                    if let Some(parsed) = parse_route_attr(attr) {
                        let fn_name = &method.sig.ident;
                        let full_path = if parsed.path == "/" {
                            prefix.clone()
                        } else {
                            format!("{}{}", prefix, parsed.path)
                        };
                        let method_ident = syn::Ident::new(
                            &parsed.method.to_lowercase(),
                            proc_macro2::Span::call_site(),
                        );

                        route_registrations.push(quote! {
                            .route(#full_path, ::synfony::axum::routing::#method_ident(<#self_ty>::#fn_name))
                        });

                        if let Some(name) = &parsed.name {
                            let method_str = &parsed.method;
                            metadata_entries.push(quote! {
                                ::synfony::RouteDefinition {
                                    name: #name.to_string(),
                                    path: #full_path.to_string(),
                                    method: #method_str.to_string(),
                                }
                            });
                        }
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

    // Generate the controller name for debug output
    let type_name = quote!(#self_ty).to_string();

    let expanded = quote! {
        #clean_impl

        impl ::synfony::Controller for #self_ty {
            fn routes() -> ::synfony::axum::Router<::synfony::AppState> {
                ::synfony::axum::Router::new()
                    #(#route_registrations)*
            }

            fn route_metadata() -> Vec<::synfony::RouteDefinition> {
                vec![#(#metadata_entries),*]
            }
        }

        // Auto-register this controller for discovery by Application::run()
        ::synfony::inventory::submit! {
            ::synfony::ControllerRegistration {
                name: #type_name,
                routes_fn: <#self_ty as ::synfony::Controller>::routes,
                metadata_fn: <#self_ty as ::synfony::Controller>::route_metadata,
            }
        }
    };

    expanded.into()
}

fn parse_route_attr(attr: &syn::Attribute) -> Option<ParsedRoute> {
    let tokens = attr.meta.require_list().ok()?;
    let args: proc_macro2::TokenStream = tokens.tokens.clone();
    let args_str = args.to_string();

    // Parse formats:
    //   #[route(GET, "/path")]
    //   #[route(GET, "/path", name = "route_name")]
    let parts: Vec<&str> = args_str.splitn(3, ',').collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0].trim().to_uppercase();
    let path = parts[1].trim().trim_matches('"').trim().to_string();

    let name = if parts.len() >= 3 {
        let name_part = parts[2].trim();
        if let Some(rest) = name_part.strip_prefix("name") {
            let rest = rest.trim().strip_prefix('=')?.trim();
            Some(rest.trim_matches('"').trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    Some(ParsedRoute { method, path, name })
}
