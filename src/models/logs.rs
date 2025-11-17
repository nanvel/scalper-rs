use crate::models::Timestamp;
use console::{Term, style};
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning(String, Option<usize>), // message, show for n seconds
    Error(String),                  // message
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
    warnings_queue: VecDeque<(String, Timestamp)>,
    status: Status,
}

impl LogManager {
    pub fn new(receiver: Receiver<Log>, term: Term) -> Self {
        LogManager {
            receiver,
            term,
            warnings_queue: VecDeque::new(),
            status: Status::Ok,
        }
    }

    pub fn status(&mut self) -> Status {
        if let Status::Critical(message) = &self.status {
            return self.status.clone();
        }
        if let Some((message, remove_at)) = self.warnings_queue.back() {
            let resp = Status::Warning(message.clone());
            if Timestamp::now() >= *remove_at {
                self.warnings_queue.pop_back();
            }
            return resp;
        };
        Status::Ok
    }

    pub fn update(&mut self) {
        while let Ok(alert) = self.receiver.try_recv() {
            match alert.level {
                LogLevel::Info => {
                    let _ = self.term.write_line(&format!(
                        "{} {} {}",
                        style("[INFO]").green(),
                        alert.created_at.to_utc_string(),
                        alert.message
                    ));
                }
                LogLevel::Warning(message, show_for) => {
                    let show_for = show_for.unwrap_or(2);
                    let _ = self.term.write_line(&format!(
                        "{} {} {}",
                        style("[WARNING]").yellow(),
                        alert.created_at.to_utc_string(),
                        alert.message
                    ));
                    let until_ts = if let Some((_, ts)) = self.warnings_queue.front() {
                        Timestamp::from_seconds(ts.seconds() + show_for as u64)
                    } else {
                        Timestamp::from_seconds(alert.created_at.seconds() + show_for as u64)
                    };
                    self.warnings_queue.push_front((message, until_ts));
                }
                LogLevel::Error(message) => {
                    let _ = self.term.write_line(&format!(
                        "{} {} {}",
                        style("[ERROR]").red(),
                        alert.created_at.to_utc_string(),
                        alert.message
                    ));
                    self.status = Status::Critical(message);
                }
            }
        }
    }
}
