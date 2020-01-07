use std::collections::HashMap;
use crate::channel_handler::*;
use twitchchat::{Writer, commands::PrivMsg, commands::Join};
use std::error::*;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct BotHandlerConfig {
    bots: Vec<ChannelHandlerConfig>,
}

impl BotHandlerConfig {
    fn from_bot_handler(handler: &BotHandler) -> BotHandlerConfig {
        let channel_handlers = handler
                    .bots
                    .iter()
                    .map(|(_, channel)| ChannelHandlerConfig::from_channel_handler(channel))
                    .collect();
        BotHandlerConfig {
            bots: channel_handlers,
        }
    }
}

pub struct BotHandler {
    bots: HashMap<String, ChannelHandler>,
}

impl BotHandler {
    #[allow(dead_code)]
    pub fn new() -> BotHandler {
        BotHandler {
            bots: HashMap::new(),
        }
    }

    pub fn with_handlers(handlers: Vec<ChannelHandler>) -> BotHandler {
        BotHandler {
            bots: handlers
                .into_iter()
                .map(|handler| (handler.channel_name().to_string(), handler))
                .collect(),
        }
    }

    #[allow(dead_code)]
    pub fn get_bot_mut(&mut self, key: &str) -> Option<&mut ChannelHandler> {
        self.bots.get_mut(key)
    }

    pub fn join_channels(&mut self) -> Result<(), Box<dyn Error>> {
        for (_, handler) in &mut self.bots {
            handler.join_channel()?;
        }
        Ok(())
    }

    pub fn handle_message(&mut self, message: &PrivMsg) -> Result<(), Box<dyn Error>> {
        let channel = message.channel().replace("#", "");
        if let Some(bot) = self.bots.get_mut(channel.as_str()) {
            bot.handle_message(message)?;
        }
        Ok(())
    }

    pub fn handle_join(&mut self, message: &Join) -> Result<(), Box<dyn Error>> {
        let channel = message.channel().replace("#", "");
        if let Some(bot) = self.bots.get_mut(channel.as_str()) {
            bot.handle_join(message)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_yaml(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let config = BotHandlerConfig::from_bot_handler(self);
        let yaml = serde_yaml::to_string(&config)?;
        let mut file = File::create(path)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_json(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let config = BotHandlerConfig::from_bot_handler(self);
        let json = serde_json::to_string_pretty(&config)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn load_yaml(path: &str, writer: Writer) -> Result<BotHandler, Box<dyn Error>> {
        let file = fs::read_to_string(path)?;
        let config: BotHandlerConfig = serde_yaml::from_str(&file)?;
        let bots: Vec<ChannelHandler> = config
            .bots
            .into_iter()
            .map(|channel| ChannelHandler::from_config(channel, writer.clone()))
            .collect();
        Ok(BotHandler::with_handlers(bots))
    }

    #[allow(dead_code)]
    pub fn load_json(path: &str, writer: Writer) -> Result<BotHandler, Box<dyn Error>> {
        let file = fs::read_to_string(path)?;
        let config: BotHandlerConfig = serde_json::from_str(&file)?;
        let bots: Vec<ChannelHandler> = config
            .bots
            .into_iter()
            .map(|channel| ChannelHandler::from_config(channel, writer.clone()))
            .collect();
        Ok(BotHandler::with_handlers(bots))
    }
}