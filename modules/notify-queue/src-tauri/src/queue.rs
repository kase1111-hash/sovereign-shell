//! In-memory notification queue with grouping and filtering.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Priority level for a notification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// A single notification in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub source: String,
    pub title: String,
    pub body: String,
    pub priority: Priority,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
    pub dismissed: bool,
}

impl Notification {
    pub fn new(source: &str, title: &str, body: &str, priority: Priority) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            priority,
            timestamp: Utc::now(),
            read: false,
            dismissed: false,
        }
    }
}

/// Grouped notifications for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationGroup {
    pub source: String,
    pub notifications: Vec<Notification>,
    pub unread_count: usize,
}

/// The notification queue — holds active (not dismissed) notifications.
pub struct NotificationQueue {
    items: VecDeque<Notification>,
    max_size: usize,
}

impl NotificationQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Push a new notification into the queue.
    pub fn push(&mut self, notif: Notification) {
        if self.items.len() >= self.max_size {
            // Drop oldest non-critical notification
            if let Some(idx) = self
                .items
                .iter()
                .position(|n| n.priority != Priority::Critical)
            {
                self.items.remove(idx);
            }
        }
        self.items.push_back(notif);
    }

    /// Get all active (non-dismissed) notifications, newest first.
    pub fn get_all(&self) -> Vec<Notification> {
        let mut result: Vec<_> = self.items.iter().filter(|n| !n.dismissed).cloned().collect();
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        result
    }

    /// Get notifications grouped by source.
    pub fn get_grouped(&self) -> Vec<NotificationGroup> {
        let active = self.get_all();
        let mut groups: std::collections::HashMap<String, Vec<Notification>> =
            std::collections::HashMap::new();

        for notif in active {
            groups
                .entry(notif.source.clone())
                .or_default()
                .push(notif);
        }

        let mut result: Vec<NotificationGroup> = groups
            .into_iter()
            .map(|(source, notifications)| {
                let unread_count = notifications.iter().filter(|n| !n.read).count();
                NotificationGroup {
                    source,
                    notifications,
                    unread_count,
                }
            })
            .collect();

        // Sort groups by most recent notification
        result.sort_by(|a, b| {
            let a_latest = a.notifications.first().map(|n| n.timestamp);
            let b_latest = b.notifications.first().map(|n| n.timestamp);
            b_latest.cmp(&a_latest)
        });

        result
    }

    /// Mark a notification as read.
    pub fn mark_read(&mut self, id: &str) {
        if let Some(n) = self.items.iter_mut().find(|n| n.id == id) {
            n.read = true;
        }
    }

    /// Dismiss a single notification.
    pub fn dismiss(&mut self, id: &str) {
        if let Some(n) = self.items.iter_mut().find(|n| n.id == id) {
            n.dismissed = true;
        }
    }

    /// Dismiss all notifications from a specific source.
    pub fn dismiss_by_source(&mut self, source: &str) {
        for n in self.items.iter_mut() {
            if n.source == source {
                n.dismissed = true;
            }
        }
    }

    /// Dismiss all notifications.
    pub fn dismiss_all(&mut self) {
        for n in self.items.iter_mut() {
            n.dismissed = true;
        }
    }

    /// Get count of unread notifications.
    pub fn unread_count(&self) -> usize {
        self.items
            .iter()
            .filter(|n| !n.dismissed && !n.read)
            .count()
    }

    /// Remove dismissed notifications (garbage collect).
    pub fn gc(&mut self) {
        self.items.retain(|n| !n.dismissed);
    }
}
