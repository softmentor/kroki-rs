//! API Key authentication middleware for the Kroki-rs server.
//!
//! When `server.auth.enabled = true`, extracts any API key or admin credentials
//! and validates them.
//! When disabled (the default), all requests pass through — enabling fast local development.

use crate::config::AuthConfig;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Axum middleware that enforces API key authentication.
///
/// Skipped entirely when `auth.enabled = false` (dev mode).
/// Returns 401 with a JSON body if the key is missing or invalid.
pub async fn auth_middleware(
    state: axum::extract::State<crate::server::AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let auth_config = &state.config.server.auth;

    if !auth_config.enabled {
        return next.run(request).await;
    }

    let header_name = &auth_config.header_name;
    let api_key = request
        .headers()
        .get(header_name)
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) => {
            if auth_config.api_keys.iter().any(|entry| entry.key == key) {
                next.run(request).await
            } else {
                tracing::warn!("Invalid API key presented");
                (
                    StatusCode::UNAUTHORIZED,
                    serde_json::json!({
                        "error": "unauthorized",
                        "message": "Invalid API key"
                    })
                    .to_string(),
                )
                    .into_response()
            }
        }
        None => {
            tracing::warn!("Missing API key in header '{}'", header_name);
            (
                StatusCode::UNAUTHORIZED,
                serde_json::json!({
                    "error": "unauthorized",
                    "message": format!("Missing API key. Provide it via the '{}' header.", header_name)
                })
                .to_string(),
            )
                .into_response()
        }
    }
}

/// Axum middleware that enforces admin authentication via Basic Auth.
///
/// Authentication is bypasses if:
/// 1. `auth.enabled = false` (dev mode)
/// 2. `auth.admin_password_hash` is not configured
///
/// Otherwise, expects "Authorization: Basic <base64>" header.
pub async fn admin_auth_middleware(
    state: axum::extract::State<crate::server::AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let auth_config = &state.config.server.auth;

    // Bypass if disabled or no password hash set
    if !auth_config.enabled || auth_config.admin_password_hash.is_none() {
        return next.run(request).await;
    }

    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let authenticated = if let Some(header) = auth_header {
        if let Some(encoded) = header.strip_prefix("Basic ") {
            if let Ok(decoded) = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, encoded)
            {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    if let Some((_user, password)) = credentials.split_once(':') {
                        if let Some(hash) = &auth_config.admin_password_hash {
                            bcrypt::verify(password, hash).unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if authenticated {
        next.run(request).await
    } else {
        tracing::warn!("Admin authentication failed");
        (
            StatusCode::UNAUTHORIZED,
            [(
                axum::http::header::WWW_AUTHENTICATE,
                "Basic realm=\"Kroki Admin\"",
            )],
            serde_json::json!({
                "error": "unauthorized",
                "message": "Admin authentication required"
            })
            .to_string(),
        )
            .into_response()
    }
}

/// Looks up the `ApiKeyEntry` for a given key string.
/// Returns `None` if the key is not found or auth is disabled.
pub fn find_api_key_entry<'a>(
    auth_config: &'a AuthConfig,
    key: &str,
) -> Option<&'a crate::config::ApiKeyEntry> {
    auth_config.api_keys.iter().find(|entry| entry.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiKeyEntry, AuthConfig};

    #[test]
    fn test_find_api_key_entry_found() {
        let config = AuthConfig {
            enabled: true,
            api_keys: vec![ApiKeyEntry {
                key: "test-key".to_string(),
                label: "test".to_string(),
                rate_limit: Some(10),
            }],
            header_name: "x-api-key".to_string(),
            admin_password_hash: None,
        };
        let entry = find_api_key_entry(&config, "test-key");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().label, "test");
    }

    #[test]
    fn test_find_api_key_entry_not_found() {
        let config = AuthConfig {
            enabled: true,
            api_keys: vec![ApiKeyEntry {
                key: "test-key".to_string(),
                label: "test".to_string(),
                rate_limit: None,
            }],
            header_name: "x-api-key".to_string(),
            admin_password_hash: None,
        };
        assert!(find_api_key_entry(&config, "wrong-key").is_none());
    }

    #[test]
    fn test_find_api_key_entry_empty_keys() {
        let config = AuthConfig::default();
        assert!(find_api_key_entry(&config, "any-key").is_none());
    }
}
