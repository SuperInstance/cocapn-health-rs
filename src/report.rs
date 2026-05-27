//! Health reporting — JSON, Markdown, one-line output.

use serde::{Deserialize, Serialize};

use crate::CheckResult;

/// A snapshot of system health at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: String,
    pub checked_at: String,
    pub total_services: usize,
    pub services_up: usize,
    pub services_down: usize,
    pub failing: Vec<String>,
    pub agent_summaries: Vec<serde_json::Value>,
    pub alerts: Vec<serde_json::Value>,
}

impl HealthReport {
    pub fn from_results(results: &[CheckResult]) -> Self {
        let up = results.iter().filter(|r| r.ok).count();
        let down = results.len() - up;
        let failing: Vec<String> = results
            .iter()
            .filter(|r| !r.ok)
            .map(|r| r.name.clone())
            .collect();

        let status = if down == 0 {
            "healthy"
        } else if down <= up {
            "degraded"
        } else {
            "unhealthy"
        };

        Self {
            status: status.into(),
            checked_at: chrono_now(),
            total_services: results.len(),
            services_up: up,
            services_down: down,
            failing,
            agent_summaries: Vec::new(),
            alerts: Vec::new(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            "# Health Report".into(),
            String::new(),
            format!(
                "**Status:** {} | **{}/{}** services up | **{}** down",
                self.status.to_uppercase(),
                self.services_up,
                self.total_services,
                self.services_down
            ),
            format!("**Checked:** {}", self.checked_at),
            String::new(),
        ];

        if !self.failing.is_empty() {
            lines.push("## Failing Services".into());
            for name in &self.failing {
                lines.push(format!("- **{}**", name));
            }
            lines.push(String::new());
        }

        lines.join("\n")
    }

    pub fn to_oneline(&self) -> String {
        let emoji = match self.status.as_str() {
            "healthy" => "✅",
            "degraded" => "⚠️",
            _ => "🔴",
        };
        let failing_str = if self.failing.is_empty() {
            String::new()
        } else {
            format!(", failing: {}", self.failing.join(", "))
        };
        format!(
            "{} {} | {}/{} up{}",
            emoji,
            self.status.to_uppercase(),
            self.services_up,
            self.total_services,
            failing_str
        )
    }
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}s since epoch", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_from_results_all_up() {
        let results = vec![
            CheckResult::new("a", true, 10.0, "UP"),
            CheckResult::new("b", true, 20.0, "UP"),
        ];
        let report = HealthReport::from_results(&results);
        assert_eq!(report.status, "healthy");
        assert_eq!(report.services_up, 2);
        assert_eq!(report.services_down, 0);
        assert!(report.failing.is_empty());
    }

    #[test]
    fn test_report_from_results_some_down() {
        let results = vec![
            CheckResult::new("a", true, 10.0, "UP"),
            CheckResult::new("b", false, 5.0, "DOWN"),
        ];
        let report = HealthReport::from_results(&results);
        assert_eq!(report.status, "degraded");
        assert_eq!(report.failing.len(), 1);
    }

    #[test]
    fn test_to_json() {
        let results = vec![CheckResult::new("a", true, 10.0, "UP")];
        let report = HealthReport::from_results(&results);
        let json = report.to_json();
        assert!(json.contains("healthy"));
    }

    #[test]
    fn test_to_markdown() {
        let results = vec![CheckResult::new("a", true, 10.0, "UP")];
        let report = HealthReport::from_results(&results);
        let md = report.to_markdown();
        assert!(md.contains("# Health Report"));
    }

    #[test]
    fn test_to_oneline() {
        let results = vec![CheckResult::new("a", true, 10.0, "UP")];
        let report = HealthReport::from_results(&results);
        let line = report.to_oneline();
        assert!(line.contains("✅"));
    }
}
