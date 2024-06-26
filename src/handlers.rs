use axum::{extract::{Extension, Path}, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::repositories::{CreateItem, ItemRepository, UpdateItem};

pub async fn create_item<T: ItemRepository>(
    Extension(repository): Extension<Arc<T>>,
    Json(payload): Json<CreateItem>,
) -> impl IntoResponse {
    let item = repository.create(payload);
    (StatusCode::CREATED, Json(item))
}

pub async fn find_item<T: ItemRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    todo!();
    Ok(StatusCode::OK)  // 暫定OK
}

pub async fn all_item<T: ItemRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    todo!();        // 暫定OK
}

pub async fn update_item<T: ItemRepository>(    // 引数の順番に注意，patchでエラーが出ることがある，理由要確認
    Extension(repositoriry): Extension<Arc<T>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateItem>,
) -> Result<impl IntoResponse, StatusCode> {
    todo!();
    Ok(StatusCode::OK)      // 暫定OK
}

pub async fn delete_item<T: ItemRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    todo!();
}