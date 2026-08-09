use std::cmp::max;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;

use actix_multipart::form::tempfile::TempFile;
use anyhow::{Context, anyhow, bail};
use chrono::{DateTime, Utc};
use image::{GenericImageView, ImageReader};
use mime::Mime;
use sqlx::prelude::FromRow;

use crate::infra::{DbPool, Id};

#[derive(FromRow)]
pub struct File {
    pub id: Id,
    pub organization_id: Id,
    pub name: String,
    pub preset: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[sqlx(skip)]
    pub formats: Vec<FileFormat>,
}

#[derive(FromRow)]
pub struct FileFormat {
    pub id: Id,
    pub file_id: Id,
    pub file_name: String,
    pub variant: u32,
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub content_type: String,
}

pub struct Files {
    pool: DbPool,
    path: PathBuf,
}

impl Files {
    pub fn new(pool: DbPool, path: PathBuf) -> Self {
        Self { pool, path }
    }

    pub async fn get_by_organization_id(
        &self,
        organization_id: &Id,
    ) -> Result<Vec<File>, sqlx::Error> {
        sqlx::query_as::<_, File>("select * from files where organization_id = $1")
            .bind(organization_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_one_by_organization_id_and_name(
        &self,
        organization_id: &Id,
        name: &str,
    ) -> Result<File, anyhow::Error> {
        let mut file = sqlx::query_as::<_, File>(
            "select * from files where organization_id = $1 and name = $2",
        )
        .bind(organization_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        let mut formats = sqlx::query_as::<_, FileFormat>(
            "select * from files_formats where file_id = $1 order by variant",
        )
        .bind(&file.id)
        .fetch_all(&self.pool)
        .await?;

        file.formats.append(&mut formats);

        Ok(file)
    }

    pub async fn get_one_by_organization_id_and_id(
        &self,
        organization_id: &Id,
        id: &Id,
    ) -> Result<File, anyhow::Error> {
        let mut file =
            sqlx::query_as::<_, File>("select * from files where organization_id = $1 and id = $2")
                .bind(organization_id)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        let mut formats = sqlx::query_as::<_, FileFormat>(
            "select * from files_formats where file_id = $1 order by variant",
        )
        .bind(&file.id)
        .fetch_all(&self.pool)
        .await?;

        file.formats.append(&mut formats);

        Ok(file)
    }

    pub async fn get_path_and_format<'a>(
        &self,
        file: &'a mut File,
        variant: &u32,
    ) -> anyhow::Result<(PathBuf, &'a FileFormat)> {
        let Some(format) = file.get_format(&variant) else {
            bail!("file format not found")
        };

        let path = self.path.join(&format.file_name);

        Ok((path, format))
    }

    pub async fn upload_many(
        &self,
        organization_id: &Id,
        files: Vec<TempFile>,
    ) -> Result<(), anyhow::Error> {
        for file in files.into_iter() {
            self.upload(organization_id, file).await?;
        }

        Ok(())
    }

    pub async fn upload(&self, organization_id: &Id, temp: TempFile) -> Result<(), anyhow::Error> {
        let temp_file_name = temp.file_name.ok_or(anyhow!("missing filename"))?;

        let (name, extension) = temp_file_name
            .rsplit_once(".")
            .filter(|(name, ext)| name.len() > 0 && ext.len() > 0)
            .ok_or(anyhow!("missing name or extension"))?;

        let count: u8 = sqlx::query_scalar(
            "select count(*) from files where organization_id = $1 and name = $2 limit 1",
        )
        .bind(organization_id)
        .bind(&name)
        .fetch_one(&self.pool)
        .await
        .context("failed checking if filename exists")?;

        if count > 0 {
            return Err(anyhow!("file with name {} already exists", name));
        }

        let content_type = temp.content_type.ok_or(anyhow!("missing content type"))?;
        let file_name = format!("{}.{}", uuid::Uuid::now_v7(), extension);
        let size = temp.size as u32;

        let dimensions =
            get_dimensions_by_content_type(&temp.file.path().to_path_buf(), &content_type)?;

        temp.file
            .persist(self.path.join(&file_name))
            .map_err(|e| anyhow!("failed persisting file: {}", e.error))?;

        let file_id =
            sqlx::query("insert into files(name, preset, organization_id) values ($1, $2, $3)")
                .bind(&name)
                .bind(content_type.type_().as_str())
                .bind(organization_id)
                .execute(&self.pool)
                .await
                .context("failed inserting file in db")?
                .last_insert_rowid();

        sqlx::query("insert into files_formats(file_id, file_name, size, variant, width, height, content_type) values ($1, $2, $3, $4, $5, $6, $7)")
            .bind(&file_id)
            .bind(&file_name)
            .bind(&size)
            .bind(&dimensions.variant)
            .bind(&dimensions.width)
            .bind(&dimensions.height)
            .bind(content_type.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_by_organization_id_and_id(
        &self,
        organization_id: &Id,
        id: &Id,
    ) -> Result<(), anyhow::Error> {
        // 1. Get the file to check its formats (optional but good practice for cascading deletes)
        let file = self
            .get_one_by_organization_id_and_id(organization_id, id)
            .await?;

        // 2. Delete associated records from files_formats
        sqlx::query!("DELETE FROM files_formats WHERE file_id = $1", &file.id)
            .execute(&self.pool)
            .await
            .context("failed deleting formats for file")?;

        // 3. Delete the file record
        sqlx::query!(
            "DELETE FROM files WHERE organization_id = $1 AND id = $2",
            organization_id,
            id
        )
        .execute(&self.pool)
        .await
        .context("failed deleting file from main table")?;

        // 4. Clean up the actual file on disk (assuming the path is derived from one of the formats)
        for format in file.formats.iter() {
            let path = self.path.join(&format.file_name);
            std::fs::remove_file(&path).ok();
        }

        Ok(())
    }

    pub async fn update_by_organization_id_and_id(
        &self,
        organization_id: &Id,
        id: &Id,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        let result =
            sqlx::query("UPDATE files SET name = $1 WHERE organization_id = $2 AND id = $3")
                .bind(name)
                .bind(organization_id)
                .bind(id)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
}

// Dimenstions width, height, variant
struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub variant: u32,
}

fn get_dimensions_by_content_type(
    path: &PathBuf,
    content_type: &Mime,
) -> Result<Dimensions, anyhow::Error> {
    match content_type.type_() {
        mime::VIDEO => get_video_dimension(&path),
        mime::IMAGE => get_image_dimension(&path),
        other => Err(anyhow!("invalid content type for dimension: {}", other)),
    }
}

fn get_image_dimension(path: &PathBuf) -> Result<Dimensions, anyhow::Error> {
    let content = std::fs::File::open(path)?;

    let image = ImageReader::new(BufReader::new(content))
        .with_guessed_format()?
        .decode()?;

    let (width, height) = image.dimensions();
    let variant = max(width, height);
    Ok(Dimensions {
        width,
        height,
        variant,
    })
}

fn get_video_dimension(path: &PathBuf) -> anyhow::Result<Dimensions> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=s=x:p=0",
            "-i",
            path.to_str().unwrap(),
        ])
        .output()
        .context("failed executing ffprobe")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("failed executing ffprobe: {}", stderr.trim());
    }

