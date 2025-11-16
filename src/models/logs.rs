use crate::models::Timestamp;
use console::{Term, style};
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Error(bool, String), // critical, message
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Ok,
    Warning(String),
    Critical(String),
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
    pub status: Status,
}

impl LogManager {
    pub fn new(receiver: Receiver<Log>, term: Term) -> Self {
        LogManager {
            receiver,
            term,
            status: Status::Ok,
        }
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
                LogLevel::Error(is_critical, message) => {
                    let _ = self.term.write_line(&format!(
                        "{} {} {}",
                        style("[ERROR]").red(),
                        alert.created_at.to_utc_string(),
                        alert.message
                    ));
                    if is_critical {
                        self.status = Status::Critical(message);
                    } else if let Status::Ok = self.status {
                        self.status = Status::Warning(message);
                    }
                }
            }
        }
    }
}
