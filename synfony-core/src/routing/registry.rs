use std::collections::HashMap;

/// Metadata for a single named route.
///
/// Equivalent to Symfony's Route object. Stores the name, path pattern,
/// and HTTP method for URL generation and debugging.
#[derive(Debug, Clone)]
pub struct RouteDefinition {
    pub name: String,
    pub path: String,
    pub method: String,
}

/// Error returned when URL generation fails.
#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("Route '{0}' is not defined")]
    RouteNotFound(String),

    #[error("Missing parameter '{param}' for route '{route}'")]
    MissingParameter { route: String, param: String },
}

/// Stores all named route definitions.
///
/// Populated during application boot, then frozen into `Arc<RouteRegistry>`
/// and registered in the DI container. Immutable after boot.
///
/// Equivalent to Symfony's RouteCollection.
pub struct RouteRegistry {
    routes: HashMap<String, RouteDefinition>,
}

impl RouteRegistry {
    pub fn new() -> Self {
        RouteRegistry {
            routes: HashMap::new(),
        }
    }

    /// Register a named route. Panics on duplicate names (fail-fast at boot).
    pub fn add(&mut self, definition: RouteDefinition) {
        if self.routes.contains_key(&definition.name) {
            panic!(
                "Duplicate route name '{}'. Route names must be unique.",
                definition.name
            );
        }
        self.routes.insert(definition.name.clone(), definition);
    }

    /// Look up a route by name.
    pub fn get(&self, name: &str) -> Option<&RouteDefinition> {
        self.routes.get(name)
    }

    /// Returns all registered route definitions (for debug:router).
    pub fn all(&self) -> Vec<&RouteDefinition> {
        let mut routes: Vec<_> = self.routes.values().collect();
        routes.sort_by_key(|r| &r.name);
        routes
    }

    /// Number of registered routes.
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Substitute `{param}` placeholders in a route's path pattern.
    pub fn generate_path(
        &self,
        name: &str,
        params: &[(&str, &str)],
    ) -> Result<String, RoutingError> {
        let definition = self
            .get(name)
            .ok_or_else(|| RoutingError::RouteNotFound(name.to_string()))?;

        let mut path = definition.path.clone();
        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            path = path.replace(&placeholder, value);
        }

        // Check for unreplaced placeholders
        if let Some(start) = path.find('{') {
            if let Some(end) = path[start..].find('}') {
                let param = &path[start + 1..start + end];
                return Err(RoutingError::MissingParameter {
                    route: name.to_string(),
                    param: param.to_string(),
                });
            }
        }

        Ok(path)
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}
