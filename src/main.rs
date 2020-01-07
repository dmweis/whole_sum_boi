mod channel_handler;
mod bot_handler;

use std::{env, error};
use std::fs::OpenOptions;
use bot_handler::*;
use twitchchat::commands;
use twitchchat::*;
use log::*;
use simplelog::*;
use clap::{Arg, App};

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = App::new("WholeSumBoi")
                          .version("0.0.1")
                          .author("David W. <dweis7@gmail.com>")
                          .about("Wholesome twitch bot")
                          .arg(Arg::with_name("config_path")
                               .short("p")
                               .long("path")
                               .value_name("CONFIG")
                               .help("Sets a custom config file")
                               .takes_value(true))
                          .get_matches();
    let config_path = matches.value_of("config_path").unwrap_or("example.yaml");
    let log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("WholeSumBoi.log");
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
            WriteLogger::new(LevelFilter::Info, Config::default(), log_file.unwrap()),
        ]
    ).unwrap();

    let username = env::var("TWITCH_USER_NAME").expect("bot needs twitch username");
    let oauth_key = env::var("TWITCH_OAUTH_KEY").expect("bot needs twitch oauth key");
    
    let mut client = twitchchat::connect_easy(username, oauth_key)
    .unwrap()
    .filter::<commands::PrivMsg>()
    .filter::<commands::NamesStart>()
    .filter::<commands::NamesEnd>()
    .filter::<commands::Join>()
    .filter::<commands::Part>();
    
    let writer = client.writer();

    let mut bot_handler = BotHandler::load_yaml(config_path, writer)?;

    for event in &mut client {
        match event {
            Event::TwitchReady(usr) => {
                info!("Joined twitch as {:?}", usr);
                bot_handler.join_channels()?;
            }
            Event::Message(Message::NamesStart(start)) => {
                info!("Users start {:?} on {}", start.users(), start.channel());
            }
            Event::Message(Message::NamesEnd(end)) => {
                info!("user end {}", end.channel());
            }
            Event::Message(Message::PrivMsg(msg)) => {
                info!("priv msg {}: {}", msg.user(), msg.message());
                bot_handler.handle_message(&msg)?;
            }
            Event::Message(Message::Join(msg)) => {
                info!("*** {} joined {}", msg.user(), msg.channel());
                bot_handler.handle_join(&msg)?;
            }
            Event::Message(Message::Part(msg)) => {
                info!("*** {} left {}", msg.user(), msg.channel());
            }
            Event::Message(Message::Irc(message)) => {
                info!("IRC msg {:?}", message);
            }
            Event::Error(err) => {
                error!("error: {}", err);
                break;
            },
            unknown => info!("Unknown event {:?}", unknown),
        }
    }
    client.wait_for_close();
    Ok(())
}
