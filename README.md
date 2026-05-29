# cocapn-health-rs

Fleet service health checker with TCP probing, time-series monitoring, severity-based alerting, and multi-format reporting (JSON, Markdown, one-line).

## What This Gives You

- **`HealthChecker`** — TCP health probes for fleet services with latency tracking
- **`HealthMonitor`** — Track service health over time with healthy/degraded/unhealthy classification
- **`AlertManager`** — Alert rules with severity levels (Critical, Warning, Info) and escalation
- **`HealthReport`** — Formatted output as JSON, Markdown table, or one-line summary
- **`CheckRegistry`** — Custom check registry with tag-based filtering and builder API
- **Zero runtime dependencies** — Pure Rust (serde + serde_json for serialization)

## Quick Start

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

### Monitor over time with alerting

```rust
use cocapn_health::monitor::HealthMonitor;
use cocapn_health::alert::{AlertManager, AlertRule, AlertSeverity, is_down};

let mut monitor = HealthMonitor::new(services);
monitor.check();

let mut alerts = AlertManager::new();
alerts.add_rule(AlertRule::new("down", |s| is_down(s), AlertSeverity::Critical));
let new_alerts = alerts.evaluate(&monitor.agent_states);

println!("Status: {:?}", monitor.overall_status());
println!("Failing: {:?}", monitor.failing_agents());
println!("Active alerts: {}", alerts.active_alerts().len());
```

### Custom checks with fluent API

```rust
use cocapn_health::check::{CheckRegistry, CheckBuilder};

let mut registry = CheckRegistry::new();
registry.add(
    CheckBuilder::new("disk")
        .run(|| cocapn_health::check::CheckResult::new("disk", true, 0.0, "OK"))
        .with_tags(&["infra"])
        .build()
);
```

## API Reference

### `ServiceDef`

| Field | Description |
|-------|-------------|
| `name` | Service identifier |
| `host` / `port` | TCP endpoint |
| `timeout` | Probe timeout in seconds (default 5.0) |
| `expect_status` | Expected HTTP status code |
| `headers` | Custom HTTP headers |

### `HealthMonitor`

| Method | Description |
|--------|-------------|
| `new(services)` | Create monitor for a list of services |
| `check()` | Run health check on all services |
| `overall_status()` | Aggregated status (Healthy/Degraded/Unhealthy) |
| `failing_agents()` | List of currently failing services |

### `AlertManager`

```rust
AlertManager::new()
alerts.add_rule(rule)             // Register an alert rule
alerts.evaluate(&states)          // Check all rules against current state
alerts.active_alerts()            // Get currently active alerts
alerts.acknowledge(rule_id)       // Acknowledge an alert
```

## How It Fits

- **[cocapn-explain-rs](https://github.com/SuperInstance/cocapn-explain-rs)** — Explain why services were flagged unhealthy
- **[ccc-os](https://github.com/SuperInstance/ccc-os)** — Fleet-wide monitoring uses health data for triage
- **[caching-service-rs](https://github.com/SuperInstance/caching-service-rs)** — Cache health results to avoid re-probing healthy services
- **[fleet-cicd-agent](https://github.com/SuperInstance/fleet-cicd-agent)** — Rollback triggers when health checks fail after deployment

## Testing

20 tests covering health checks, monitoring state transitions, alert rule evaluation, custom checks, and report formatting.

```bash
cargo test
```

## Installation

```toml
[dependencies]
cocapn-health = { git = "https://github.com/SuperInstance/cocapn-health-rs" }
```

```bash
git clone https://github.com/SuperInstance/cocapn-health-rs.git
cd cocapn-health-rs
cargo build
```

## License

MIT

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance) ecosystem.
