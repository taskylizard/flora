use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use tracing::error;

use crate::{
    handlers::auth::{ensure_guild_admin, require_identity},
    handlers::auth::{ensure_guild_admin, require_session},
    handlers::{error::ApiError, response::ApiJson},
    state::AppState,
};

use super::DeploymentResponse;

/// Fetch a single deployment by guild id.
#[utoipa::path(
    get,
    path = "/deployments/{guild_id}",
    params(
        ("guild_id" = String, Path, description = "Discord guild id")
    ),
    tag = "deployment",
    responses(
        (status = 200, description = "Deployment found", body = DeploymentResponse),
        (status = 404, description = "Deployment not found", body = crate::handlers::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn get_deployment_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<DeploymentResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    let session = require_session(&state.auth, &headers).await?;

    let deployment = state.deployments.get_deployment(&guild_id).await.map_err(|err| {
        error!(target: "oakmoss:api", guild_id, ?err, "failed to fetch deployment");
        ApiError::internal(err)
    })?;

    let Some(deployment) = deployment else {
        return Err(ApiError::not_found("deployment not found"));
    };

    ensure_guild_admin(&state, &identity, &guild_id).await?;
    ensure_guild_admin(&state.auth, &session, &guild_id).await?;

    Ok(ApiJson(Json(deployment.into())))
}
