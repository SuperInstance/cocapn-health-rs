use cocapn_health::*;
use std::collections::HashMap;

#[test]
fn test_service_def_default() {
    let sd = ServiceDef::default();
    assert_eq!(sd.path, "/");
    assert_eq!(sd.method, "GET");
    assert_eq!(sd.timeout, 5.0);
}

#[test]
fn test_check_result_creation() {
    let cr = CheckResult::new("test", true, 42.5, "UP");
    assert_eq!(cr.name, "test");
    assert!(cr.ok);
    assert!((cr.latency_ms - 42.5).abs() < 1e-10);
    assert!(cr.details.is_empty());
}

#[test]
fn test_check_result_with_detail() {
    let cr = CheckResult::new("svc", true, 10.0, "UP")
        .with_detail("status_code", serde_json::json!(200));
    assert_eq!(cr.details.len(), 1);
    assert_eq!(cr.details["status_code"], 200);
}

#[test]
fn test_health_checker_check_all() {
    let services = vec![ServiceDef {
        name: "unreachable".into(),
        host: "192.0.2.1".into(), // RFC 5737 test address
        port: 1,
        timeout: 0.1,
        ..Default::default()
    }];
    let checker = HealthChecker::new(services);
    let results = checker.check_all();
    assert_eq!(results.len(), 1);
    assert!(!results[0].ok); // Should be down
}

#[test]
fn test_report_markdown() {
    let results = vec![
        CheckResult::new("api", true, 15.0, "UP"),
        CheckResult::new("db", false, 5000.0, "DOWN"),
    ];
    let report = HealthChecker::report(&results, "markdown");
    assert!(report.contains("# Fleet Health Report"));
    assert!(report.contains("api"));
    assert!(report.contains("db"));
}

#[test]
fn test_report_json() {
    let results = vec![CheckResult::new("svc", true, 10.0, "UP")];
    let report = HealthChecker::report(&results, "json");
    let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(parsed["summary"]["total"], 1);
    assert_eq!(parsed["summary"]["up"], 1);
}

#[test]
fn test_report_oneline() {
    let results = vec![CheckResult::new("svc", true, 10.0, "UP")];
    let report = HealthChecker::report(&results, "oneline");
    assert!(report.contains("1/1 up"));
}

#[test]
fn test_service_def_serialize_deserialize() {
    let sd = ServiceDef {
        name: "test".into(),
        host: "localhost".into(),
        port: 8080,
        ..Default::default()
    };
    let json = serde_json::to_string(&sd).unwrap();
    let sd2: ServiceDef = serde_json::from_str(&json).unwrap();
    assert_eq!(sd2.name, "test");
    assert_eq!(sd2.port, 8080);
}
