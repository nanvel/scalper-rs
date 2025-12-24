use super::sound::Sound;
use super::timestamp::Timestamp;
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
    pub sound: Option<Sound>,
    pub created_at: Timestamp,
}

impl Log {
    pub fn new(level: LogLevel, message: String, sound: Option<Sound>) -> Self {
        Log {
            level,
            message,
            sound,
            created_at: Timestamp::now(),
        }
    }
}

pub struct LogManager {
    receiver: Receiver<Log>,
    term: Term,
    warnings_queue: VecDeque<(String, Timestamp)>,
    status: Status,
    with_sound: bool,
}

impl LogManager {
    pub fn new(receiver: Receiver<Log>, term: Term, with_sound: bool) -> Self {
        LogManager {
            receiver,
            term,
            warnings_queue: VecDeque::new(),
            status: Status::Ok,
            with_sound,
        }
    }

    pub fn set_with_sound(&mut self, with_sound: bool) {
        self.with_sound = with_sound;
    }

    pub fn log_info(&self, message: &str) {
        let _ = self.term.write_line(&format!(
            "{} {} {}",
            style("[INFO]").green(),
            Timestamp::now().to_utc_string(),
            message
        ));
    }

    pub fn log_warning(&self, message: &str) {
        let _ = self.term.write_line(&format!(
            "{} {} {}",
            style("[WARNING]").yellow(),
            Timestamp::now().to_utc_string(),
            message
        ));
    }

    pub fn log_error(&self, message: &str) {
        let _ = self.term.write_line(&format!(
            "{} {} {}",
            style("[ERROR]").red(),
            Timestamp::now().to_utc_string(),
            message
        ));
    }

    pub fn status(&mut self) -> Status {
        if let Status::Critical(_message) = &self.status {
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

    pub fn consume(&mut self) {
        while let Ok(alert) = self.receiver.try_recv() {
            match alert.level {
                LogLevel::Info => {
                    self.log_info(&alert.message);
                }
                LogLevel::Warning(message, show_for) => {
                    let show_for = show_for.unwrap_or(2);
                    self.log_warning(&alert.message);
                    let until_ts = if let Some((_, ts)) = self.warnings_queue.front() {
                        Timestamp::from_seconds(ts.seconds() + show_for as u64)
                    } else {
                        Timestamp::from_seconds(alert.created_at.seconds() + show_for as u64)
                    };
                    self.warnings_queue.push_front((message, until_ts));
                }
                LogLevel::Error(message) => {
                    self.log_error(&alert.message);
                    self.status = Status::Critical(message);
                }
            }
            if self.with_sound {
                if let Some(sound) = alert.sound {
                    let sound_clone = sound.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = sound_clone.play() {
                            println!("Sound error: {:?}", e);
                        }
                    });
                }
            }
        }
    }
}
