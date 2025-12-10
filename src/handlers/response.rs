use std::collections::BTreeMap;

use axum::{Json, response::IntoResponse};
use serde::Serialize;
use utoipa::openapi::{RefOr, content::ContentBuilder, response::ResponseBuilder};
use utoipa::{IntoResponses, ToSchema};

/// JSON wrapper that also carries utoipa response metadata.
#[derive(Debug)]
pub struct ApiJson<T>(pub Json<T>);

impl<T> IntoResponse for ApiJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

impl<T> From<Json<T>> for ApiJson<T> {
    fn from(value: Json<T>) -> Self {
        Self(value)
    }
}

impl<T> IntoResponses for ApiJson<T>
where
    T: ToSchema + Serialize,
{
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        let content = ContentBuilder::new().schema(Some(T::schema())).build();
        let response = ResponseBuilder::new()
            .description("Successful response")
            .content("application/json", content)
            .build();

        BTreeMap::from([("200".to_string(), RefOr::T(response))])
    }
}
