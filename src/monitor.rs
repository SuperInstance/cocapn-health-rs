//! HealthMonitor — track multiple agents/services over time.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{CheckResult, HealthChecker, ServiceDef};

/// Overall system health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Tracks the health state of a single agent/service over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub name: String,
    pub last_ok: Option<bool>,
    pub consecutive_failures: usize,
    pub consecutive_successes: usize,
    pub total_checks: usize,
    pub total_failures: usize,
    pub last_status: Option<String>,
    #[serde(skip)]
    history: Vec<CheckResult>,
}

impl AgentState {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            last_ok: None,
            consecutive_failures: 0,
            consecutive_successes: 0,
            total_checks: 0,
            total_failures: 0,
            last_status: None,
            history: Vec::new(),
        }
    }

    pub fn update(&mut self, result: &CheckResult) {
        self.last_ok = Some(result.ok);
        self.last_status = Some(result.status.clone());
        self.total_checks += 1;

        if result.ok {
            self.consecutive_failures = 0;
            self.consecutive_successes += 1;
        } else {
            self.consecutive_successes = 0;
            self.consecutive_failures += 1;
            self.total_failures += 1;
        }

        self.history.push(result.clone());
        if self.history.len() > 100 {
            let drain = self.history.len() - 100;
            self.history.drain(..drain);
        }
    }

    pub fn availability(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        ((self.total_checks - self.total_failures) as f64 / self.total_checks as f64) * 100.0
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().map(|r| r.latency_ms).sum::<f64>() / self.history.len() as f64
    }
}

/// Monitors fleet services and tracks health state over time.
pub struct HealthMonitor {
    services: Vec<ServiceDef>,
    checker: HealthChecker,
    degraded_threshold: f64,
    unhealthy_threshold: f64,
    pub agent_states: HashMap<String, AgentState>,
    pub system_states: HashMap<String, AgentState>,
    check_count: usize,
}

impl HealthMonitor {
    pub fn new(services: Vec<ServiceDef>) -> Self {
        let agent_states = services
            .iter()
            .map(|s| (s.name.clone(), AgentState::new(&s.name)))
            .collect();
        Self {
            checker: HealthChecker::new(services.clone()),
            services,
            degraded_threshold: 0.5,
            unhealthy_threshold: 0.2,
            agent_states,
            system_states: HashMap::new(),
            check_count: 0,
        }
    }

    pub fn with_thresholds(mut self, degraded: f64, unhealthy: f64) -> Self {
        self.degraded_threshold = degraded;
        self.unhealthy_threshold = unhealthy;
        self
    }

    pub fn check(&mut self) -> Vec<CheckResult> {
        let results = self.checker.check_all();
        self.check_count += 1;

        for result in &results {
            if let Some(state) = self.agent_states.get_mut(&result.name) {
                state.update(result);
            }
        }

        results
    }

    pub fn overall_status(&self) -> HealthStatus {
        let total = self.services.len();
        if total == 0 {
            return HealthStatus::Healthy;
        }

        let up = self
            .agent_states
            .values()
            .filter(|s| s.last_ok == Some(true))
            .count();
        let ratio = up as f64 / total as f64;

        if ratio >= self.degraded_threshold {
            HealthStatus::Healthy
        } else if ratio >= self.unhealthy_threshold {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    pub fn failing_agents(&self) -> Vec<&str> {
        self.agent_states
            .iter()
            .filter(|(_, s)| s.last_ok == Some(false))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    pub fn check_count(&self) -> usize {
        self.check_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServiceDef;

    #[test]
    fn test_agent_state_update_ok() {
        let mut state = AgentState::new("test");
        let result = CheckResult::new("test", true, 10.0, "UP");
        state.update(&result);
        assert_eq!(state.last_ok, Some(true));
        assert_eq!(state.consecutive_successes, 1);
        assert_eq!(state.consecutive_failures, 0);
        assert_eq!(state.total_checks, 1);
    }

    #[test]
    fn test_agent_state_update_fail() {
        let mut state = AgentState::new("test");
        let result = CheckResult::new("test", false, 5.0, "DOWN");
        state.update(&result);
        assert_eq!(state.last_ok, Some(false));
        assert_eq!(state.consecutive_failures, 1);
        assert_eq!(state.total_failures, 1);
    }

    #[test]
    fn test_availability() {
        let mut state = AgentState::new("test");
        state.update(&CheckResult::new("test", true, 1.0, "UP"));
        state.update(&CheckResult::new("test", true, 1.0, "UP"));
        state.update(&CheckResult::new("test", false, 1.0, "DOWN"));
        assert!((state.availability() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_avg_latency() {
        let mut state = AgentState::new("test");
        state.update(&CheckResult::new("test", true, 10.0, "UP"));
        state.update(&CheckResult::new("test", true, 20.0, "UP"));
        assert!((state.avg_latency_ms() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_monitor_overall_status() {
        let svc = ServiceDef {
            name: "test".into(),
            host: "127.0.0.1".into(),
            port: 1,
            path: "/".into(),
            method: "GET".into(),
            timeout: 0.1,
            expect_status: None,
            headers: HashMap::new(),
            extract: None,
        };
        let mut mon = HealthMonitor::new(vec![svc]);
        // Port 1 won't be listening, so all will be "down"
        let results = mon.check();
        assert!(!results[0].ok);
        // With 1/0 services up, ratio=0 < 0.2, so unhealthy
        assert_eq!(mon.overall_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_failing_agents() {
        let svc = ServiceDef {
            name: "test".into(),
            host: "127.0.0.1".into(),
            port: 1,
            path: "/".into(),
            method: "GET".into(),
            timeout: 0.1,
            expect_status: None,
            headers: HashMap::new(),
            extract: None,
        };
        let mut mon = HealthMonitor::new(vec![svc]);
        mon.check();
        assert_eq!(mon.failing_agents().len(), 1);
    }
}
