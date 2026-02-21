//! Per-provider circuit breaker for the Kroki-rs server.
//!
//! Implements the Closed → Open → Half-Open state machine pattern.
//! Each diagram provider type gets its own independent circuit breaker.
//! When the circuit is open, requests fail immediately with 503 without
//! invoking the actual provider.

use crate::config::CircuitBreakerConfig;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation — requests flow through.
    Closed,
    /// Failures exceeded threshold — requests rejected immediately.
    Open,
    /// Testing recovery — one request allowed through.
    HalfOpen,
}

/// Internal state for a single provider's circuit breaker.
struct ProviderCircuit {
    state: CircuitState,
    consecutive_failures: u32,
    last_failure_time: Option<Instant>,
    failure_threshold: u32,
    reset_timeout_secs: u64,
}

impl ProviderCircuit {
    fn new(config: &CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            last_failure_time: None,
            failure_threshold: config.failure_threshold,
            reset_timeout_secs: config.reset_timeout_secs,
        }
    }

    /// Checks if a request should be allowed through.
    fn should_allow(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if reset timeout has elapsed → transition to HalfOpen
                if let Some(last_fail) = self.last_failure_time {
                    if last_fail.elapsed().as_secs() >= self.reset_timeout_secs {
                        self.state = CircuitState::HalfOpen;
                        tracing::info!("Circuit breaker transitioning to HalfOpen");
                        true // Allow one test request
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Only one request is allowed in HalfOpen; subsequent ones are rejected
                // until the first one completes.
                false
            }
        }
    }

    /// Records a successful request. Resets the circuit to Closed.
    fn record_success(&mut self) {
        if self.state == CircuitState::HalfOpen {
            tracing::info!("Circuit breaker closing after successful HalfOpen test");
        }
        self.state = CircuitState::Closed;
        self.consecutive_failures = 0;
        self.last_failure_time = None;
    }

    /// Records a failed request. May trip the circuit to Open.
    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.last_failure_time = Some(Instant::now());

        if self.consecutive_failures >= self.failure_threshold {
            self.state = CircuitState::Open;
            tracing::warn!(
                "Circuit breaker opened after {} consecutive failures",
                self.consecutive_failures
            );
        }

        // If we were in HalfOpen and the test failed, go back to Open
        if self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Open;
            tracing::warn!("Circuit breaker re-opened after HalfOpen test failure");
        }
    }
}

/// Manages circuit breakers for all diagram providers.
#[derive(Clone)]
pub struct CircuitBreakerManager {
    circuits: Arc<DashMap<String, ProviderCircuit>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// Creates a new circuit breaker manager.
    pub fn new(config: &CircuitBreakerConfig) -> Self {
        Self {
            circuits: Arc::new(DashMap::new()),
            config: config.clone(),
        }
    }

    /// Checks if the circuit for the given provider allows requests.
    /// Returns `true` if the request should proceed, `false` if it should be rejected.
    pub fn should_allow(&self, provider: &str) -> bool {
        let mut entry = self
            .circuits
            .entry(provider.to_string())
            .or_insert_with(|| ProviderCircuit::new(&self.config));
        entry.should_allow()
    }

    /// Records a successful request for the given provider.
    pub fn record_success(&self, provider: &str) {
        if let Some(mut entry) = self.circuits.get_mut(provider) {
            entry.record_success();
        }
    }

    /// Records a failed request for the given provider.
    pub fn record_failure(&self, provider: &str) {
        let mut entry = self
            .circuits
            .entry(provider.to_string())
            .or_insert_with(|| ProviderCircuit::new(&self.config));
        entry.record_failure();
    }

    /// Returns the current state of the circuit for a given provider.
    pub fn get_state(&self, provider: &str) -> CircuitState {
        self.circuits
            .get(provider)
            .map(|entry| entry.state)
            .unwrap_or(CircuitState::Closed)
    }

    /// Returns the states of all known circuits.
    pub fn get_all_states(&self) -> Vec<(String, CircuitState)> {
        self.circuits
            .iter()
            .map(|entry| (entry.key().clone(), entry.state))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 3,
            reset_timeout_secs: 1,
        }
    }

    #[test]
    fn test_circuit_starts_closed() {
        let mgr = CircuitBreakerManager::new(&test_config());
        assert!(mgr.should_allow("mermaid"));
        assert_eq!(mgr.get_state("mermaid"), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_opens_after_threshold() {
        let mgr = CircuitBreakerManager::new(&test_config());
        // 3 consecutive failures should open the circuit
        for _ in 0..3 {
            mgr.record_failure("mermaid");
        }
        assert_eq!(mgr.get_state("mermaid"), CircuitState::Open);
        assert!(!mgr.should_allow("mermaid"));
    }

    #[test]
    fn test_success_resets_failure_count() {
        let mgr = CircuitBreakerManager::new(&test_config());
        mgr.record_failure("mermaid");
        mgr.record_failure("mermaid");
        mgr.record_success("mermaid"); // reset
        mgr.record_failure("mermaid"); // only 1 failure now
        assert_eq!(mgr.get_state("mermaid"), CircuitState::Closed);
    }

    #[test]
    fn test_independent_providers() {
        let mgr = CircuitBreakerManager::new(&test_config());
        for _ in 0..3 {
            mgr.record_failure("mermaid");
        }
        assert_eq!(mgr.get_state("mermaid"), CircuitState::Open);
        assert_eq!(mgr.get_state("graphviz"), CircuitState::Closed);
        assert!(mgr.should_allow("graphviz"));
    }

    #[test]
    fn test_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 1,
            reset_timeout_secs: 0, // instant timeout for testing
        };
        let mgr = CircuitBreakerManager::new(&config);
        mgr.record_failure("mermaid");
        assert_eq!(mgr.get_state("mermaid"), CircuitState::Open);

        // With reset_timeout_secs = 0, should transition to HalfOpen immediately
        assert!(mgr.should_allow("mermaid"));
        assert_eq!(mgr.get_state("mermaid"), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_success_closes() {
        let config = CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 1,
            reset_timeout_secs: 0,
        };
        let mgr = CircuitBreakerManager::new(&config);
        mgr.record_failure("mermaid");
        mgr.should_allow("mermaid"); // transitions to HalfOpen
        mgr.record_success("mermaid");
        assert_eq!(mgr.get_state("mermaid"), CircuitState::Closed);
    }

    #[test]
    fn test_get_all_states() {
        let mgr = CircuitBreakerManager::new(&test_config());
        mgr.should_allow("mermaid");
        mgr.should_allow("graphviz");
        let states = mgr.get_all_states();
        assert_eq!(states.len(), 2);
    }
}
