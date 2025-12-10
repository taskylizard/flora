mod deployments;
mod discord_handler;
mod handlers;
mod ops;
mod runtime;
mod state;
mod transpile;
mod v8_init;

use std::{future::IntoFuture, net::SocketAddr, path::Path, sync::Arc};

use color_eyre::eyre::Result;
use deployments::DeploymentService;
use discord_handler::DiscordHandler;
use eyre::eyre;
use fred::prelude::*;
use handlers::create_router;
use runtime::BotRuntime;
use serenity::all::{Client, GatewayIntents};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tokio::net::TcpListener;
use tracing::error;

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

    let api_state = AppState { runtime: runtime.clone(), deployments: deployment_service.clone() };

    let api_router = create_router(api_state);
    let listener = TcpListener::bind(api_addr).await?;
    let api_service = api_router.into_make_service();
    let api_task = tokio::spawn(axum::serve(listener, api_service).into_future());

    let discord_task = tokio::spawn(async move { client.start().await });

    let (api_res, discord_res) = tokio::try_join!(api_task, discord_task)?;
    api_res.map_err(|err: std::io::Error| eyre!(err))?;
    discord_res.map_err(|err: serenity::Error| eyre!(err))?;

    Ok(())
}
