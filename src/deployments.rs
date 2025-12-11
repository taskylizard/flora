use std::sync::Arc;

use chrono::{DateTime, Utc};
use color_eyre::eyre::{Result, eyre};
use fred::{prelude::*, types::ConnectHandle};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use tracing::{info, warn};
use utoipa::ToSchema;

/// Stored representation of a guild deployment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Deployment {
    pub guild_id: String,
    pub language: ScriptLanguage,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supported scripting languages for deployments.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ScriptLanguage {
    Javascript,
    Typescript,
}

impl ScriptLanguage {
    pub fn from_option(value: Option<String>) -> Self {
        match value.unwrap_or_else(|| "typescript".to_string()).to_ascii_lowercase().as_str() {
            "js" | "javascript" => ScriptLanguage::Javascript,
            _ => ScriptLanguage::Typescript,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ScriptLanguage::Javascript => "javascript",
            ScriptLanguage::Typescript => "typescript",
        }
    }

    pub fn module_name(&self, guild_id: &str) -> String {
        match self {
            ScriptLanguage::Javascript => format!("guild:{guild_id}.js"),
            ScriptLanguage::Typescript => format!("guild:{guild_id}.ts"),
        }
    }
}

#[derive(Clone)]
pub struct DeploymentService {
    db: Pool<Postgres>,
    cache: Client,
    _cache_task: Arc<ConnectHandle>,
}

#[derive(FromRow)]
struct DeploymentRow {
    guild_id: String,
    language: String,
    script: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl DeploymentService {
    pub fn new(db: Pool<Postgres>, cache: Client, cache_task: ConnectHandle) -> Self {
        Self { db, cache, _cache_task: Arc::new(cache_task) }
    }

    pub async fn upsert_deployment(
        &self,
        guild_id: String,
        source: String,
        language: ScriptLanguage,
    ) -> Result<Deployment> {
        let record = sqlx::query_as::<_, DeploymentRow>(
            r#"
            INSERT INTO deployments (guild_id, language, script)
            VALUES ($1, $2, $3)
            ON CONFLICT (guild_id) DO UPDATE
            SET language = EXCLUDED.language,
                script = EXCLUDED.script,
                updated_at = NOW()
            RETURNING guild_id, language, script, created_at, updated_at
            "#,
        )
        .bind(&guild_id)
        .bind(language.as_str())
        .bind(&source)
        .fetch_one(&self.db)
        .await?;

        let deployment = to_deployment(record)?;
        self.cache_deployment(&deployment).await?;
        info!(target: "oakmoss:deployments", guild_id = deployment.guild_id, "deployment stored");
        Ok(deployment)
    }

    pub async fn get_deployment(&self, guild_id: &str) -> Result<Option<Deployment>> {
        if let Some(cached) = self.fetch_cached_deployment(guild_id).await? {
            return Ok(Some(cached));
        }

        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT guild_id, language, script, created_at, updated_at
            FROM deployments
            WHERE guild_id = $1
            "#,
        )
        .bind(guild_id)
        .fetch_optional(&self.db)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let deployment = to_deployment(row)?;
        self.cache_deployment(&deployment).await?;
        Ok(Some(deployment))
    }

    pub async fn list_deployments(&self) -> Result<Vec<Deployment>> {
        let rows = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT guild_id, language, script, created_at, updated_at
            FROM deployments
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        rows.into_iter().map(to_deployment).collect::<Result<Vec<_>>>()
    }

    async fn cache_deployment(&self, deployment: &Deployment) -> Result<()> {
        let key = cache_key(&deployment.guild_id);
        let value = serde_json::to_string(deployment)?;
        // cache for 10 minutes to reduce DB traffic.
        self.cache.set::<(), _, _>(key, value, Some(Expiration::EX(600)), None, false).await?;
        Ok(())
    }

    async fn fetch_cached_deployment(&self, guild_id: &str) -> Result<Option<Deployment>> {
        let key = cache_key(guild_id);
        let value: Option<String> = self.cache.get(key).await?;
        if let Some(raw) = value {
            match serde_json::from_str::<Deployment>(&raw) {
                Ok(deployment) => Ok(Some(deployment)),
                Err(err) => {
                    warn!(target: "oakmoss:deployments", guild_id, ?err, "failed to decode cached deployment");
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

fn cache_key(guild_id: &str) -> String {
    format!("deployment:{guild_id}")
}

fn to_deployment(row: DeploymentRow) -> Result<Deployment> {
    let language = match row.language.as_str() {
        "javascript" | "js" => ScriptLanguage::Javascript,
        "typescript" | "ts" => ScriptLanguage::Typescript,
        other => return Err(eyre!("unsupported language {other}")),
    };

    Ok(Deployment {
        guild_id: row.guild_id,
        language,
        source: row.script,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
