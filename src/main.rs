mod handlers;
mod repositories;

use crate::repositories::{ItemRepository, ItemRepositoryForMemory};
use axum::{
    body::Bytes,
    extract::{Extension, MatchedPath},
    http::{HeaderMap, Request},
    response::{Html, Response},
    routing::{get, post},
    Router,
};
use handlers::create_item;
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{info, info_span, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kakeibo=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let repository = ItemRepositoryForMemory::new();
    // ルーティングを作成,どのパスでどのサービスへたどり着くかを設定する
    // getで受ける関数は最低限ブラウザで処理可能なテキスト(str?)をを返していればok
    // ルートがない場合は404を返す
    let app = create_app(repository);

    // hyperでアプリを起動する
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Web家計簿解析アプリ</h1>")
}

fn create_app<T: ItemRepository>(item: T) -> Router {
    Router::new()
        .route("/", get(root))
        // .route("/items", post((StatusCode::CREATED, Json(item2))))
        .route("/items", post(create_item::<T>))
        .layer(Extension(Arc::new(item)))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| info!("request"))
                .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
                    info!("response")
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    // ...
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {
                        // ...
                    },
                )
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        // ...
                    },
                ),
        )
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use axum::{
//         body::Body,
//         http::{header, Method, Request}, RequestPartsExt,
//     };
//     use tower::{Service, ServiceExt};

//     // // todo : エラーチェック
//     // #[tokio::test]
//     // async fn should_return_hello_world() {
//     //     let repository = ItemRepositoryForMemory::new();
//     //     let req = Request::builder().uri("/").body(Body::empty()).unwrap();
//     //     let res = create_app(repository).oneshot(req).await.unwrap();

//     //     let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
//     //     let body: String = String::from_utf8(bytes.to_vec()).unwrap();
//     //     assert!(body, "<h1>Web家計簿解析アプリ</h1>");
//     // }
// }