    let dimensions = String::from_utf8(output.stdout).context("failed decoding ffprobe output")?;
    let dimensions = dimensions.trim();

    let parts: Vec<&str> = dimensions.split('x').collect();
    if parts.len() != 2 {
        bail!("unexpected ffprobe output format: {:?}", dimensions);
    }

    let width: u32 = parts[0].parse().context("failed parsing width")?;
    let height: u32 = parts[1].parse().context("failed parsing height")?;
    let variant = height;

    Ok(Dimensions {
        width,
        height,
        variant,
    })
}

impl File {
    /// get_format returns the best format for the given variant.
    /// if the requested width is less than or equal to 0, or if there is only one format available,
    /// it returns the biggest format. Otherwise, it finds the nearest bigger or equal format based on the requested width.
    pub fn get_format(&mut self, query: &u32) -> Option<&FileFormat> {
        // if no format return None
        if self.formats.len() == 0 {
            return None;
        }

        // If there is only format just return it
        if self.formats.len() == 1 {
            return self.formats.first();
        }

        // Sort the formats by their variant (width) in ascending order
        self.formats.sort_by_key(|f| f.variant);

        // if query is zero return the best format
        if *query == 0 {
            return self.formats.last();
        }

        // find the near bigger or equal format based on the requested width
        for format in self.formats.iter() {
            // Find the first format that is smaller than or equal to the requested width
            if *query <= format.variant {
                return Some(format);
            }
        }

        // if not found return last format
        self.formats.last()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_formats() {
        struct Test {
            name: &'static str,
            variants: Vec<u32>,
            query: u32,
            want: Option<u32>,
        }

        let tests = [
            Test {
                name: "if only one format return it",
                variants: vec![100],
                query: 300,
                want: Some(100),
            },
            Test {
                name: "If only one format, return it",
                variants: vec![100],
                query: 300,
                want: Some(100),
            },
            Test {
                name: "If query is 0, return the best format",
                variants: vec![200, 300, 100],
                query: 0,
                want: Some(300),
            },
            Test {
                name: "If query is greater than all formats, return the best format",
                variants: vec![200, 300, 100],
                query: 400,
                want: Some(300),
            },
            Test {
                name: "If query is between formats, return the best format",
                variants: vec![200, 300, 100],
                query: 150,
                want: Some(200),
            },
            Test {
                name: "If no variants, return none",
                variants: vec![],
                query: 100,
                want: None,
            },
        ];

        for test in tests {
            let formats = test
                .variants
                .into_iter()
                .map(|v| FileFormat {
                    id: 0,
                    file_id: 0,
                    file_name: "file_name".into(),
                    variant: v,
                    size: 0,
                    width: 0,
                    height: 0,
                    content_type: "content_type".into(),
                })
                .collect();

            let mut file = File {
                id: 0,
                organization_id: 0,
                name: "file".into(),
                preset: "preset".into(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                formats,
            };

            let format = file.get_format(&test.query);

            assert_eq!(format.map(|f| f.variant), test.want, "{}", test.name);
        }
    }
}
