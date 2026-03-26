extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

/// Derive macro for serialization groups.
///
/// Generates per-group serialization functions, similar to Symfony's
/// `#[Groups(["list", "detail"])]` serialization groups.
///
/// # Example
/// ```ignore
/// #[derive(Serialize, SerializeGroups)]
/// struct UserDto {
///     #[groups("list", "detail")]
///     id: i32,
///
///     #[groups("list", "detail")]
///     name: String,
///
///     #[groups("detail")]
///     email: String,
///
///     #[groups("admin")]
///     role: String,
/// }
///
/// // Serialize only "list" group fields → { id, name }
/// let json = UserDto::serialize_group(&user, "list")?;
/// ```
#[proc_macro_derive(SerializeGroups, attributes(groups))]
pub fn derive_serialize_groups(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "SerializeGroups only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "SerializeGroups only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Collect all groups and their fields
    let mut all_groups = std::collections::BTreeSet::new();
    let mut field_groups: Vec<(syn::Ident, syn::Type, Vec<String>)> = Vec::new();

    for field in fields {
        let field_name = field.ident.clone().unwrap();
        let field_type = field.ty.clone();
        let mut groups = Vec::new();

        for attr in &field.attrs {
            if attr.path().is_ident("groups") {
                if let Ok(meta) = attr.meta.require_list() {
                    let tokens = meta.tokens.to_string();
                    // Parse comma-separated string literals: "list", "detail"
                    for part in tokens.split(',') {
                        let group = part.trim().trim_matches('"').trim().to_string();
                        if !group.is_empty() {
                            all_groups.insert(group.clone());
                            groups.push(group);
                        }
                    }
                }
            }
        }

        field_groups.push((field_name, field_type, groups));
    }

    // Generate the serialize_group method
    // For each field, include it only if its groups contain the requested group
    let field_serialization: Vec<_> = field_groups
        .iter()
        .map(|(name, _ty, groups)| {
            let name_str = name.to_string();
            let group_strs: Vec<&str> = groups.iter().map(|s| s.as_str()).collect();
            quote! {
                if [#(#group_strs),*].contains(&group) {
                    map.insert(
                        #name_str.to_string(),
                        ::serde_json::to_value(&self.#name)?,
                    );
                }
            }
        })
        .collect();

    let groups_list: Vec<&str> = all_groups.iter().map(|s| s.as_str()).collect();

    let expanded = quote! {
        impl #name {
            /// Serialize this struct including only fields belonging to the given group.
            ///
            /// Equivalent to Symfony's serialization groups: `$serializer->serialize($obj, 'json', ['groups' => ['list']])`
            pub fn serialize_group(&self, group: &str) -> Result<::serde_json::Value, ::serde_json::Error> {
                let mut map = ::serde_json::Map::new();
                #(#field_serialization)*
                Ok(::serde_json::Value::Object(map))
            }

            /// Serialize with multiple groups (union of all fields in any of the groups).
            pub fn serialize_groups(&self, groups: &[&str]) -> Result<::serde_json::Value, ::serde_json::Error> {
                let mut map = ::serde_json::Map::new();
                for &group in groups {
                    // Merge each group's fields
                    if let ::serde_json::Value::Object(group_map) = self.serialize_group(group)? {
                        map.extend(group_map);
                    }
                }
                Ok(::serde_json::Value::Object(map))
            }

            /// Returns the list of all available serialization groups for this type.
            pub fn available_groups() -> &'static [&'static str] {
                &[#(#groups_list),*]
            }
        }
    };

    expanded.into()
}
