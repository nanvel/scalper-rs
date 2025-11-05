use crate::models::Timestamp;
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub enum MessageType {
    Info,
    Error,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub message: String,
    pub created_at: Timestamp,
    pub duration: u32,
}

impl Message {
    pub fn new(message_type: MessageType, message: String, duration: Option<u32>) -> Self {
        Message {
            message_type: MessageType::Info,
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

pub struct MessageManager {
    active_messages: VecDeque<Message>,
    receiver: Receiver<Message>,
}

impl MessageManager {
    pub fn new(receiver: Receiver<Message>) -> Self {
        MessageManager {
            active_messages: VecDeque::new(),
            receiver,
        }
    }

    pub fn update(&mut self) {
        while let Ok(alert) = self.receiver.try_recv() {
            self.active_messages.push_back(alert);
        }

        self.active_messages.retain(|alert| !alert.is_expired());
    }

    pub fn get_active_alerts(&self) -> &VecDeque<Message> {
        &self.active_messages
    }
}
