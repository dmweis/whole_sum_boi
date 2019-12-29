mod channel_handler;

use std::env;
use std::error;
use channel_handler::{*, TriggerType::*, ResponseType::*};
use twitchchat::commands;
use twitchchat::*;

fn main() -> Result<(), Box<dyn error::Error>> {
    let username = env::var("TWITCH_USER_NAME").expect("bot needs twitch username");
    let oauth_key = env::var("TWITCH_OAUTH_KEY").expect("bot needs twitch username");
    
    let mut client = twitchchat::connect_easy(username, oauth_key)
    .unwrap()
    .filter::<commands::PrivMsg>()
    .filter::<commands::Join>();
    
    let writer = client.writer();

    let mut example_handler = ChannelHandler::new("client_name", writer);

    example_handler.add_handler(Contains("his name".to_owned()), Static("His name is Jeffbob Blobby Ewing".to_owned()));
    example_handler.add_handler(Contains("sombreros".to_owned()), Static("Did you mean hats?".to_owned()));
    example_handler.add_handler(Contains("kapelusz".to_owned()), Static("Did you mean hats?".to_owned()));
    example_handler.add_handler(Contains("hats".to_owned()), Static("@client_name could you please adjust the hats and googly eyes?".to_owned()));
    example_handler.add_handler(Contains("blurp".to_owned()), Static("Oh no. Look out she is gonna kill him again!!!".to_owned()));
    example_handler.add_handler(Contains("bot say:".to_owned()), Repeat);

    for event in &mut client {
        match event {
            Event::TwitchReady(usr) => {
                println!("Joined twitch as {:?}", usr);
                example_handler.join_channel()?;
            }
            Event::Message(Message::PrivMsg(msg)) => {
                println!("priv msg {}: {}", msg.user(), msg.message());
                example_handler.handle_message(&msg)?;
            }
            Event::Message(Message::Join(msg)) => {
                println!("*** {} joined {}", msg.user(), msg.channel());
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
