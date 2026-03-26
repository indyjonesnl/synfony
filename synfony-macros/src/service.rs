use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, ItemStruct};

pub fn service_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;

    let fields = match &input.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                &input,
                "#[service] can only be applied to structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let field_vis: Vec<_> = fields.iter().map(|f| &f.vis).collect();
    let field_attrs: Vec<Vec<_>> = fields.iter().map(|f| f.attrs.iter().collect()).collect();

    let expanded = quote! {
        #(#attrs)*
        #vis struct #name #generics {
            #(
                #(#field_attrs)*
                #field_vis #field_names: #field_types,
            )*
        }

        impl #name {
            /// Constructs this service by resolving all dependencies from the container.
            pub fn from_container(container: &::synfony::di::Container) -> ::std::sync::Arc<Self> {
                ::std::sync::Arc::new(Self {
                    #(#field_names: container.resolve::<#field_types>(),)*
                })
            }
        }
    };

    expanded.into()
}
