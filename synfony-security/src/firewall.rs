use axum::body::Body;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::authenticator::AuthenticatorBox;
use crate::error::AuthError;
use crate::token::SecurityToken;
use crate::voter::{AccessDecisionManager, DecisionStrategy};

use serde::Deserialize;

/// Security configuration loaded from security.yaml.
///
/// Mirrors Symfony's security.yaml structure.
///
/// ```yaml
/// security:
///   firewalls:
///     api:
///       pattern: "/api/*"
///       authenticator: jwt
///     public:
///       pattern: "/*"
///       anonymous: true
///   access_control:
///     - path: "/api/admin/*"
///       roles: ["ROLE_ADMIN"]
///     - path: "/api/*"
///       roles: ["ROLE_USER"]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub firewalls: HashMap<String, FirewallConfig>,
    #[serde(default)]
    pub access_control: Vec<AccessControlEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FirewallConfig {
    pub pattern: String,
    #[serde(default)]
    pub authenticator: Option<String>,
    #[serde(default)]
    pub anonymous: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessControlEntry {
    pub path: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

/// A configured firewall — knows its pattern, authenticator, and access rules.
pub struct Firewall {
    pub name: String,
    pub config: FirewallConfig,
    pub authenticator: Option<AuthenticatorBox>,
}

/// Tower Layer that applies firewall authentication and access control.
///
/// This is the middleware that protects routes — equivalent to Symfony's
/// firewall system configured in security.yaml.
#[derive(Clone)]
pub struct FirewallLayer {
    firewalls: Arc<Vec<Firewall>>,
    access_control: Arc<Vec<AccessControlEntry>>,
    decision_manager: Arc<AccessDecisionManager>,
}

impl FirewallLayer {
    pub fn new() -> Self {
        FirewallLayer {
            firewalls: Arc::new(Vec::new()),
            access_control: Arc::new(Vec::new()),
            decision_manager: Arc::new(AccessDecisionManager::default()),
        }
    }

    /// Build from security config and registered authenticators.
    pub fn from_config(
        config: SecurityConfig,
        authenticators: HashMap<String, AuthenticatorBox>,
    ) -> Self {
        let firewalls: Vec<Firewall> = config
            .firewalls
            .into_iter()
            .map(|(name, fw_config)| {
                let authenticator = fw_config
                    .authenticator
                    .as_ref()
                    .and_then(|auth_name| authenticators.get(auth_name).cloned());
                Firewall {
                    name,
                    config: fw_config,
                    authenticator,
                }
            })
            .collect();

        FirewallLayer {
            firewalls: Arc::new(firewalls),
            access_control: Arc::new(config.access_control),
            decision_manager: Arc::new(AccessDecisionManager::new(DecisionStrategy::Affirmative)),
        }
    }

    pub fn with_decision_manager(mut self, dm: AccessDecisionManager) -> Self {
        self.decision_manager = Arc::new(dm);
        self
    }
}

impl<S> Layer<S> for FirewallLayer {
    type Service = FirewallMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FirewallMiddleware {
            inner,
            firewalls: self.firewalls.clone(),
            access_control: self.access_control.clone(),
            decision_manager: self.decision_manager.clone(),
        }
    }
}

/// The actual middleware service.
#[derive(Clone)]
pub struct FirewallMiddleware<S> {
    inner: S,
    firewalls: Arc<Vec<Firewall>>,
    access_control: Arc<Vec<AccessControlEntry>>,
    decision_manager: Arc<AccessDecisionManager>,
}

impl<S> Service<Request<Body>> for FirewallMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let firewalls = self.firewalls.clone();
        let access_control = self.access_control.clone();
        let decision_manager = self.decision_manager.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let path = req.uri().path().to_string();

            // Find the matching firewall
            let matching_firewall = firewalls
                .iter()
                .find(|fw| path_matches(&path, &fw.config.pattern));

            if let Some(firewall) = matching_firewall {
                // If the firewall allows anonymous access, skip auth
                if firewall.config.anonymous {
                    return inner.call(req).await;
                }

                // Try to authenticate
                if let Some(authenticator) = &firewall.authenticator {
                    // Extract parts for authenticator
                    let (mut parts_struct, body) = req.into_parts();

                    if authenticator.supports(&parts_struct) {
                        match authenticator.authenticate(&parts_struct).await {
                            Ok(token) => {
                                // Check access control rules
                                if let Some(rejection) = check_access_control(
                                    &path,
                                    &token,
                                    &access_control,
                                    &decision_manager,
                                ) {
                                    return Ok(rejection);
                                }

                                // Store the token in request extensions
                                parts_struct.extensions.insert(token);
                                let req = Request::from_parts(parts_struct, body);
                                return inner.call(req).await;
                            }
                            Err(e) => {
                                return Ok(authenticator.on_failure(e));
                            }
                        }
                    } else {
                        // Authenticator doesn't support this request = unauthenticated
                        return Ok(AuthError::Unauthenticated.into_response());
                    }
                }
            }

            // No matching firewall or no authenticator — pass through
            inner.call(req).await
        })
    }
}

/// Check access control rules for the given path and token.
fn check_access_control(
    path: &str,
    token: &SecurityToken,
    rules: &[AccessControlEntry],
    dm: &AccessDecisionManager,
) -> Option<Response> {
    for rule in rules {
        if path_matches(path, &rule.path) {
            for required_role in &rule.roles {
                // Use a unit struct as the subject for role-based checks
                if !dm.is_granted(token, required_role, &()) {
                    return Some(
                        AuthError::InsufficientRole {
                            required: required_role.clone(),
                        }
                        .into_response(),
                    );
                }
            }
            // First matching rule wins (like Symfony)
            return None;
        }
    }
    None
}

/// Simple glob-style path matching.
/// Supports `*` as a wildcard for any number of path segments.
fn path_matches(path: &str, pattern: &str) -> bool {
    if pattern == "/*" || pattern == "*" {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path.starts_with(prefix);
    }

    if let Some(prefix) = pattern.strip_suffix("*") {
        return path.starts_with(prefix);
    }

    path == pattern
}
