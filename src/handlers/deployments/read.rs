use axum::{
    Json,
    extract::{Path, State},
};
use tracing::error;

use crate::{
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
) -> Result<ApiJson<DeploymentResponse>, ApiError> {
    let deployment = state.deployments.get_deployment(&guild_id).await.map_err(|err| {
        error!(target: "oakmoss:api", guild_id, ?err, "failed to fetch deployment");
        ApiError::internal(err)
    })?;

    match deployment {
        Some(deployment) => Ok(ApiJson(Json(deployment.into()))),
        None => Err(ApiError::not_found("deployment not found")),
    }
}
