use twitchchat::{Writer, commands::PrivMsg};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::error::*;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub enum TriggerType {
    Contains(String),
    Equivalent(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ResponseType {
    Static(String),
    Repeat,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageHandler {
    trigger: TriggerType,
    response: ResponseType,
}

impl MessageHandler {
    #[allow(dead_code)]
    pub fn new(trigger: TriggerType, response: ResponseType) -> MessageHandler {
        MessageHandler {
            trigger,
            response,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ChannelHandlerConfig {
    name: String,
    user_timeout: Duration,
    handlers: Vec<MessageHandler>,
}

impl ChannelHandlerConfig {
    #[allow(dead_code)]
    fn from_channel_handler(handler: &ChannelHandler) -> ChannelHandlerConfig {
        ChannelHandlerConfig {
            name: handler.name.clone(),
            user_timeout: handler.user_timeout.clone(),
            handlers: handler.handlers.clone(),
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
    #[allow(dead_code)]
    pub fn new(name: &str, writer: Writer) -> ChannelHandler {
        ChannelHandler {
            name: name.to_owned(),
            writer,
            user_timeouts: HashMap::new(),
            user_timeout: Duration::from_secs(10),
            handlers: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn load_yaml(path: &str, writer: Writer) -> Result<ChannelHandler, Box<dyn Error>> {
        let file = fs::read_to_string(path)?;
        let config: ChannelHandlerConfig = serde_yaml::from_str(&file)?;
        Ok(ChannelHandler {
            name: config.name,
            writer,
            user_timeouts: HashMap::new(),
            user_timeout: config.user_timeout,
            handlers: config.handlers,
        })
    }

    #[allow(dead_code)]
    pub fn load_json(path: &str, writer: Writer) -> Result<ChannelHandler, Box<dyn Error>> {
        let file = fs::read_to_string(path)?;
        let config: ChannelHandlerConfig = serde_json::from_str(&file)?;
        Ok(ChannelHandler {
            name: config.name,
            writer,
            user_timeouts: HashMap::new(),
            user_timeout: config.user_timeout,
            handlers: config.handlers,
        })
    }

    #[allow(dead_code)]
    pub fn save_yaml(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let config = ChannelHandlerConfig::from_channel_handler(self);
        let yaml = serde_yaml::to_string(&config)?;
        let mut file = File::create(path)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_json(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let config = ChannelHandlerConfig::from_channel_handler(self);
        let json = serde_json::to_string_pretty(&config)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
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

    #[allow(dead_code)]
    pub fn add_handler(&mut self, trigger_type: TriggerType, response: ResponseType) {
        self.handlers.push(MessageHandler::new(trigger_type, response));
    }
}