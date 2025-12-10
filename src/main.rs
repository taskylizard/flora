mod deployments;
mod ops;
mod runtime;
mod transpile;
mod v8_init;

use std::{net::SocketAddr, path::Path, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path as AxumPath, State},
    http::StatusCode,
    routing::{get, post},
};
use color_eyre::eyre::Result;
use deployments::{Deployment, DeploymentService, ScriptLanguage};
use eyre::eyre;
use fred::prelude::*;
use runtime::BotRuntime;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::all::{
    ChannelId, Client, Context, EventHandler, GatewayIntents, GuildId, Message, MessageId,
    MessageUpdateEvent, Ready, async_trait,
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Clone)]
struct ApiState {
    runtime: Arc<BotRuntime>,
    deployments: DeploymentService,
}

#[derive(Deserialize)]
struct DeploymentRequest {
    code: String,
    language: Option<String>,
}

#[derive(Serialize)]
struct DeploymentResponse {
    guild_id: String,
    language: String,
    created_at: String,
    updated_at: String,
}

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
            .dispatch_js_event("ready", None, serde_json::to_value(payload).unwrap_or_default())
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

        let guild_id = msg.guild_id.map(|guild| guild.get().to_string());
        if let Err(err) = self.runtime.dispatch_js_event("messageCreate", guild_id, value).await {
            error!("dispatch_js_event error: {:?}", err);
        }
    }

    async fn message_update(
        &self,
        _ctx: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let payload = MessageUpdatePayload::from_parts(old, new, &event);
        let guild_id = payload.guild_id.clone();
        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message update payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageUpdate", guild_id, value).await {
            error!("dispatch_js_event (messageUpdate) error: {:?}", err);
        }
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        let payload = MessageDeletePayload {
            id: deleted_message_id.get().to_string(),
            channel_id: channel_id.get().to_string(),
            guild_id: guild_id.map(|g| g.get().to_string()),
        };
        let guild_id = payload.guild_id.clone();

        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message delete payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageDelete", guild_id, value).await {
            error!("dispatch_js_event (messageDelete) error: {:?}", err);
        }
    }

    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        multiple_deleted_messages_ids: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) {
        let payload = MessageDeleteBulkPayload {
            ids: multiple_deleted_messages_ids.into_iter().map(|id| id.get().to_string()).collect(),
            channel_id: channel_id.get().to_string(),
            guild_id: guild_id.map(|g| g.get().to_string()),
        };
        let guild_id = payload.guild_id.clone();

        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message bulk delete payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageDeleteBulk", guild_id, value).await
        {
            error!("dispatch_js_event (messageDeleteBulk) error: {:?}", err);
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

impl From<&serenity::all::User> for UserPayload {
    fn from(user: &serenity::all::User) -> Self {
        Self {
            id: user.id.get().to_string(),
            username: user.name.clone(),
            discriminator: user.discriminator.map(|d| d.get()),
            bot: user.bot,
        }
    }
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
struct MessageUpdatePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
    content: Option<String>,
    author: Option<UserPayload>,
    edited_timestamp: Option<String>,
    old: Option<MessagePayload>,
    new: Option<MessagePayload>,
}

impl MessageUpdatePayload {
    fn from_parts(old: Option<Message>, new: Option<Message>, event: &MessageUpdateEvent) -> Self {
        let guild_id = event
            .guild_id
            .map(|g| g.get().to_string())
            .or_else(|| new.as_ref().and_then(|m| m.guild_id).map(|g| g.get().to_string()))
            .or_else(|| old.as_ref().and_then(|m| m.guild_id).map(|g| g.get().to_string()));

        let content = event.content.clone().or_else(|| new.as_ref().map(|m| m.content.clone()));

        let author = event
            .author
            .as_ref()
            .map(UserPayload::from)
            .or_else(|| new.as_ref().map(|m| UserPayload::from(&m.author)));

        let edited_timestamp =
            event.edited_timestamp.and_then(|ts| ts.to_rfc3339()).or_else(|| {
                new.as_ref().and_then(|m| m.edited_timestamp.and_then(|ts| ts.to_rfc3339()))
            });

        Self {
            id: event.id.get().to_string(),
            channel_id: event.channel_id.get().to_string(),
            guild_id,
            content,
            author,
            edited_timestamp,
            old: old.as_ref().map(MessagePayload::from),
            new: new.as_ref().map(MessagePayload::from),
        }
    }
}

#[derive(Serialize)]
struct MessageDeletePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
}

#[derive(Serialize)]
struct MessageDeleteBulkPayload {
    ids: Vec<String>,
    channel_id: String,
    guild_id: Option<String>,
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
            guild_ids: ready.guilds.iter().map(|g| g.id.get().to_string()).collect(),
        }
    }
}

