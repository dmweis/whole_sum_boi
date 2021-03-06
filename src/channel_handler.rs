use twitchchat::{Writer, commands::PrivMsg, commands::Join};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::error::*;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::prelude::*;
use regex::Regex;
use reqwest;
use log::*;

/// Data structure for representing when an
/// action should match message
#[derive(Serialize, Deserialize, Clone)]
pub enum TriggerType {
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    Equivalent(String),
    RegexMatch(String),
}

impl TriggerType {
    fn check_match(&self, message: &str) -> Result<bool, Box<dyn Error>> {
        Ok(match self {
            TriggerType::Contains(text) => message.to_lowercase().contains(&text.to_lowercase()),
            TriggerType::StartsWith(text) => message.to_lowercase().starts_with(&text.to_lowercase()),
            TriggerType::EndsWith(text) => message.to_lowercase().ends_with(&text.to_lowercase()),
            TriggerType::Equivalent(text) => message.to_lowercase() == text.to_lowercase(),
            TriggerType::RegexMatch(pattern) => {
                let matcher = Regex::new(&pattern)?;
                matcher.is_match(&message.to_lowercase())
            }
        })
    }
}

/// Data structure for representing type
/// of a response an action generates
#[derive(Serialize, Deserialize, Clone)]
pub enum ResponseType {
    Static(String),
    Repeat,
    DadJoke
}

#[derive(Serialize, Deserialize, Clone)]
struct DadJoke {
    id: String,
    joke: String,
    status: i32,
}

fn get_dad_joke() -> Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let res: DadJoke = client.get("https://icanhazdadjoke.com/")
        .header("Accept", "application/json")
        .send()?
        .json()?;
    Ok(res.joke)
}

/// Data structure defining how to match
/// to a messages and what response to generate
#[derive(Serialize, Deserialize, Clone)]
pub struct Action {
    trigger: TriggerType,
    response: ResponseType,
}

impl Action {
    #[allow(dead_code)]
    pub fn new(trigger: TriggerType, response: ResponseType) -> Action {
        Action {
            trigger,
            response,
        }
    }
}

/// Data structure used for serializing channel handlers
#[derive(Serialize, Deserialize)]
pub struct ChannelHandlerConfig {
    name: String,
    user_timeout: Duration,
    handlers: Vec<Action>,
}

impl ChannelHandlerConfig {
    #[allow(dead_code)]
    pub fn from_channel_handler(handler: &ChannelHandler) -> ChannelHandlerConfig {
        ChannelHandlerConfig {
            name: handler.name.clone(),
            user_timeout: handler.user_timeout.clone(),
            handlers: handler.handlers.clone(),
        }
    }
}

/// Massed handler for channel
/// One should be created for each channel and
/// messages for that channel should be routed into it
pub struct ChannelHandler {
    name: String,
    writer: Writer,
    user_timeouts: HashMap<String, Instant>,
    user_timeout: Duration,
    handlers: Vec<Action>,
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

    pub fn from_config(config: ChannelHandlerConfig, writer: Writer) -> ChannelHandler {
        ChannelHandler {
            name: config.name,
            writer,
            user_timeouts: HashMap::new(),
            user_timeout: config.user_timeout,
            handlers: config.handlers,
        }
    }

    pub fn channel_name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub fn load_yaml(path: &str, writer: Writer) -> Result<ChannelHandler, Box<dyn Error>> {
        let file = fs::read_to_string(path)?;
        let config: ChannelHandlerConfig = serde_yaml::from_str(&file)?;
        Ok(ChannelHandler::from_config(config, writer))
    }

    #[allow(dead_code)]
    pub fn load_json(path: &str, writer: Writer) -> Result<ChannelHandler, Box<dyn Error>> {
        let file = fs::read_to_string(path)?;
        let config: ChannelHandlerConfig = serde_json::from_str(&file)?;
        Ok(ChannelHandler::from_config(config, writer))
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
                info!("user {} timed out {} out of {}", &username, &time.elapsed().as_secs_f32(), self.user_timeout.as_secs_f32());
                return Ok(());
            }
        }
        let message_text = message.message();
        // handle message
        let mut sent_message = false;
        for handler in &self.handlers {
            if handler.trigger.check_match(&message_text)? {
                match &handler.response {
                    ResponseType::Static(response_text) => {
                        info!("Sent message {} into {}", &response_text, &self.name);
                        self.writer.send(&self.name, &response_text)?;
                    },
                    ResponseType::Repeat => {
                        let message_to_send = message_text
                                            .split(":")
                                            .into_iter()
                                            .skip(1)
                                            .collect::<Vec<&str>>()
                                            .join("");
                        info!("Sent message {} into {}", &message_to_send, &self.name);
                        self.writer.send(&self.name, message_to_send)?;
                    },
                    ResponseType::DadJoke => {
                        let dad_joke = get_dad_joke()?;
                        info!("Sent message {} into {}", &dad_joke, &self.name);
                        self.writer.send(&self.name, &dad_joke)?;
                    }
                }
                sent_message = true;
                break;
            }
        }
        if sent_message {
            self.user_timeouts.insert(username, Instant::now());
        }
        Ok(())
    }

    pub fn handle_join(&mut self, _: &Join) -> Result<(), Box<dyn Error>> {
        // handle channel connections here
        Ok(())
    }

    #[allow(dead_code)]
    pub fn add_handler(&mut self, trigger_type: TriggerType, response: ResponseType) {
        self.handlers.push(Action::new(trigger_type, response));
    }
}
