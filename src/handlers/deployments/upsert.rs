use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    deployments::{Deployment, ScriptLanguage},
    handlers::auth::{ensure_guild_admin, require_identity},
    handlers::auth::{ensure_guild_admin, require_session},
    handlers::{error::ApiError, response::ApiJson},
    state::AppState,
};

/// Body for creating or replacing a deployment.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentRequest {
    /// Raw source code for the guild bot.
    pub code: String,
    /// Optional scripting language hint. Defaults to `typescript`.
    pub language: Option<String>,
}

/// API representation of a deployment.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentResponse {
    pub guild_id: String,
    pub language: String,
    pub created_at: String,
    pub updated_at: String,
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

/// Create or update a deployment for a guild.
#[utoipa::path(
    post,
    path = "/deployments/{guild_id}",
    request_body = DeploymentRequest,
    params(
        ("guild_id" = String, Path, description = "Discord guild id")
    ),
    tag = "deployment",
    responses(
        (status = 200, description = "Deployment stored", body = DeploymentResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn upsert_deployment_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DeploymentRequest>,
) -> Result<ApiJson<DeploymentResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;
    let session = require_session(&state.auth, &headers).await?;
    ensure_guild_admin(&state.auth, &session, &guild_id).await?;

    let language = ScriptLanguage::from_option(request.language);
    let deployment = state
        .deployments
        .upsert_deployment(guild_id.clone(), request.code, language.clone())
        .await
        .map_err(|err| {
            error!(target: "oakmoss:api", guild_id, ?err, "failed to upsert deployment");
            ApiError::internal(err)
        })?;

    state.runtime.deploy_guild_script(deployment.clone()).await.map_err(|err| {
        error!(target: "oakmoss:api", guild_id, ?err, "failed to deploy guild script");
        ApiError::internal(err)
    })?;

    Ok(ApiJson(Json(deployment.into())))
}
