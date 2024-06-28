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
use handlers::{all_item, create_item, delete_item, find_item, update_item};
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
        .route("/items", post(create_item::<T>).get(all_item::<T>))
        .route(
            "/items:id",
            get(find_item::<T>)
                .delete(delete_item::<T>)
                .patch(update_item::<T>),
        )
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::repositories::{CreateItem, Item};
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        response::Response,
        Json,
    };
    use serde::de::Expected;
    use tower::ServiceExt;

    fn build_item_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    fn build_item_req_with_empty(method: Method, path: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_item(res: Response) -> Item {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        // 以下の処理でserde_json::from_str()の後，3つ目の"124"をi32に変える必要がある．
        let item: Item = serde_json::from_str(&body)
            .expect(&format!("cannot convert Item instance. body: {}", body));
        item
    }

    #[tokio::test]
    async fn should_create_item() {
        let expected = Item::new(
            1,
            "牛乳".to_string(),
            124,
            "2024-06-27".to_string(),
            "ベルクス".to_string(),
        );
        let repository = ItemRepositoryForMemory::new();
        let req = build_item_req_with_json(
            "/items",
            Method::POST,
            r#"{ 
                "name": "牛乳",
                "price": "124",
                "date": "2024-06-27",
                "store": "ベルクス",
            }"#.to_string(),
        );
        let res = create_app(repository).oneshot(req).await.unwrap();
        let item = res_to_item(res).await;
        assert_eq!(expected, item);
    }

    #[tokio::test]
    async fn should_find_item() {
        let expected = Item::new(
            1,
            "牛乳".to_string(),
            124,
            "2024-06-27".to_string(),
            "ベルクス".to_string(),
        );

        let repository = ItemRepositoryForMemory::new();
        repository.create(CreateItem::new("牛乳".to_string(), 124, "2024-06-27".to_string(), "ベルクス".to_string()));
        let req = build_item_req_with_empty(Method::GET, "/items");
        let res = create_app(repository).oneshot(req).await.unwrap();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let item: Vec<Item> = serde_json::from_str(&body).expect(&format!("cannot convert Item instance. body {}", body));
        assert_eq!(vec![expected], item);
    }
}
