use axum::{Json, extract::State};
use tracing::error;

use crate::{
    handlers::{error::ApiError, response::ApiJson},
    state::AppState,
};

use super::DeploymentResponse;

/// List every stored deployment.
#[utoipa::path(
    get,
    path = "/deployments",
    tag = "deployment",
    responses(
        (status = 200, description = "Deployments retrieved", body = [DeploymentResponse]),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn list_deployments_handler(
    State(state): State<AppState>,
) -> Result<ApiJson<Vec<DeploymentResponse>>, ApiError> {
    let deployments = state.deployments.list_deployments().await.map_err(|err| {
        error!(target: "oakmoss:api", ?err, "failed to list deployments");
        ApiError::internal(err)
    })?;

    let response = deployments.into_iter().map(DeploymentResponse::from).collect();

    Ok(ApiJson(Json(response)))
}
