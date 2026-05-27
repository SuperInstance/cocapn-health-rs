# cocapn-health-rs

Rust port of [cocapn-health](https://github.com/SuperInstance/cocapn-health) — fleet service health checker with monitoring, alerting, and reporting.

## Features

- **`HealthChecker`** — TCP health checks for fleet services
- **`HealthMonitor`** — Track service health over time with healthy/degraded/unhealthy classification
- **`AlertManager`** — Alert rules with severity levels and escalation
- **`HealthReport`** — JSON, Markdown, and one-line reports
- **`CheckRegistry`** — Custom check registry with tag-based filtering
- **`CheckBuilder`** — Fluent API for building custom checks

## Usage

### Basic health check

```rust
use cocapn_health::{ServiceDef, HealthChecker};

let services = vec![
    ServiceDef {
        name: "api".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        ..Default::default()
    },
];

let checker = HealthChecker::new(services);
let results = checker.check_all();
println!("{}", HealthChecker::report(&results, "markdown"));
```

### Monitor over time

```rust
use cocapn_health::monitor::HealthMonitor;

let services = vec![
    ServiceDef {
        name: "api".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        ..Default::default()
    },
];

let mut monitor = HealthMonitor::new(services);
monitor.check();
println!("Status: {:?}", monitor.overall_status());
println!("Failing: {:?}", monitor.failing_agents());
```

### Alerting with rules

```rust
use cocapn_health::alert::{AlertManager, AlertRule, AlertSeverity, is_down};

let mut alerts = AlertManager::new();
alerts.add_rule(AlertRule::new(
    "down",
    |s| is_down(s),
    AlertSeverity::Critical,
));
let new_alerts = alerts.evaluate(&monitor.agent_states);
println!("Active alerts: {}", alerts.active_alerts().len());
```

### Custom checks

```rust
use cocapn_health::check::{CheckRegistry, CustomCheck, CheckBuilder};

let mut registry = CheckRegistry::new();
registry.add(CustomCheck::new("disk", || {
    // check disk space...
    cocapn_health::CheckResult::new("disk", true, 0.0, "OK")
}).with_tags(&["infra"]));

let results = registry.run_tagged("infra");
```

### Reports

```rust
use cocapn_health::report::HealthReport;

let report = HealthReport::from_results(&results);
println!("{}", report.to_json());
println!("{}", report.to_markdown());
println!("{}", report.to_oneline());
```

## Installation

```toml
[dependencies]
cocapn-health = "0.1.0"
```

## License

MIT
