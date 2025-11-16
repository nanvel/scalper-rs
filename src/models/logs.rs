use crate::models::Timestamp;
use console::{Term, style};
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
}

impl Log {
    pub fn new(level: LogLevel, message: String) -> Self {
        Log {
            level,
            message,
            created_at: Timestamp::now(),
        }
    }
}

pub struct LogManager {
    receiver: Receiver<Log>,
    term: Term,
}

impl LogManager {
    pub fn new(receiver: Receiver<Log>, term: Term) -> Self {
        LogManager { receiver, term }
    }

    pub fn update(&mut self) {
        while let Ok(alert) = self.receiver.try_recv() {
            match alert.level {
                LogLevel::Info => {
                    let _ = self.term.write_line(&format!(
                        "[INFO] {} {}",
                        alert.created_at.to_utc_string(),
                        alert.message
                    ));
                }
                LogLevel::Error => {
                    let _ = self.term.write_line(&format!(
                        "{} {} {}",
                        style("[ERROR]").red(),
                        alert.created_at.to_utc_string(),
                        alert.message
                    ));
                }
            }
        }
    }
}
