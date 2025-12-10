use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::eyre::{Result, eyre};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "oakmoss", about = "Deployment CLI for oakmoss guild scripts")]
struct Cli {
    /// API base URL (env: OAKMOSS_API_URL)
    #[arg(long, env = "OAKMOSS_API_URL", default_value = "http://localhost:3000")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deploy or update a guild script
    Deploy {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
        /// Path to JS/TS script file
        file: PathBuf,
        /// Script language (defaults to ts)
        #[arg(value_enum, default_value = "typescript")]
        language: Language,
    },
    /// Fetch a guild deployment
    Get {
        /// Discord guild ID
        #[arg(long)]
        guild: String,
    },
    /// List all deployments
    List,
    /// Health check
    Health,
}

#[derive(Clone, Debug, ValueEnum)]
enum Language {
    Javascript,
    Typescript,
}

#[derive(Serialize)]
struct DeploymentRequest<'a> {
    code: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
struct DeploymentResponse {
    guild_id: String,
    language: String,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize, Debug)]
struct HealthResponse(String);

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let client = Client::new();

    match cli.command {
        Commands::Deploy {
            guild,
            file,
            language,
        } => deploy(&client, &cli.api_url, guild, file, language).await?,
        Commands::Get { guild } => get(&client, &cli.api_url, guild).await?,
        Commands::List => list(&client, &cli.api_url).await?,
        Commands::Health => health(&client, &cli.api_url).await?,
    }

    Ok(())
}

async fn deploy(
    client: &Client,
    api_url: &str,
    guild: String,
    file: PathBuf,
    language: Language,
) -> Result<()> {
    let code = fs::read_to_string(&file)
        .map_err(|err| eyre!("failed to read {}: {err}", file.display()))?;
    let lang_str = match language {
        Language::Javascript => "javascript",
        Language::Typescript => "typescript",
    };

    let url = format!("{api_url}/deployments/{guild}");
    let body = DeploymentRequest {
        code: &code,
        language: Some(lang_str),
    };

    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<DeploymentResponse>()
        .await?;

    println!(
        "Deployed guild {} ({}) at {}",
        resp.guild_id, resp.language, resp.updated_at
    );
    Ok(())
}

async fn get(client: &Client, api_url: &str, guild: String) -> Result<()> {
    let url = format!("{api_url}/deployments/{guild}");
    let resp = client.get(url).send().await?.error_for_status()?;
    let deployment = resp.json::<DeploymentResponse>().await?;
    println!(
        "Guild {} ({})\n  created: {}\n  updated: {}",
        deployment.guild_id, deployment.language, deployment.created_at, deployment.updated_at
    );
    Ok(())
}

async fn list(client: &Client, api_url: &str) -> Result<()> {
    let url = format!("{api_url}/deployments");
    let deployments = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<DeploymentResponse>>()
        .await?;

    if deployments.is_empty() {
        println!("No deployments found");
    } else {
        for d in deployments {
            println!(
                "{} ({}) created={} updated={}",
                d.guild_id, d.language, d.created_at, d.updated_at
            );
        }
    }
    Ok(())
}

async fn health(client: &Client, api_url: &str) -> Result<()> {
    let url = format!("{api_url}/health");
    let resp = client.get(url).send().await?.error_for_status()?;
    let body = resp.text().await?;
    println!("{body}");
    Ok(())
}
