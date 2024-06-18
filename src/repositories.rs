use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
