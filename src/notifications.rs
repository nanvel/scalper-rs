use crate::models::Timestamp;
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub level: NotificationLevel,
    pub message: String,
    pub created_at: Timestamp,
    pub duration: u32,
}

impl Notification {
    pub fn new(level: NotificationLevel, message: String, duration: Option<u32>) -> Self {
        Notification {
            level,
            message,
            created_at: Timestamp::now(),
            duration: duration.unwrap_or(10),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Timestamp::now();
        let elapsed = now.seconds() - self.created_at.seconds();
        elapsed >= self.duration as u64
    }
}

pub struct AlertManager {
    active_alerts: VecDeque<Notification>,
    receiver: Receiver<Notification>,
}

impl AlertManager {
    pub fn new(receiver: Receiver<Notification>) -> Self {
        AlertManager {
            active_alerts: VecDeque::new(),
            receiver,
        }
    }

    pub fn update(&mut self) {
        while let Ok(alert) = self.receiver.try_recv() {
            self.active_alerts.push_back(alert);
        }

        self.active_alerts.retain(|alert| !alert.is_expired());
    }

    pub fn get_active_alerts(&self) -> &VecDeque<Notification> {
        &self.active_alerts
    }
}
