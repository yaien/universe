use std::collections::HashMap;
use std::pin::Pin;

use anyhow::{Context, anyhow};
use chrono::{DateTime, Utc};
use log::error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use sqlx::types::JsonValue;
use tokio::sync::{mpsc, watch};
use tokio::time::{Duration, sleep};

use crate::infra::{DbPool, Id};

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "SMALLINT")]
#[repr(u8)]
pub enum Status {
    Pending = 1,
    Processing = 2,
    Completed = 3,
    Failed = 4,
}

#[derive(FromRow)]
pub struct Job {
    pub id: Id,
    pub name: String,
    pub data: JsonValue,
    pub status: Status,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Queue {
    sender: mpsc::Sender<Job>,
    pool: DbPool,
}

impl Queue {
    pub fn new(pool: DbPool, sender: mpsc::Sender<Job>) -> Self {
        Self { sender, pool }
    }

    pub async fn push<T: Serialize>(&self, to: &str, v: &T) -> anyhow::Result<()> {
        let data = serde_json::to_string(v).context("failed converting value to json")?;

        let job = sqlx::query_as::<_, Job>(
            "insert into jobs(name, data, status) values ($1, $2, $3) returning *",
        )
        .bind(to)
        .bind(data)
        .bind(Status::Pending)
        .fetch_one(&self.pool)
        .await
        .context("failed inserting job")?;

        self.sender.try_send(job).ok();

        Ok(())
    }
}

pub struct Data(Value);

impl Data {
    fn try_into<T: DeserializeOwned>(self) -> anyhow::Result<T> {
        serde_json::from_value(self.0).context("failed at parsing json")
    }
}

pub trait Processor: Send {
    fn name(&self) -> &'static str;
    fn process(&self, data: Data) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;
}

pub struct Worker {
    pool: DbPool,
    receiver: mpsc::Receiver<Job>,
    cancel: watch::Receiver<bool>,
    processors: HashMap<String, Box<dyn Processor>>,
}

impl Worker {
    pub fn new(pool: DbPool, receiver: mpsc::Receiver<Job>, cancel: watch::Receiver<bool>) -> Self {
        let (cancelation_sender, cancelation_receiver) = watch::channel(false);
        Self {
            pool,
            receiver,
            cancel,
            processors: HashMap::new(),
        }
    }

    pub fn procesor(&mut self, processor: Box<dyn Processor>) -> &mut Self {
        self.processors
            .insert(processor.name().to_string(), processor);
        self
    }

    async fn process(&mut self, mut job: Job) {
        let Some(processor) = self.processors.get(&job.name) else {
            return;
        };

        job.status = Status::Processing;
        job.updated_at = Utc::now();

        match processor.process(Data(job.data)).await {
            Ok(_) => job.status = Status::Completed,
            Err(e) => {
                job.status = Status::Failed;
                job.error = Some(e.to_string());
            }
        }

        job.updated_at = Utc::now();

        let result =
            sqlx::query("update jobs set status = $1, error = $2, updated_at = $3 where id = $4")
                .bind(&job.status)
                .bind(&job.error)
                .bind(&job.updated_at)
                .bind(&job.id)
                .execute(&self.pool)
                .await;

        if let Err(e) = result {
            error!("failed updating job {}", e);
            return;
        }
    }

    pub async fn work(&mut self) {
        loop {
            tokio::select! {
                job = self.receiver.recv() => {

                    let Some(job) = job else {
                        break;
                    };

                    self.process(job).await;
                }
                _ = self.cancel.changed() => {
                    break
                }
            }
        }
    }
}

pub struct Fetcher {
    pool: DbPool,
    sender: mpsc::Sender<Job>,
}

impl Fetcher {
    pub fn new(pool: DbPool, sender: mpsc::Sender<Job>) -> Self {
        Self { pool, sender }
    }

    pub async fn start(&self) {
        loop {
            let jobs = sqlx::query_as::<_, Job>("select * from jobs where status = $1 limit 10")
                .bind(Status::Pending)
                .fetch_all(&self.pool)
                .await;

            let jobs = match jobs {
                Ok(jobs) => jobs,
                Err(err) => {
                    error!("Error: failed fetching jobs: {}", err);
                    sleep(Duration::from_mins(1)).await;
                    continue;
                }
            };

            for job in jobs {
                self.sender
                    .send(job)
                    .await
                    .inspect_err(|e| error!("failed sending job throught channel sender: {e}"))
                    .ok();
            }

            sleep(Duration::from_mins(5)).await;
        }
    }
}

#[cfg(test)]
mod test {

    use std::time::Duration;

    use tokio::time::sleep;

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct Task {
        message: String,
    }

    pub struct Manager;

    impl Manager {
        const TASKNAME: &'static str = "task";
    }

    impl Processor for Manager {
        fn name(&self) -> &'static str {
            Self::TASKNAME
        }

        fn process(&self, data: Data) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
            Box::pin(async move {
                let task = data.try_into::<Task>()?;
                println!("Task message: {}", task.message);
                Ok(())
            })
        }
    }

    #[tokio::test]
    async fn test_queue_worker() -> anyhow::Result<()> {
        let pool = sqlx::SqlitePool::connect(":memory:").await?;
        sqlx::migrate!().run(&pool).await?;
        let (sender, receiver) = mpsc::channel(1);
        let (cancel_sender, cancel_receiver) = watch::channel(false);
        let queue = Queue::new(pool.clone(), sender.clone());

        let worker_pool = pool.clone();
        let worker = tokio::spawn(async {
            let manager = Box::new(Manager);
            let mut worker = Worker::new(worker_pool, receiver, cancel_receiver);
            worker.procesor(manager);
            worker.work().await;
        });

        let task = &Task {
            message: "hello_task".into(),
        };

        queue
            .push(Manager::TASKNAME, task)
            .await
            .context("failed pushing tash queue")?;

        sqlx::query_as::<_, Job>("select * from jobs where name = $1 and status = $2 limit 1")
            .bind(Manager::TASKNAME)
            .bind(Status::Pending)
            .fetch_one(&pool)
            .await
            .context("failed getting job queue")?;

        sleep(Duration::from_millis(100)).await;

        sqlx::query_as::<_, Job>("select * from jobs where name = $1 and status = $2 limit 1")
            .bind(Manager::TASKNAME)
            .bind(Status::Completed)
            .fetch_one(&pool)
            .await
            .context("failed getting completed")?;

        cancel_sender.send(true).unwrap();

        worker.await.expect("failed closing worker");

        Ok(())
    }

    #[tokio::test]
    async fn test_fetcher() -> anyhow::Result<()> {
        let pool = sqlx::SqlitePool::connect(":memory:").await?;
        sqlx::migrate!().run(&pool).await?;
        let (sender, mut receiver) = mpsc::channel(1);

        for i in 0..5 {
            let task = Task {
                message: format!("message from task {i}"),
            };

            let value = serde_json::to_value(&task).expect("failed serializing task");

            sqlx::query("insert into jobs(name, data, status) values ($1, $2, $3)")
                .bind("task")
                .bind(&value)
                .bind(Status::Pending)
                .execute(&pool)
                .await
                .expect("failed inserting job");
        }

        let fetcher_pool = pool.clone();

        tokio::spawn(async {
            let fetcher = Fetcher::new(fetcher_pool, sender);
            fetcher.start().await;
        });

        let mut count = 0;
        while let Some(_) = receiver.recv().await {
            count += 1;
            if count == 5 {
                break;
            }
        }

        assert_eq!(count, 5, "expected to receive 5 messages, received {count}");

        Ok(())
    }
}
