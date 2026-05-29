# cocapn-health-rs — Fleet Health Checker (Rust)

[![crates.io](https://img.shields.io/crates/v/placeholder)](https://crates.io/crates/placeholder) [![SuperInstance](https://img.shields.io/badge/SuperInstance-Ecosystem-blue)](https://github.com/SuperInstance)



**TCP probing, time-series monitoring, severity-based alerting, and multi-format reporting for fleet services.**

## What This Gives You

- **TCP health checks** — probe fleet services over TCP with configurable timeouts
- **Time-series tracking** — record and query health data over time
- **Severity-based alerting** — INFO, WARNING, CRITICAL alerts with configurable thresholds
- **Multi-format reports** — JSON, Markdown, or one-line status output
- **Rust-native** — fast, low-memory, suitable for continuous monitoring daemons

## Quick Start

```toml
# Cargo.toml
[dependencies]
cocapn-health-rs = "0.1"
```

```rust
use cocapn_health_rs::{HealthChecker, CheckConfig, Severity};

// Configure health checks
let checker = HealthChecker::new()
    .add_check(CheckConfig {
        name: "api-gateway".into(),
        host: "localhost".into(),
        port: 8080,
        timeout_ms: 5000,
        warning_threshold_ms: 2000,
        critical_threshold_ms: 4000,
    });

// Run checks
let results = checker.run().await;
for result in &results {
    println!("[{}] {} — {}ms", 
        match result.severity {
            Severity::Ok => "OK",
            Severity::Warning => "WARN",
            Severity::Critical => "CRIT",
        },
        result.name,
        result.latency_ms.unwrap_or(0)
    );
}

// Generate report
let report = checker.report(&results);
println!("{}", report.to_markdown());
```

## API Reference

### `HealthChecker` — `add_check(config)`, `run() → Vec<CheckResult>`, `report(results)`
### `CheckConfig { name, host, port, timeout_ms, warning_threshold_ms, critical_threshold_ms }`
### `CheckResult { name, severity, latency_ms, error }`
### `Severity` — `Ok`, `Warning`, `Critical`
### `Report` — `to_json()`, `to_markdown()`, `to_oneline()`

## How It Fits
- [OpenConstruct Documentation](https://github.com/SuperInstance/openconstruct-docs) — ecosystem-wide docs and guides

The Rust-based health checking component for the [SuperInstance fleet](https://github.com/SuperInstance). Runs alongside the Python [fleet-health-monitor](https://github.com/SuperInstance/fleet-health-monitor) for low-level TCP probing.

- **[fleet-health-monitor](https://github.com/SuperInstance/fleet-health-monitor)** — Python fleet health daemon (uses these results)
- **[cocapn-cli](https://github.com/SuperInstance/cocapn-cli)** — Terminal output formatting for reports
- **[ccc-os](https://github.com/SuperInstance/ccc-os)** — Autonomous monitoring (consumes health data)

## Testing

```bash
cargo test
```

## Installation

```toml
[dependencies]
cocapn-health-rs = "0.1"
```

MIT OR Apache-2.0.
