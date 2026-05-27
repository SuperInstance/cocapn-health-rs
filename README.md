# cocapn-health-rs

Rust port of [cocapn-health](https://github.com/SuperInstance/cocapn-health) — fleet service health checker with monitoring, alerting, and reporting.

## Features

- **`HealthChecker`** — TCP/HTTP health checks for fleet services
- **`HealthMonitor`** — Track service health over time with healthy/degraded/unhealthy classification
- **`AlertManager`** — Alert rules with severity levels and escalation
- **`HealthReport`** — JSON, Markdown, and one-line reports
- **`CheckRegistry`** — Custom check registry with tag-based filtering
- **`CheckBuilder`** — Fluent API for building custom checks

## Usage

```rust
use cocapn_health::{ServiceDef, HealthChecker, CheckResult};
use cocapn_health::monitor::HealthMonitor;
use cocapn_health::alert::{AlertManager, AlertRule, AlertSeverity, is_down};

let services = vec![
    ServiceDef {
        name: "api".into(), host: "127.0.0.1".into(), port: 8080,
        ..Default::default()
    },
];

// Simple check
let checker = HealthChecker::new(services.clone());
let results = checker.check_all();
println!("{}", HealthChecker::report(&results, "markdown"));

// Monitor over time
let mut monitor = HealthMonitor::new(services.clone());
let results = monitor.check();
println!("Status: {:?}", monitor.overall_status());

// Alerting
let mut alerts = AlertManager::new();
alerts.add_rule(AlertRule::new("down", |s| is_down(s), AlertSeverity::Critical));
let new_alerts = alerts.evaluate(&monitor.agent_states);
```

## Installation

```toml
[dependencies]
cocapn-health = "0.1.0"
```

## License

MIT
