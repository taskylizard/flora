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
    /// API token (env: OAKMOSS_TOKEN). If unset, CLI will read from config after `login`.
    #[arg(long, env = "OAKMOSS_TOKEN")]
    token: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Store a user token locally for subsequent commands.
    Login {
        /// Token string obtained from POST /tokens
        #[arg(long)]
        token: String,
    },
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
    let token_store = TokenStore::new()?;

    if let Some(token) = cli.token.as_deref() {
        token_store.save(token)?;
    }

    match cli.command {
        Commands::Login { token } => {
            token_store.save(&token)?;
            println!("Saved token to {}", token_store.path().display());
        }
        Commands::Deploy { guild, file, language } => {
            let token = token_store.get(cli.token.as_deref())?;
            deploy(&client, &cli.api_url, &token, guild, file, language).await?
        }
        Commands::Get { guild } => {
            let token = token_store.get(cli.token.as_deref())?;
            get(&client, &cli.api_url, &token, guild).await?
        }
        Commands::List => {
            let token = token_store.get(cli.token.as_deref())?;
            list(&client, &cli.api_url, &token).await?
        }
        Commands::Health => health(&client, &cli.api_url).await?,
    }

    Ok(())
}

async fn deploy(
    client: &Client,
    api_url: &str,
    token: &str,
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
    let body = DeploymentRequest { code: &code, language: Some(lang_str) };

    let resp = client
        .post(url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<DeploymentResponse>()
        .await?;

    println!("Deployed guild {} ({}) at {}", resp.guild_id, resp.language, resp.updated_at);
    Ok(())
}

async fn get(client: &Client, api_url: &str, token: &str, guild: String) -> Result<()> {
    let url = format!("{api_url}/deployments/{guild}");
    let resp = client.get(url).bearer_auth(token).send().await?.error_for_status()?;
    let deployment = resp.json::<DeploymentResponse>().await?;
    println!(
        "Guild {} ({})\n  created: {}\n  updated: {}",
        deployment.guild_id, deployment.language, deployment.created_at, deployment.updated_at
    );
    Ok(())
}

async fn list(client: &Client, api_url: &str, token: &str) -> Result<()> {
    let url = format!("{api_url}/deployments");
    let deployments = client
        .get(url)
        .bearer_auth(token)
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

/// Simple file-based token cache under ~/.config/oakmoss/token
struct TokenStore {
    path: PathBuf,
}

impl TokenStore {
    fn new() -> Result<Self> {
        let base =
            dirs::config_dir().ok_or_else(|| eyre!("cannot resolve config dir"))?.join("oakmoss");
        fs::create_dir_all(&base)?;
        Ok(Self { path: base.join("token") })
    }

    fn save(&self, token: &str) -> Result<()> {
        fs::write(&self.path, token)?;
        Ok(())
    }

    fn get(&self, override_token: Option<&str>) -> Result<String> {
        if let Some(tok) = override_token {
            return Ok(tok.to_string());
        }
        if self.path.exists() {
            let raw = fs::read_to_string(&self.path)?;
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(eyre!("stored token file is empty; run `oakmoss login --token ...`"));
            }
            Ok(trimmed.to_string())
        } else {
            Err(eyre!(
                "no token found. Provide --token / env OAKMOSS_TOKEN or run `oakmoss login --token <token>`"
            ))
        }
    }

    #[allow(dead_code)]
    fn path(&self) -> &PathBuf {
        &self.path
    }
}
