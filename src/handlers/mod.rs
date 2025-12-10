use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::state::AppState;

pub mod deployments;
pub mod error;
pub mod health;
pub mod response;

/// Build the top-level router with API routes and interactive docs.
pub fn create_router(state: AppState) -> Router {
    #[derive(OpenApi)]
    #[openapi(
        nest(
            (path = "/deployments", api = deployments::DeploymentApi),
            (path = "/health", api = health::HealthApi)
        ),
        tags(
            (name = "oakmoss", description = "Oakmoss bot runtime API")
        )
    )]
    struct ApiDoc;

    let api_router =
        Router::new().merge(deployments::router()).route("/health", get(health::health_check));

    let oapi_router = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .route("/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    Router::new()
        .nest("/api-docs", oapi_router)
        // keep the old base paths available while also exposing `/api/*`
        .nest("/api", api_router.clone())
        .merge(api_router)
        .with_state(state)
}
