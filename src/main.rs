use axum::{
    body::Bytes,
    extract::MatchedPath,
    http::{HeaderMap, Request, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::{Arc, RwLock}, time::Duration};
use thiserror::Error;
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

    // ルーティングを作成,どのパスでどのサービスへたどり着くかを設定する
    // getで受ける関数は最低限ブラウザで処理可能なテキスト(str?)をを返していればok
    // ルートがない場合は404を返す
    let app = Router::new().route("/", get(root)).layer(
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
            .on_response(|_response: &Response, _latency: Duration, _span: &Span| info!("response"))
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
    );

    // hyperでアプリを起動する
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Web家計簿解析アプリ</h1>")
}

// 書き方要確認
#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

pub trait ItemRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateItem) -> Item;
    fn find(&self, id: i32) -> Option<Item>;
    fn all(&self) -> Vec<Item>;
    fn update(&self, id: i32, payload: UpdateItem) -> anyhow::Result<Item>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

// 商品名,値段,費用分類,日付,店名
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Item {
    id: i32,
    name: String,
    price: i32,
    date: String,
    store_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateItem {
    name: String,
    price: i32,
    date: String,
    store_name: String,
}

// 一部の情報のみが渡されることを想定し，id以外の値はOption
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateItem {
    name: Option<String>,
    price: Option<i32>,
    date: Option<String>,
    store_name: Option<String>,
}

impl Item {
    pub fn new(id: i32, name: String, price: i32, date: String, store_name: String) -> Self {
        Self {
            id,
            name,
            price,
            date,
            store_name,
        }
    }
}

type ItemDatas = HashMap<i32, Item>;

#[derive(Debug, Clone)]
pub struct ItemRepositoryForMemory {
    store: Arc<RwLock<ItemDatas>>,
}

impl ItemRepositoryForMemory {
    pub fn new() -> Self {
        ItemRepositoryForMemory {
            store: Arc::default(),
        }
    }
}

impl ItemRepository for ItemRepositoryForMemory {
    fn create(&self, payload: CreateItem) -> Item {
        todo!()
    }

    fn find(&self, id: i32) -> Option<Item> {
        todo!()
    }

    fn all(&self) -> Vec<Item> {
        todo!()
    }

    fn update(&self, id: i32, payload: UpdateItem) -> anyhow::Result<Item> {
        todo!()
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        todo!()
    }
}