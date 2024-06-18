use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::repositories::{CreateItem, ItemRepository};

pub async fn create_item<T: ItemRepository>(
    Extension(repository): Extension<Arc<T>>,
    Json(payload): Json<CreateItem>,
) -> impl IntoResponse {
    let item = repository.create(payload);
    (StatusCode::CREATED, Json(item))
}
