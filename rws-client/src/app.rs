use anyhow::Result;
use chrono::{DateTime, Local};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Message {
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub is_system: bool,
}

impl Message {
    pub fn new(content: String, is_system: bool) -> Self {
        Self {
            content,
            timestamp: Local::now(),
            is_system,
        }
    }
}

pub struct App {
    pub username: String,
    pub server_url: String,
    pub messages: Vec<Message>,
    pub input: String,
    pub current_room: Option<String>,
    pub should_quit: bool,
    pub tx: Option<mpsc::UnboundedSender<String>>,
}

impl App {
    pub fn new(username: String, server_url: String) -> Result<Self> {
        Ok(Self {
            username,
            server_url,
            messages: Vec::new(),
            input: String::new(),
            current_room: None,
            should_quit: false,
            tx: None,
        })
    }

    pub fn add_message(&mut self, content: String, is_system: bool) {
        self.messages.push(Message::new(content, is_system));
        // Keep only last 100 messages
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }
}
