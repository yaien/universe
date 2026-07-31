#![allow(dead_code)]

use std::collections::HashMap;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use sqlx::types::JsonValue;
use tokio::sync::{mpsc, watch};
use tokio::time::{Duration, sleep};

use crate::infra::{DbPool, Id};

struct Handler<T> {
    name: String,
    max_retries: Option<u8>,
    timeout: Option<Duration>,
    handle: Arc<dyn Fn(T, Value) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>>>,
}

impl<T> Handler<T> {
    pub fn builder(to: &'static str) -> HandlerBuilder {
        HandlerBuilder::new(to)
    }
}

struct HandlerBuilder {
    name: &'static str,
    max_retries: Option<u8>,
    timeout: Option<Duration>,
}

impl HandlerBuilder {
    pub fn new(name: &'static str) -> Self {
        Self {
            name: name,
            max_retries: None,
            timeout: None,
        }
    }

    pub fn with_max_retries(&mut self, max_retries: u8) -> &mut Self {
        self.max_retries = Some(max_retries);
        self
    }

    pub fn with_timeout(&mut self, d: Duration) -> &mut Self {
        self.timeout = Some(d);
        self
    }

    pub fn handle<T, S, F, Fut>(&mut self, f: F) -> Handler<T>
    where
        S: DeserializeOwned,
        F: Fn(T, S) -> Fut + Clone + 'static,
        T: Clone + 'static,
        Fut: Future<Output = anyhow::Result<()>>,
    {
        Handler {
            name: self.name.to_string(),
            max_retries: self.max_retries,
            timeout: self.timeout,
            handle: Arc::new(move |state, value| {
                let f = f.clone();
                let state = state.clone();
                Box::pin(async move {
                    let data =
                        serde_json::from_value::<S>(value).context("failed converting value")?;
                    f(state, data).await.context("failed on function")?;
                    Ok(())
                })
            }),
        }
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "INTEGER")]
#[repr(u8)]
enum Status {
    Pending = 1,
    Processing = 2,
    Completed = 3,
    Failed = 4,
}

#[derive(FromRow)]
struct Job {
    pub id: Id,
    pub name: String,
    pub data: JsonValue,
    pub status: Status,
    pub updated_at: DateTime<Utc>,
    pub error: Option<String>,
}

struct Worker<T: Clone + Sync + Send> {
    state: T,
    handlers: HashMap<String, Handler<T>>,
    pool: DbPool,
    channel: (mpsc::Sender<Job>, mpsc::Receiver<Job>),
    watch: Mutex<(watch::Sender<bool>, watch::Receiver<bool>)>,
}

impl<T: Clone + Send + Sync> Worker<T> {
    pub fn new(state: T, pool: DbPool) -> Self {
        Self {
            state,
            pool,
            handlers: HashMap::new(),
            channel: mpsc::channel(5),
            watch: Mutex::new(watch::channel(false)),
        }
    }

    pub fn handle(&mut self, h: Handler<T>) {
        self.handlers.insert(h.name.clone(), h);
    }

    pub async fn queue<S: Serialize>(&mut self, name: &str, data: S) -> anyhow::Result<()> {
        let value = serde_json::to_value(data)?;

        let job = sqlx::query_as::<_, Job>(
            "insert into jobs(name, data, status) values ($1, $2, $3) returning *",
        )
        .bind(name)
        .bind(value)
        .bind(Status::Pending)
        .fetch_one(&self.pool)
        .await
        .context("failed inserting job")?;

        self.channel.0.try_send(job).ok();

        Ok(())
    }

    pub async fn start(&mut self) {
        self.listen().await;
    }

    pub async fn stop(&self) {
        let lock = self.watch.lock().unwrap();
        lock.0.send(true).ok();
    }

    async fn listen(&mut self) {
        while let Some(job) = self.channel.1.recv().await {
            let mut job = job;

            let Some(handler) = self.handlers.get(&job.name) else {
                continue;
            };

            job.status = Status::Processing;
            job.updated_at = Utc::now();

            sqlx::query("update jobs set status = $1, updated_at = $2 where id = $3")
                .bind(&job.status)
                .bind(&job.updated_at)
                .bind(&job.id)
                .execute(&self.pool)
                .await
                .unwrap();

            match (handler.handle)(self.state.clone(), job.data).await {
                Ok(()) => {
                    job.status = Status::Completed;
                }
                Err(e) => {
                    job.status = Status::Failed;
                    job.error = Some(e.to_string());
                }
            }

            sqlx::query("update jobs set status = $1, updated_at = $2, error = $3 where id = $4")
                .bind(&job.status)
                .bind(&job.updated_at)
                .bind(&job.error)
                .bind(&job.id)
                .execute(&self.pool)
                .await
                .unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Context;
    use serde::Deserialize;
    use sqlx::migrate;

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct Data {
        pub message: String,
    }

    #[test]
    fn test_handler_builder() {
        let handler = HandlerBuilder::new("game")
            .with_max_retries(10)
            .with_timeout(Duration::from_mins(5))
            .handle(async |state: String, data: Data| {
                println!("{}, {}", state, data.message);
                Ok(())
            });

        assert_eq!(handler.name, "game".to_string());
    }

    #[tokio::test]
    async fn test_worker_handle_queue() -> anyhow::Result<()> {
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
        migrate!().run(&pool).await.unwrap();

        let mut worker = Worker::new("state".to_string(), pool.clone());

        let f = async move |state: String, data: Data| -> anyhow::Result<()> {
            println!("message {}, captured {}", data.message, state);
            Ok(())
        };

        let h = HandlerBuilder::new("task")
            .with_timeout(Duration::from_mins(1))
            .with_max_retries(3)
            .handle(f);

        worker.handle(h);

        let data = Data {
            message: "nuevo mensaje".to_string(),
        };

        let listener = worker.listen();

        worker.queue("task", data).await.context("failed at queue");

        listener.await;

        Ok(())
    }
}
