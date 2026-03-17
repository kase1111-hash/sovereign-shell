//! Per-app notification filtering rules.

use crate::queue::Priority;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Action to take when a notification arrives from a source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    /// Show toast and add to queue.
    Show,
    /// Suppress toast but add to queue.
    Silent,
    /// Block entirely — do not show or queue.
    Block,
}

impl Default for RuleAction {
    fn default() -> Self {
        RuleAction::Show
    }
}

/// A filtering rule for a specific application/source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRule {
    pub source: String,
    pub action: RuleAction,
    #[serde(default = "default_duration")]
    pub duration_seconds: u32,
    pub priority: Option<Priority>,
}

fn default_duration() -> u32 {
    5
}

/// Default rule applied when no source-specific rule matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultRule {
    pub action: RuleAction,
    pub duration_seconds: u32,
}

impl Default for DefaultRule {
    fn default() -> Self {
        Self {
            action: RuleAction::Show,
            duration_seconds: 5,
        }
    }
}

/// The rules engine — holds default + per-source rules.
pub struct RulesEngine {
    default_rule: DefaultRule,
    rules: HashMap<String, NotificationRule>,
}

/// Result of applying rules to a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub action: RuleAction,
    pub duration_seconds: u32,
    pub effective_priority: Priority,
}

impl RulesEngine {
    pub fn new(default_rule: DefaultRule) -> Self {
        Self {
            default_rule,
            rules: HashMap::new(),
        }
    }

    /// Load rules from a list.
    pub fn load_rules(&mut self, rules: Vec<NotificationRule>) {
        self.rules.clear();
        for rule in rules {
            self.rules.insert(rule.source.clone(), rule);
        }
    }

    /// Evaluate what to do with a notification from a given source.
    pub fn evaluate(&self, source: &str, priority: &Priority) -> RuleResult {
        // Check for source-specific rule (case-insensitive match)
        let source_lower = source.to_lowercase();
        let matching_rule = self
            .rules
            .iter()
            .find(|(k, _)| k.to_lowercase() == source_lower)
            .map(|(_, v)| v);

        if let Some(rule) = matching_rule {
            let effective_priority = rule.priority.clone().unwrap_or_else(|| priority.clone());

            // High/Critical priority notifications show even in silent mode
            let action = if (effective_priority == Priority::High
                || effective_priority == Priority::Critical)
                && rule.action == RuleAction::Silent
            {
                RuleAction::Show
            } else {
                rule.action.clone()
            };

            RuleResult {
                action,
                duration_seconds: rule.duration_seconds,
                effective_priority,
            }
        } else {
            RuleResult {
                action: self.default_rule.action.clone(),
                duration_seconds: self.default_rule.duration_seconds,
                effective_priority: priority.clone(),
            }
        }
    }

    /// Get all rules for display.
    pub fn get_rules(&self) -> Vec<NotificationRule> {
        self.rules.values().cloned().collect()
    }

    /// Add or update a rule.
    pub fn set_rule(&mut self, rule: NotificationRule) {
        self.rules.insert(rule.source.clone(), rule);
    }

    /// Remove a rule.
    pub fn remove_rule(&mut self, source: &str) {
        self.rules.remove(source);
    }

    /// Get the default rule.
    pub fn get_default(&self) -> &DefaultRule {
        &self.default_rule
    }

    /// Update the default rule.
    pub fn set_default(&mut self, rule: DefaultRule) {
        self.default_rule = rule;
    }

    /// Check if silent mode should be overridden for high-priority.
    pub fn is_silent_mode(&self) -> bool {
        self.default_rule.action == RuleAction::Silent
    }
}
