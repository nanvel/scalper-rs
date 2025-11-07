use crate::models::Timestamp;
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct Log {
    pub level: LogLevel,
    pub message: String,
    pub created_at: Timestamp,
    pub duration: u32,
}

impl Log {
    pub fn new(level: LogLevel, message: String, duration: Option<u32>) -> Self {
        Log {
            level: LogLevel::Info,
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

pub struct LogManager {
    active_logs: VecDeque<Log>,
    receiver: Receiver<Log>,
}

impl LogManager {
    pub fn new(receiver: Receiver<Log>) -> Self {
        LogManager {
            active_logs: VecDeque::new(),
            receiver,
        }
    }

    pub fn update(&mut self) {
        while let Ok(alert) = self.receiver.try_recv() {
            self.active_logs.push_back(alert);
        }

        self.active_logs.retain(|alert| !alert.is_expired());
    }

    pub fn get_active_alerts(&self) -> &VecDeque<Log> {
        &self.active_logs
    }
}
