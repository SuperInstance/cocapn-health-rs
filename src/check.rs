//! Custom check registry and builder.

use std::collections::HashMap;

use crate::CheckResult;

/// A user-defined health check.
pub struct CustomCheck {
    pub name: String,
    pub timeout: f64,
    pub tags: Vec<String>,
    func: Box<dyn Fn() -> CheckResult>,
}

impl CustomCheck {
    pub fn new<F: Fn() -> CheckResult + 'static>(name: &str, func: F) -> Self {
        Self {
            name: name.into(),
            timeout: 5.0,
            tags: Vec::new(),
            func: Box::new(func),
        }
    }

    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn run(&self) -> CheckResult {
        (self.func)()
    }
}

/// Registry for custom health checks.
pub struct CheckRegistry {
    checks: HashMap<String, CustomCheck>,
}

impl CheckRegistry {
    pub fn new() -> Self {
        Self { checks: HashMap::new() }
    }

    pub fn add(&mut self, check: CustomCheck) {
        self.checks.insert(check.name.clone(), check);
    }

    pub fn remove(&mut self, name: &str) {
        self.checks.remove(name);
    }

    pub fn run(&self, name: &str) -> CheckResult {
        match self.checks.get(name) {
            Some(check) => check.run(),
            None => CheckResult::new(name, false, 0.0, &format!("ERROR | unknown check '{}'", name)),
        }
    }

    pub fn run_all(&self) -> Vec<CheckResult> {
        self.checks.values().map(|c| c.run()).collect()
    }

    pub fn run_tagged(&self, tag: &str) -> Vec<CheckResult> {
        self.checks.values()
            .filter(|c| c.tags.iter().any(|t| t == tag))
            .map(|c| c.run())
            .collect()
    }

    pub fn check_names(&self) -> Vec<&str> {
        self.checks.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize { self.checks.len() }
    pub fn is_empty(&self) -> bool { self.checks.is_empty() }
}

/// Fluent builder for creating custom checks.
pub struct CheckBuilder {
    name: String,
    timeout: f64,
    tags: Vec<String>,
}

impl CheckBuilder {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), timeout: 5.0, tags: Vec::new() }
    }

    pub fn timeout(mut self, seconds: f64) -> Self {
        self.timeout = seconds;
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn build<F: Fn() -> CheckResult + 'static>(self, func: F) -> CustomCheck {
        CustomCheck {
            name: self.name,
            timeout: self.timeout,
            tags: self.tags,
            func: Box::new(func),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_check() {
        let check = CustomCheck::new("test", || CheckResult::new("test", true, 5.0, "UP"));
        let result = check.run();
        assert!(result.ok);
    }

    #[test]
    fn test_registry_run_all() {
        let mut reg = CheckRegistry::new();
        reg.add(CustomCheck::new("a", || CheckResult::new("a", true, 1.0, "UP")));
        reg.add(CustomCheck::new("b", || CheckResult::new("b", false, 2.0, "DOWN")));
        let results = reg.run_all();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_registry_run_tagged() {
        let mut reg = CheckRegistry::new();
        reg.add(CustomCheck::new("a", || CheckResult::new("a", true, 1.0, "UP")).with_tags(&["infra"]));
        reg.add(CustomCheck::new("b", || CheckResult::new("b", true, 1.0, "UP")).with_tags(&["app"]));
        let results = reg.run_tagged("infra");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "a");
    }

    #[test]
    fn test_check_builder() {
        let check = CheckBuilder::new("my_check")
            .timeout(10.0)
            .tag("infra")
            .build(|| CheckResult::new("my_check", true, 1.0, "UP"));
        assert_eq!(check.name, "my_check");
        assert_eq!(check.timeout, 10.0);
        assert_eq!(check.tags.len(), 1);
    }

    #[test]
    fn test_registry_unknown() {
        let reg = CheckRegistry::new();
        let result = reg.run("unknown");
        assert!(!result.ok);
    }
}
