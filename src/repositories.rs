use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use anyhow::{Context, Ok};

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

    fn write_store_ref(&self) -> RwLockWriteGuard<ItemDatas> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<ItemDatas> {
        self.store.read().unwrap()
    }
}

impl ItemRepository for ItemRepositoryForMemory {
    fn create(&self, payload: CreateItem) -> Item {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        // 書き方要件等
        let item = Item::new(id, payload.name.clone(), payload.price.clone(), payload.date.clone(), payload.store_name.clone());
        store.insert(id, item.clone());
        item
    }

    fn find(&self, id: i32) -> Option<Item> {
        let store = self.read_store_ref();
        store.get(&id).map(|item| item.clone())
    }

    fn all(&self) -> Vec<Item> {
        let store = self.read_store_ref();
        Vec::from_iter(store.values().map(|todo| todo.clone()))
    }

    fn update(&self, id: i32, payload: UpdateItem) -> anyhow::Result<Item> {
        let mut store = self.write_store_ref();
        let item = store
            .get(&id)
            .context(RepositoryError::NotFound(id))?;
        let name = payload.name.unwrap_or(item.name.clone());
        let price = payload.price.unwrap_or(item.price.clone());
        let date = payload.date.unwrap_or(item.date.clone());
        let store_name = payload.store_name.unwrap_or(item.store_name.clone());
        let item = Item {
            id,
            name,
            price,
            date,
            store_name,
        };
        store.insert(id, item.clone());
        Ok(item)
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
        Ok(())
    }
}
