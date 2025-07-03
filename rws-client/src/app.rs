use anyhow::Result;
use chrono::{DateTime, Local};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub enum UiEvent {
    AddMessage { content: String, is_system: bool },
    AddMessageWithId { id: Uuid, content: String, is_system: bool },
    UpdateMessage { id: Uuid, content: String },
}

#[derive(Debug)]
pub struct Message {
    pub id: Option<Uuid>,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub is_system: bool,
}

impl Message {
    pub fn new(content: String, is_system: bool) -> Self {
        Self {
            id: None,
            content,
            timestamp: Local::now(),
            is_system,
        }
    }

    pub fn new_with_id(id: Uuid, content: String, is_system: bool) -> Self {
        Self {
            id: Some(id),
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

    pub fn add_message_with_id(&mut self, id: Uuid, content: String, is_system: bool) {
        self.messages.push(Message::new_with_id(id, content, is_system));
        // Keep only last 100 messages
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn update_message(&mut self, id: Uuid, new_content: String) -> bool {
        for message in &mut self.messages {
            if message.id == Some(id) {
                message.content = new_content;
                return true;
            }
        }
        false
    }

    pub fn handle_ui_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::AddMessage { content, is_system } => {
                self.add_message(content, is_system);
            }
            UiEvent::AddMessageWithId { id, content, is_system } => {
                self.add_message_with_id(id, content, is_system);
            }
            UiEvent::UpdateMessage { id, content } => {
                self.update_message(id, content);
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }
}