fn router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/deployments", get(list_deployments))
        .route("/deployments/{guild_id}", post(create_or_update_deployment).get(read_deployment))
        .with_state(state)
}

async fn create_or_update_deployment(
    AxumPath(guild_id): AxumPath<String>,
    State(state): State<ApiState>,
    Json(request): Json<DeploymentRequest>,
) -> Result<Json<DeploymentResponse>, (StatusCode, String)> {
    let language = ScriptLanguage::from_option(request.language);
    let deployment = state
        .deployments
        .upsert_deployment(guild_id.clone(), request.code, language.clone())
        .await
        .map_err(internal_error)?;

    state.runtime.deploy_guild_script(deployment.clone()).await.map_err(internal_error)?;

    Ok(Json(deployment.into()))
}

async fn read_deployment(
    AxumPath(guild_id): AxumPath<String>,
    State(state): State<ApiState>,
) -> Result<Json<DeploymentResponse>, (StatusCode, String)> {
    let deployment = state.deployments.get_deployment(&guild_id).await.map_err(internal_error)?;

    match deployment {
        Some(deployment) => Ok(Json(deployment.into())),
        None => Err((StatusCode::NOT_FOUND, "deployment not found".to_string())),
    }
}

fn internal_error<T: std::fmt::Display>(err: T) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

impl From<Deployment> for DeploymentResponse {
    fn from(value: Deployment) -> Self {
        Self {
            guild_id: value.guild_id,
            language: value.language.as_str().to_string(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

async fn list_deployments(
    State(state): State<ApiState>,
) -> Result<Json<Vec<DeploymentResponse>>, (StatusCode, String)> {
    let deployments = state.deployments.list_deployments().await.map_err(internal_error)?;

    let response = deployments.into_iter().map(DeploymentResponse::from).collect();

    Ok(Json(response))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .map_err(|_| color_eyre::eyre::eyre!("DISCORD_TOKEN environment variable not set"))?;
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5433/oakmoss".to_string());
    let valkey_url =
        std::env::var("VALKEY_URL").unwrap_or_else(|_| "redis://127.0.0.1:5434/0".to_string());
    let api_addr: SocketAddr = std::env::var("API_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .map_err(|_| eyre!("invalid API_ADDR"))?;

    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let valkey_config = Config::from_url(&valkey_url)?;
    let valkey_client = Builder::from_config(valkey_config).build()?;
    let valkey_task = valkey_client.init().await?;
    let deployment_service =
        DeploymentService::new(pool.clone(), valkey_client.clone(), valkey_task);

    v8_init::init();

    let http = Arc::new(serenity::http::Http::new(&token));
    let runtime = Arc::new(BotRuntime::new(http.clone()));
    runtime.initialize().await.map_err(|err| eyre!(err))?;

    if let Err(err) = runtime.load_user_script("dist/sdk-bundle.js").await {
        error!("Failed to load SDK bundle: {:?}", err);
    }

    // Optionally load a default script for local development when present.
    if Path::new("scripts/bot.ts").exists() {
        if let Err(err) = runtime.load_user_script("scripts/bot.ts").await {
            error!("Failed to load user script: {:?}", err);
        }
    }

    deployment_service.migrate().await?;
    let cached_deployments = deployment_service.list_deployments().await?;
    for deployment in cached_deployments {
        if let Err(err) = runtime.deploy_guild_script(deployment.clone()).await {
            error!("Failed to load deployment for guild {}: {:?}", deployment.guild_id, err);
        }
    }

    let intents = GatewayIntents::all();

    let handler = DiscordHandler { runtime: runtime.clone() };

    let mut client = Client::builder(&token, intents).event_handler(handler).await?;

    let api_state = ApiState { runtime: runtime.clone(), deployments: deployment_service.clone() };

    let api_router = router(api_state);
    let listener = TcpListener::bind(api_addr).await?;
    let api_task =
        tokio::spawn(
            async move { axum::serve(listener, api_router).await.map_err(|err| eyre!(err)) },
        );

    let discord_task = tokio::spawn(async move { client.start().await.map_err(|err| eyre!(err)) });

    let (api_res, discord_res) = tokio::try_join!(api_task, discord_task)?;
    api_res?;
    discord_res?;

    Ok(())
}
