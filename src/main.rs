use dotenv::dotenv;
use std::env;

use serenity::builder::{ CreateMessage, CreateEmbed };
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!quote" {
            let quote = fetch_quote()
                .await
                .unwrap_or("Sorry couldn't fecth a quote".to_string());

            if let Err(why) = msg.channel_id.say(&ctx.http, quote).await {
                println!("Error sending message: {:?}", why)
            }
        }

        // Respond with an inspirational image
        if msg.content == "!image" {
            let image_url = fetch_image_url()
                .await
                .unwrap_or("Sorry, couldn't fetch an image.".to_string());

            let embed = CreateEmbed::new().image(&image_url);
            let builder = CreateMessage::new().embed(embed);


            if let Err(why) = msg
                .channel_id
                .send_message(&ctx.http, builder)
                .await
            {
                println!("Error sending image: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn fetch_quote() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://zenquotes.io/api/random") // Example API for random quotes
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Access the first element of the array and then the "q" and "a" fields
    let content = res[0]["q"].as_str().unwrap_or("No quote available");
    let author = res[0]["a"].as_str().unwrap_or("Unknown");

    Ok(format!("\"{}\" - {}", content, author))
}

// Function to fetch the random inspirational image URL
async fn fetch_image_url() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get("https://zenquotes.io/api/image").send().await?;

    // Get the final URL from the response
    Ok(res.url().to_string())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
