mod channel_handler;
mod bot_handler;

use std::env;
use std::error;
use bot_handler::*;
use twitchchat::commands;
use twitchchat::*;

fn main() -> Result<(), Box<dyn error::Error>> {
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

    let mut bot_handler = BotHandler::load_yaml("example.yaml", writer)?;

    bot_handler.save_json("example.json")?;

    for event in &mut client {
        match event {
            Event::TwitchReady(usr) => {
                println!("Joined twitch as {:?}", usr);
                bot_handler.join_channels()?;
            }
            Event::Message(Message::NamesStart(start)) => {
                println!("Users start {:?} on {}", start.users(), start.channel());
            }
            Event::Message(Message::NamesEnd(end)) => {
                println!("user end {}", end.channel());
            }
            Event::Message(Message::PrivMsg(msg)) => {
                println!("priv msg {}: {}", msg.user(), msg.message());
                bot_handler.handle_message(&msg)?;
            }
            Event::Message(Message::Join(msg)) => {
                println!("*** {} joined {}", msg.user(), msg.channel());
                bot_handler.handle_join(&msg)?;
            }
            Event::Message(Message::Part(msg)) => {
                println!("*** {} left {}", msg.user(), msg.channel());
            }
            Event::Message(Message::Irc(message)) => {
                println!("IRC msg {:?}", message);
            }
            Event::Error(err) => {
                eprintln!("error: {}", err);
                break;
            },
            unknown => println!("Unknown event {:?}", unknown),
        }
    }
    client.wait_for_close();
    Ok(())
}
