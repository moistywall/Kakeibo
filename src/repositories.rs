use anyhow::Ok;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use validator::Validate;

// 書き方要確認
#[derive(Debug, Error)]
enum RepositoryError {
    #[error("Unexpected Error: [{0}]")]
    Unexpected(String),
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

#[async_trait]
pub trait ItemRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateItem) -> anyhow::Result<Item>;
    async fn find(&self, id: i32) -> anyhow::Result<Item>;
    async fn all(&self) -> anyhow::Result<Vec<Item>>;
    async fn update(&self, id: i32, payload: UpdateItem) -> anyhow::Result<Item>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

// 商品名,値段,費用分類,日付,店名
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, FromRow)]
pub struct Item {
    id: i32,
    name: String,
    price: i32,
    date: String,
    store_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct CreateItem {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    name: String,
    price: i32,
    date: String,
    store_name: String,
}

// 一部の情報のみが渡されることを想定し，id以外の値はOption
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct UpdateItem {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    name: Option<String>,
    price: Option<i32>,
    date: Option<String>,
    store_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ItemRepositoryForDb {
    pool: PgPool,
}

impl ItemRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        ItemRepositoryForDb { pool }
    }
}

// ToDo! sqlのクエリ処理についてbindする値改変しないと多分ダメ
#[async_trait]
impl ItemRepository for ItemRepositoryForDb {
    async fn create(&self, payload: CreateItem) -> anyhow::Result<Item> {
        let item = sqlx::query_as::<_, Item>(
            r#"
            insert into items (name, price, date, store)
            values ($1, false)
            returning *
            "#,
        )
        .bind(payload.name.clone())
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Item> {
        let item = sqlx::query_as::<_, Item>(
            r#"
            slect * from items where id=$1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(item)
    }

    async fn all(&self) -> anyhow::Result<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>(
            r#"
            select * from items
            order by id desc;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    // 難しいので保留
    async fn update(&self, _id: i32, _payload: UpdateItem) -> anyhow::Result<Item> {
        todo!()
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            delete from items where id=$1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}

#[cfg(test)]
pub mod test_utils {
    use anyhow::Context;
    use axum::async_trait;
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    };

    use super::*;

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

    impl CreateItem {
        pub fn new(name: String, price: i32, date: String, store_name: String) -> Self {
            Self {
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

    #[async_trait]
    impl ItemRepository for ItemRepositoryForMemory {
        async fn create(&self, payload: CreateItem) -> anyhow::Result<Item> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            // 書き方要件等
            let item = Item::new(
                id,
                payload.name.clone(),
                payload.price.clone(),
                payload.date.clone(),
                payload.store_name.clone(),
            );
            store.insert(id, item.clone());
            Ok(item)
        }

        async fn find(&self, id: i32) -> anyhow::Result<Item> {
            let store = self.read_store_ref();
            let item = store
                .get(&id)
                .map(|item| item.clone())
                .ok_or(RepositoryError::NotFound(id))?;
            Ok(item)
        }

        async fn all(&self) -> anyhow::Result<Vec<Item>> {
            let store = self.read_store_ref();
            Ok(Vec::from_iter(store.values().map(|todo| todo.clone())))
        }

        async fn update(&self, id: i32, payload: UpdateItem) -> anyhow::Result<Item> {
            let mut store = self.write_store_ref();
            let item = store.get(&id).context(RepositoryError::NotFound(id))?;
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

        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut store = self.write_store_ref();
            store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
            Ok(())
        }
    }
}
