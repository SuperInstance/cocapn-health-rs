//! cocapn-health — Fleet service health checker with monitoring, alerting, and reporting.

pub mod monitor;
pub mod alert;
pub mod report;
pub mod check;

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// A service definition for health checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_timeout")]
    pub timeout: f64,
    pub expect_status: Option<u16>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub extract: Option<HashMap<String, String>>,
}

fn default_path() -> String { "/".into() }
fn default_method() -> String { "GET".into() }
fn default_timeout() -> f64 { 5.0 }

/// Result of a single health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
    pub latency_ms: f64,
    pub status: String,
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

impl CheckResult {
    pub fn new(name: &str, ok: bool, latency_ms: f64, status: &str) -> Self {
        Self {
            name: name.into(),
            ok,
            latency_ms,
            status: status.into(),
            details: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: serde_json::Value) -> Self {
        self.details.insert(key.into(), value);
        self
    }
}

/// Health checker for a fleet of services.
pub struct HealthChecker {
    pub services: Vec<ServiceDef>,
}

impl HealthChecker {
    pub fn new(services: Vec<ServiceDef>) -> Self {
        Self { services }
    }

    /// Check a single service (TCP connectivity check).
    pub fn check_one(&self, svc: &ServiceDef) -> CheckResult {
        let start = Instant::now();
        let addr = format!("{}:{}", svc.host, svc.port);

        match std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()),
            std::time::Duration::from_secs_f64(svc.timeout),
        ) {
            Ok(_) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                CheckResult::new(&svc.name, true, latency, &format!("UP | TCP connected to {}:{}", svc.host, svc.port))
            }
            Err(e) => {
                let latency = start.elapsed().as_secs_f64() * 1000.0;
                CheckResult::new(&svc.name, false, latency, &format!("DOWN | {}", e))
            }
        }
    }

    /// Check all services.
    pub fn check_all(&self) -> Vec<CheckResult> {
        self.services.iter().map(|svc| self.check_one(svc)).collect()
    }

    /// Generate a report string.
    pub fn report(results: &[CheckResult], format: &str) -> String {
        let up = results.iter().filter(|r| r.ok).count();
        let down = results.len() - up;

        match format {
            "json" => serde_json::to_string_pretty(&serde_json::json!({
                "summary": {"total": results.len(), "up": up, "down": down},
                "services": results
            })).unwrap_or_default(),
            "markdown" | "md" => {
                let mut lines = vec![
                    "# Fleet Health Report".into(),
                    String::new(),
                    format!("**{}/{} services UP** — {} down", up, results.len(), down),
                    String::new(),
                    "| Service | Status | Latency |".into(),
                    "|---------|--------|---------|".into(),
                ];
                for r in results {
                    let emoji = if r.ok { "🟢" } else { "🔴" };
                    lines.push(format!("| {} {} | {} | {:.0}ms |", emoji, r.name, r.status, r.latency_ms));
                }
                lines.join("\n")
            }
            "oneline" => {
                let status = if down == 0 { "✅".into() } else { format!("⚠️ {} down", down) };
                format!("Fleet: {}/{} up {}", up, results.len(), status)
            }
            _ => String::new(),
        }
    }
}
