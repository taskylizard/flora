mod ops;
mod runtime;
mod transpile;
mod v8_init;

use std::sync::Arc;

use color_eyre::eyre::Result;
use eyre::eyre;
use runtime::BotRuntime;
use serde::Serialize;
use serde_json;
use serenity::all::{Client, Context, EventHandler, GatewayIntents, Message, Ready, async_trait};
use tracing::{error, info};

#[derive(Clone)]
struct DiscordHandler {
    runtime: Arc<BotRuntime>,
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        let payload = ReadyPayload::from(&ready);
        if let Err(err) = self
            .runtime
            .dispatch_js_event("ready", serde_json::to_value(payload).unwrap_or_default())
            .await
        {
            error!("dispatch_js_event (ready) error: {:?}", err);
        }
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        info!(
            target: "oakmoss:discord",
            "message event channel={} author={} content={}",
            msg.channel_id,
            msg.author.id,
            msg.content
        );
        let payload = MessagePayload::from(&msg);
        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageCreate", value).await {
            error!("dispatch_js_event error: {:?}", err);
        }
    }
}

#[derive(Serialize)]
struct UserPayload {
    id: String,
    username: String,
    discriminator: Option<u16>,
    bot: bool,
}

#[derive(Serialize)]
struct MessagePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
    content: String,
    author: UserPayload,
}

impl From<&Message> for MessagePayload {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.id.get().to_string(),
            channel_id: msg.channel_id.get().to_string(),
            guild_id: msg.guild_id.map(|g| g.get().to_string()),
            content: msg.content.clone(),
            author: UserPayload {
                id: msg.author.id.get().to_string(),
                username: msg.author.name.clone(),
                discriminator: msg.author.discriminator.map(|d| d.get()),
                bot: msg.author.bot,
            },
        }
    }
}

#[derive(Serialize)]
struct ReadyPayload {
    user: UserPayload,
    guild_ids: Vec<String>,
}

impl From<&Ready> for ReadyPayload {
    fn from(ready: &Ready) -> Self {
        Self {
            user: UserPayload {
                id: ready.user.id.get().to_string(),
                username: ready.user.name.clone(),
                discriminator: ready.user.discriminator.map(|d| d.get()),
                bot: ready.user.bot,
            },
            guild_ids: ready
                .guilds
                .iter()
                .map(|g| g.id.get().to_string())
                .collect(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| color_eyre::eyre::eyre!("DISCORD_TOKEN environment variable not set"))?;

    v8_init::init();

    let http = Arc::new(serenity::http::Http::new(&token));
    let runtime = Arc::new(BotRuntime::new(http.clone()));
    runtime.initialize().await.map_err(|err| eyre!(err))?;

    if let Err(err) = runtime.load_user_script("scripts/sdk-bundle.js").await {
        error!("Failed to load SDK bundle: {:?}", err);
    }

    // Load a default script for local development.
    if let Err(err) = runtime.load_user_script("scripts/bot.ts").await {
        error!("Failed to load user script: {:?}", err);
    }

    let intents = GatewayIntents::all();

    let handler = DiscordHandler {
        runtime: runtime.clone(),
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await?;

    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
    }

    Ok(())
}
