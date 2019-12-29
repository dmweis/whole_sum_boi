use twitchchat::{Writer, commands::PrivMsg};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::error::*;

pub enum TriggerType {
    Contains(String),
    Equivalent(String),
}

pub enum ResponseType {
    Static(String),
    Repeat,
}

pub struct MessageHandler {
    trigger: TriggerType,
    response: ResponseType,
}

impl MessageHandler {
    pub fn new(trigger: TriggerType, response: ResponseType) -> MessageHandler {
        MessageHandler {
            trigger,
            response,
        }
    }
}

pub struct ChannelHandler {
    name: String,
    writer: Writer,
    user_timeouts: HashMap<String, Instant>,
    user_timeout: Duration,
    handlers: Vec<MessageHandler>,
}

impl ChannelHandler {
    pub fn new(name: &str, writer: Writer) -> ChannelHandler {
        ChannelHandler {
            name: name.to_owned(),
            writer,
            user_timeouts: HashMap::new(),
            user_timeout: Duration::from_secs(10),
            handlers: vec![],
        }
    }

    pub fn join_channel(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(self.writer.join(&self.name)?)
    }

    pub fn handle_message(&mut self, message: &PrivMsg) -> Result<(), Box<dyn Error>> {
        let username = message.user().to_lowercase();
        // check timeout
        if let Some(time) = self.user_timeouts.get(&username) {
            if time.elapsed() <= self.user_timeout {
                println!("user {} timed out {} out of {}", &username, &time.elapsed().as_secs_f32(), self.user_timeout.as_secs_f32());
                return Ok(());
            }
        }
        let message_text = message.message();
        // handle message
        let mut sent_message = false;
        for handler in &self.handlers {
            match &handler.trigger {
                TriggerType::Contains(text) => {
                    if message_text.to_lowercase().contains(&text.to_lowercase()) {
                        match &handler.response {
                            ResponseType::Static(response_text) => {
                                self.writer.send(&self.name, &response_text)?;
                            },
                            ResponseType::Repeat => {
                                self.writer.send(&self.name, &message.message().replace(text, ""))?;
                            },
                        }
                        sent_message = true;
                        break;
                    }
                },
                TriggerType::Equivalent(text) => {
                    if message_text.to_lowercase() == text.to_lowercase() {
                        match &handler.response {
                            ResponseType::Static(response_text) => {
                                self.writer.send(&self.name, &response_text)?;
                            },
                            ResponseType::Repeat => {
                                self.writer.send(&self.name, &message.message().replace(text, ""))?;
                            },
                        }
                        sent_message = true;
                        break;
                    }
                }
            }
        }
        if sent_message {
            self.user_timeouts.insert(username, Instant::now());
        }
        Ok(())
    }

    pub fn add_handler(&mut self, trigger_type: TriggerType, response: ResponseType) {
        self.handlers.push(MessageHandler::new(trigger_type, response));
    }
}