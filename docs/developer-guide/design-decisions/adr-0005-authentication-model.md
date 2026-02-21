# ADR 0005: Authentication & Authorization Model

## Status
Proposed

## Context
As Kroki-rs moves towards production deployments, the server needs authentication to prevent unauthorized use. Use cases range from local development (no auth needed) to multi-tenant SaaS (OAuth with per-user rate limits). The design must accommodate this full spectrum without adding complexity for simple deployments.

## Decision

### Two-Tier Authentication

**1. API User Auth** (protecting `/serve` endpoints):
- **API Key** (simple): Token-based via configurable `X-API-Key` header. Keys are defined in `kroki.toml` with optional per-key rate limits.
- **OAuth** (advanced, optional): OAuth2 token validation against external identity providers (e.g., Auth0, Keycloak). Enabled via `[server.auth.oauth]` config section.
- **Demo user**: A built-in API key with strict usage limits for try-before-you-buy scenarios.

**2. Admin Auth** (protecting admin dashboard on port 8081):
- **Password-based**: bcrypt-hashed password stored in `kroki.toml` or passed via `KROKI_ADMIN_PASSWORD` env var. A CLI helper (`kroki-rs admin encrypt-password`) generates the hash.
- **OAuth option**: Admin email allowlist for OAuth-based admin access.

### Dev Mode
When `server.auth.enabled = false` (the default), all authentication is completely bypassed. This enables fast local debugging without any token management. Rate limiting is also independently toggleable.

### Configuration
```toml
[server.auth]
enabled = false
api_keys = [
  { key = "prod-key-abc123", label = "production", rate_limit = 100 },
  { key = "demo-key-xyz", label = "demo", rate_limit = 5 },
]
header_name = "X-API-Key"

[server.auth.oauth]
enabled = false
issuer_url = ""
client_id = ""

[server.admin_auth]
enabled = false
password_hash = ""
oauth_admin_emails = []
```

## Consequences
- **Positive**: Simple deployments remain zero-config. Production deployments get enterprise-grade auth.
- **Positive**: Dev mode prevents auth from slowing down development iteration.
- **Positive**: Per-key rate limits enable multi-tenant usage with differentiated service tiers.
- **Negative**: OAuth integration adds optional dependencies and configuration complexity.
