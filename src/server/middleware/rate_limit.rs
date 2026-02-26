//! Token-bucket rate limiting middleware for the Kroki-rs server.
//!
//! Implements per-IP rate limiting using a concurrent `DashMap` for O(1) lookups.
//! When `server.rate_limit.enabled = false` (default), all requests pass through.

use crate::config::RateLimitConfig;
use axum::{
    body::Body,
    extract::{connect_info::ConnectInfo, Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

/// A token bucket for a single client IP.
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(burst_size: u32, refill_rate: u32) -> Self {
        Self {
            tokens: burst_size as f64,
            last_refill: Instant::now(),
            max_tokens: burst_size as f64,
            refill_rate: refill_rate as f64,
        }
    }

    /// Attempts to consume a token. Returns true if allowed, false if rate limited.
    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Returns the estimated seconds until a token is available.
    fn retry_after(&self) -> u64 {
        if self.refill_rate <= 0.0 {
            return 60;
        }
        let deficit = 1.0 - self.tokens;
        (deficit / self.refill_rate).ceil() as u64
    }
}

/// Shared rate limiter state keyed by client IP.
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<IpAddr, TokenBucket>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Creates a new rate limiter from configuration.
    pub fn new(config: &RateLimitConfig) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            config: config.clone(),
        }
    }

    /// Attempts to allow a request from the given IP.
    /// Returns `Ok(())` if allowed, or `Err(retry_after_secs)` if rate limited.
    pub fn check(&self, ip: IpAddr) -> Result<(), u64> {
        let mut entry = self.buckets.entry(ip).or_insert_with(|| {
            TokenBucket::new(self.config.burst_size, self.config.requests_per_second)
        });

        if entry.try_consume() {
            Ok(())
        } else {
            Err(entry.retry_after())
        }
    }
}

/// Axum middleware that enforces rate limits.
///
/// Skipped entirely when `rate_limit.enabled = false` (dev mode).
/// Returns 429 with `Retry-After` header when the limit is exceeded.
pub async fn rate_limit_middleware(
    State(state): State<crate::server::AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !state.config.server.rate_limit.enabled {
        return next.run(request).await;
    }

    let fallback_ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip())
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap());

    let ip = extract_client_ip(&request, fallback_ip);

    if let Some(ref limiter) = state.rate_limiter {
        match limiter.check(ip) {
            Ok(()) => next.run(request).await,
            Err(retry_after) => {
                tracing::warn!("Rate limit exceeded for IP: {}", ip);
                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": "Too many requests. Please retry later.",
                        "retry_after_seconds": retry_after
                    })
                    .to_string(),
                )
                    .into_response();

                response.headers_mut().insert(
                    "retry-after",
                    HeaderValue::from_str(&retry_after.to_string())
                        .unwrap_or_else(|_| HeaderValue::from_static("60")),
                );

                response
            }
        }
    } else {
        next.run(request).await
    }
}

/// Extracts the client IP from the request.
/// Checks `X-Forwarded-For` first (for reverse proxy setups), then falls back to
/// `X-Real-IP`, and finally defaults to `127.0.0.1`.
fn extract_client_ip(request: &Request<Body>, fallback: IpAddr) -> IpAddr {
    // Try X-Forwarded-For (first IP in the chain)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_allows_burst() {
        let mut bucket = TokenBucket::new(5, 1);
        for _ in 0..5 {
            assert!(bucket.try_consume());
        }
        // 6th request should be denied
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_token_bucket_refills() {
        let mut bucket = TokenBucket::new(1, 1000); // very fast refill
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume());
        // Simulate time passing
        bucket.last_refill = Instant::now() - std::time::Duration::from_millis(10);
        assert!(bucket.try_consume());
    }

    #[test]
    fn test_rate_limiter_per_ip() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_second: 1,
            burst_size: 2,
        };
        let limiter = RateLimiter::new(&config);
        let ip1: IpAddr = "192.168.1.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.2".parse().unwrap();

        // Each IP gets its own bucket
        assert!(limiter.check(ip1).is_ok());
        assert!(limiter.check(ip1).is_ok());
        assert!(limiter.check(ip1).is_err()); // exhausted for ip1

        assert!(limiter.check(ip2).is_ok()); // ip2 is independent
    }
}
