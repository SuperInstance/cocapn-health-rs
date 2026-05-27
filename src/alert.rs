//! Alert management with severity levels and escalation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::monitor::AgentState;

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertState {
    Pending,
    Firing,
    Resolved,
    Escalated,
}

/// A health alert instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub rule_name: String,
    pub agent_name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub message: String,
    pub fired_at: f64,
    pub resolved_at: Option<f64>,
    pub escalation_count: usize,
}

/// Built-in condition: fires when the agent is down.
pub fn is_down(state: &AgentState) -> bool {
    state.last_ok == Some(false)
}

/// Built-in condition: fires after N consecutive failures.
pub fn consecutive_failures(state: &AgentState, threshold: usize) -> bool {
    state.consecutive_failures >= threshold
}

/// Built-in condition: fires when availability drops below threshold.
pub fn low_availability(state: &AgentState, threshold: f64) -> bool {
    state.total_checks >= 3 && state.availability() < threshold
}

/// Built-in condition: fires when average latency exceeds threshold.
pub fn high_latency(state: &AgentState, threshold_ms: f64) -> bool {
    state.total_checks >= 2 && state.avg_latency_ms() > threshold_ms
}

/// Manages alert rules and evaluates them against agent states.
pub struct AlertManager {
    rules: Vec<AlertRule>,
    alerts: HashMap<String, HealthAlert>, // key: "rule_name:agent_name"
}

/// An alert rule definition.
pub struct AlertRule {
    pub name: String,
    pub severity: AlertSeverity,
    pub escalation_after_failures: usize,
    pub message_template: String,
    pub condition: Box<dyn Fn(&AgentState) -> bool>,
}

impl std::fmt::Debug for AlertRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertRule")
            .field("name", &self.name)
            .finish()
    }
}

impl AlertRule {
    pub fn new<F: Fn(&AgentState) -> bool + 'static>(
        name: &str,
        condition: F,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            name: name.into(),
            severity,
            condition: Box::new(condition),
            escalation_after_failures: 3,
            message_template: "{name} is failing".into(),
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            alerts: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    pub fn remove_rule(&mut self, name: &str) {
        self.rules.retain(|r| r.name != name);
    }

    /// Evaluate all rules against all agent states. Returns newly fired alerts.
    pub fn evaluate(&mut self, agent_states: &HashMap<String, AgentState>) -> Vec<HealthAlert> {
        let mut newly_fired = Vec::new();

        for rule in &self.rules {
            for (agent_name, state) in agent_states {
                let key = format!("{}:{}", rule.name, agent_name);
                let triggered = (rule.condition)(state);

                if triggered {
                    let existing = self.alerts.get(&key).cloned();
                    let should_fire = match &existing {
                        Some(a)
                            if a.state == AlertState::Firing
                                || a.state == AlertState::Escalated =>
                        {
                            false
                        }
                        Some(a) if a.state == AlertState::Resolved => true,
                        _ => true,
                    };

                    if should_fire {
                        let alert = HealthAlert {
                            rule_name: rule.name.clone(),
                            agent_name: agent_name.clone(),
                            severity: rule.severity,
                            state: AlertState::Firing,
                            message: rule.message_template.replace("{name}", &state.name),
                            fired_at: existing.as_ref().map_or(0.0, |a| a.fired_at),
                            resolved_at: None,
                            escalation_count: 0,
                        };
                        self.alerts.insert(key, alert.clone());
                        newly_fired.push(alert);
                    } else if let Some(existing) = self.alerts.get_mut(&key) {
                        // Check escalation
                        if state.consecutive_failures >= rule.escalation_after_failures
                            && existing.escalation_count == 0
                        {
                            existing.state = AlertState::Escalated;
                            existing.escalation_count = 1;
                            existing.message = format!("{} (ESCALATED)", existing.message);
                        }
                    }
                } else if let Some(existing) = self.alerts.get_mut(&key) {
                    if existing.state == AlertState::Firing
                        || existing.state == AlertState::Escalated
                    {
                        existing.state = AlertState::Resolved;
                        existing.resolved_at = Some(0.0);
                    }
                }
            }
        }

        newly_fired
    }

    pub fn active_alerts(&self) -> Vec<&HealthAlert> {
        self.alerts
            .values()
            .filter(|a| a.state == AlertState::Firing || a.state == AlertState::Escalated)
            .collect()
    }

    pub fn all_alerts(&self) -> Vec<&HealthAlert> {
        self.alerts.values().collect()
    }

    pub fn clear_resolved(&mut self) -> usize {
        let to_remove: Vec<String> = self
            .alerts
            .iter()
            .filter(|(_, a)| a.state == AlertState::Resolved)
            .map(|(k, _)| k.clone())
            .collect();
        let count = to_remove.len();
        for k in to_remove {
            self.alerts.remove(&k);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CheckResult;

    fn make_agent(name: &str, ok: bool, failures: usize) -> AgentState {
        let mut state = AgentState::new(name);
        for _ in 0..failures {
            state.update(&CheckResult::new(name, false, 1.0, "DOWN"));
        }
        if ok {
            state.update(&CheckResult::new(name, true, 1.0, "UP"));
        }
        state
    }

    #[test]
    fn test_is_down() {
        let state = make_agent("svc", false, 1);
        assert!(is_down(&state));
        let state2 = make_agent("svc", true, 0);
        assert!(!is_down(&state2));
    }

    #[test]
    fn test_consecutive_failures() {
        let state = make_agent("svc", false, 5);
        assert!(consecutive_failures(&state, 3));
        let state2 = make_agent("svc", false, 2);
        assert!(!consecutive_failures(&state2, 3));
    }

    #[test]
    fn test_alert_manager_fire_and_resolve() {
        let mut mgr = AlertManager::new();
        mgr.add_rule(AlertRule::new(
            "down",
            |s| is_down(s),
            AlertSeverity::Critical,
        ));

        let mut states = HashMap::new();
        states.insert("svc".into(), make_agent("svc", false, 1));
        let fired = mgr.evaluate(&states);
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].state, AlertState::Firing);

        // Now resolve
        states.insert("svc".into(), make_agent("svc", true, 0));
        mgr.evaluate(&states);
        assert_eq!(mgr.active_alerts().len(), 0);
    }

    #[test]
    fn test_clear_resolved() {
        let mut mgr = AlertManager::new();
        mgr.add_rule(AlertRule::new(
            "down",
            |s| is_down(s),
            AlertSeverity::Critical,
        ));

        let mut states = HashMap::new();
        states.insert("svc".into(), make_agent("svc", false, 1));
        mgr.evaluate(&states);

        states.insert("svc".into(), make_agent("svc", true, 0));
        mgr.evaluate(&states);

        let cleared = mgr.clear_resolved();
        assert_eq!(cleared, 1);
    }
}
