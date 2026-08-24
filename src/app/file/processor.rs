use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::app::Files;
use crate::infra::{Data, Id, Processor, Task};

#[derive(Serialize, Deserialize)]
pub struct FileProcessTask {
    pub file_id: Id,
    pub organization_id: Id,
}

impl Task for FileProcessTask {
    fn name() -> &'static str {
        "file_process"
    }
}

pub struct FileProcessor {
    files: Arc<Files>,
}

impl FileProcessor {
    pub fn new(files: Arc<Files>) -> Self {
        Self { files }
    }
}

impl Processor for FileProcessor {
    fn name(&self) -> &'static str {
        FileProcessTask::name()
    }

    fn process(&self, data: Data) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        let files = self.files.clone();
        Box::pin(async move {
            let task: FileProcessTask = data.try_into()?;
            files.convert(&task.file_id, &task.organization_id).await?;
            Ok(())
        })
    }
}
