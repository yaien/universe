use std::cmp::max;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;

use actix_multipart::form::tempfile::TempFile;
use anyhow::{Context, anyhow, bail};
use chrono::{DateTime, Utc};
use image::{ImageReader, image_dimensions};
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::prelude::FromRow;

use crate::infra::{Data, DbPool, Id, Processor, Queue, Task};

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

#[derive(FromRow, Debug)]
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
    queue: Arc<Queue>,
}

impl Files {
    pub fn new(pool: DbPool, queue: Arc<Queue>, path: PathBuf) -> Self {
        Self { pool, path, queue }
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

    pub async fn get_path_and_format<'f>(
        &self,
        file: &'f mut File,
        variant: &u32,
    ) -> anyhow::Result<(PathBuf, &'f FileFormat)> {
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
            get_dimensions_by_content_type(&temp.file.path().to_path_buf(), &content_type)
                .context("failed getting dimensions")?;

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

        self.queue
            .push(FileConversionTask {
                file_id,
                organization_id: organization_id.clone(),
            })
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

        // 3. Delete the file record
        sqlx::query("DELETE FROM files WHERE organization_id = $1 AND id = $2")
            .bind(organization_id)
            .bind(id)
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

    /// Convert checks if the file with the given ID has any missing variants based on its preset. If there are missing variants,
    /// it mounts the biggest format of the file, performs the necessary conversions to create the missing variants,
    /// and uploads the converted files back to the storage. Finally,
    /// it updates the file record in the repository with the new formats and removes the original biggest format if it is dropable.
    pub async fn convert(&self, file_id: &Id, organization_id: &Id) -> Result<(), anyhow::Error> {
        let file = self
            .get_one_by_organization_id_and_id(organization_id, file_id)
            .await?;

        let state = file.get_conversion_state()?;

        let filesrc = self.path.join(&state.biggest_format.file_name);

        let preset: Preset = file.preset.parse()?;

        let convertions = preset.convert_file(&filesrc, &self.path)?;

        for conversion in convertions {
            sqlx::query(r#"
                insert into files_formats (file_id, variant, file_name, content_type, size, width, height, variant)
                values ($1, $2, $3, $4, $5, $6, $7, $8)
            "#)
                .bind(file_id)
                .bind(conversion.variant)
                .bind(conversion.file_name)
                .bind(conversion.content_type)
                .bind(conversion.size)
                .bind(conversion.width)
                .bind(conversion.height)
                .bind(conversion.variant)
                .execute(&self.pool)
                .await?;
        }

        if state.biggest_format_is_droppable {
            sqlx::query("delete from files_formats where id = $1")
                .bind(state.biggest_format.id)
                .execute(&self.pool)
                .await?;

            fs::remove_file(self.path.join(&state.biggest_format.file_name)).ok();
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
    let (width, height) = ImageReader::open(path)?
        .with_guessed_format()?
        .into_dimensions()?;
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

#[derive(Debug, PartialEq)]
enum Preset {
    Image,
    Video,
}

struct Conversion {
    variant: u32,
    path: PathBuf,
    file_name: String,
    content_type: &'static str,
    size: u32,
    width: u32,
    height: u32,
}

impl Preset {
    pub fn max_variant(&self) -> &'static u32 {
        self.variants().last().unwrap()
    }

    pub fn variants(&self) -> &'static [u32] {
        use Preset::*;
        match self {
            Image => &[320, 640, 1280],
            Video => &[480, 720],
        }
    }

    pub fn target_content_type(&self) -> &'static str {
        use Preset::*;
        match self {
            Image => "image/webp",
            Video => "video/mp4",
        }
    }

    pub fn convert_file(
        &self,
        src: &Path,
        outdir: &Path,
    ) -> Result<Vec<Conversion>, anyhow::Error> {
        use Preset::*;
        match self {
            Image => self.convert_image(src, outdir),
            Video => self.convert_video(src, outdir),
        }
    }

    fn convert_image(&self, src: &Path, outdir: &Path) -> Result<Vec<Conversion>, anyhow::Error> {
        let mut conversions = Vec::new();

        for variant in self.variants() {
            let file_name = format!("{}.webp", uuid::Uuid::now_v7());
            let outfile = outdir.join(&file_name);

            let output = Command::new("vips")
                .arg("thumbnail")
                .arg(src)
                .arg(format!("{}[Q=75,strip]", outfile.display()))
                .arg(format!("{}", variant))
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("failed executing ffprobe: {}", stderr.trim());
            }

            let metadata = fs::metadata(&outfile)?;

            let dimensions = get_image_dimension(&outfile)?;

            conversions.push(Conversion {
                file_name,
                path: outfile,
                content_type: self.target_content_type(),
                size: metadata.len() as u32,
                variant: dimensions.variant,
                width: dimensions.width,
                height: dimensions.height,
            });
        }

        Ok(conversions)
    }

    /// convert_video converts a video file to a different format or quality. it relies on ffmpeg for the actual conversion,
    fn convert_video(&self, src: &Path, outdir: &Path) -> Result<Vec<Conversion>, anyhow::Error> {
        let mut conversions = Vec::new();
        for variant in self.variants() {
            let file_name = format!("{}.mp4", uuid::Uuid::now_v7());

            let output_file_path = outdir.join(&file_name);

            let output = Command::new("ffmpeg")
                .arg("-i")
                .arg(src)
                .arg("-vf")
                .arg(format!("scale=-2:{}", variant))
                .arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg("medium")
                .arg("-crf")
                .arg("23")
                .arg("-movflags")
                .arg("+faststart")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("128k")
                .arg(&output_file_path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("failed executing ffprobe: {}", stderr.trim());
            }

            let metadata = fs::metadata(&output_file_path)?;

            let dimensions = get_video_dimension(&output_file_path)?;

            conversions.push(Conversion {
                file_name,
                variant: dimensions.variant,
                path: output_file_path,
                content_type: self.target_content_type(),
                size: metadata.len() as u32,
                width: dimensions.width,
                height: dimensions.height,
            });
        }

        Ok(conversions)
    }
}

impl FromStr for Preset {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "image" => Ok(Preset::Image),
            "video" => Ok(Preset::Video),
            _ => bail!("invalid preset: {}", s),
        }
    }
}

#[derive(Debug)]
pub struct ConversionState<'a> {
    biggest_format: &'a FileFormat,
    biggest_format_is_droppable: bool,
    missing_variants: Vec<&'static u32>,
    preset: Preset,
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

    /// get_conversion_state returns the conversion state of the file.
    pub fn get_conversion_state<'a>(&'a self) -> Result<ConversionState<'a>, anyhow::Error> {
        let preset: Preset = self.preset.parse()?;

        let mut already = HashSet::new();

        let biggest_format = self
            .formats
            .iter()
            .max_by_key(|f| f.variant)
            .ok_or_else(|| anyhow!("missing max format"))?;

        let biggest_format_is_dropable = &biggest_format.variant > preset.max_variant()
            || biggest_format.content_type != preset.target_content_type();

        for format in &self.formats {
            if format.content_type == preset.target_content_type() {
                already.insert(format.variant);
            }
        }

        let mut missing_variants: Vec<&'static u32> = Vec::new();

        for variant in preset.variants() {
            if already.contains(variant) || *variant > biggest_format.variant {
                continue;
            }

            missing_variants.push(variant);
        }

        Ok(ConversionState {
            biggest_format,
            biggest_format_is_droppable: biggest_format_is_dropable,
            missing_variants,
            preset,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileConversionTask {
    pub file_id: Id,
    pub organization_id: Id,
}

impl Task for FileConversionTask {
    fn name() -> &'static str {
        "file_conversion"
    }
}

pub struct FileConversion {
    files: Arc<Files>,
}

impl FileConversion {
    pub fn new(files: Arc<Files>) -> Self {
        Self { files }
    }
}

impl Processor for FileConversion {
    fn name(&self) -> &'static str {
        FileConversionTask::name()
    }

    fn process(&self, data: Data) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>> {
        let files = self.files.clone();
        Box::pin(async move {
            let task: FileConversionTask = data.try_into()?;
            files.convert(&task.file_id, &task.organization_id).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {

    use std::time::{Duration, Instant};

    use tempfile::TempDir;

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

    #[test]
    fn get_file_dimensions() {
        struct Test {
            name: &'static str,
            filepath: &'static str,
            content_type: &'static str,
            width: u32,
            height: u32,
            variant: u32,
        }

        let tests = [
            Test {
                name: "big_photo",
                filepath: "testdata/files/big_photo.jpg",
                content_type: "image/jpeg",
                width: 3303,
                height: 4954,
                variant: 4954,
            },
            Test {
                name: "big_video",
                filepath: "testdata/files/big_video.mp4",
                content_type: "video/mp4",
                width: 1920,
                height: 1080,
                variant: 1080,
            },
        ];

        for test in tests {
            let filepath = PathBuf::from(test.filepath);
            let mime: Mime = test.content_type.parse().expect(&format!(
                "{}: failed at parsing content type: {}",
                test.name, test.content_type
            ));

            let start = Instant::now();

            let dimensions = match get_dimensions_by_content_type(&filepath, &mime) {
                Ok(dimensions) => dimensions,
                Err(e) => panic!(
                    "{}: failed at get dimensions file {}: {}",
                    test.name, test.filepath, e
                ),
            };

            assert_eq!(
                test.width, dimensions.width,
                "{}: expected width {}, got {}",
                test.name, test.width, dimensions.width
            );

            assert_eq!(
                test.height, dimensions.height,
                "{}: expected height {}, got {}",
                test.name, test.height, dimensions.height
            );

            assert_eq!(
                test.variant, dimensions.variant,
                "{}: expected quality {}, got {}",
                test.name, test.variant, dimensions.variant
            );

            let duration = start.elapsed();
            let limit = Duration::from_millis(500);

            assert!(
                duration < limit,
                "{}: expected duration to be < {}ms, got {}",
                test.name,
                limit.as_millis(),
                duration.as_millis()
            );
        }
    }

    #[test]
    fn test_file_conversion_state() {
        struct Format {
            content_type: &'static str,
            variant: u32,
        }

        struct Test {
            name: &'static str,
            formats: &'static [Format],
            preset: &'static str,
            result: Result<State, anyhow::Error>,
        }

        struct State {
            biggest_format: Format,
            biggest_format_is_droppable: bool,
            preset: Preset,
            missing_variants: Vec<&'static u32>,
        }

        let tests = [
            Test {
                name: "no formats",
                formats: &[],
                preset: "image",
                result: Err(anyhow::anyhow!("file has no formats")),
            },
            Test {
                name: "missing all formats",
                preset: "image",
                formats: &[Format {
                    content_type: "image/jpeg",
                    variant: 4954,
                }],
                result: Ok(State {
                    biggest_format: Format {
                        content_type: "image/jpeg",
                        variant: 4954,
                    },
                    biggest_format_is_droppable: true,
                    missing_variants: vec![&320, &640, &1280],
                    preset: Preset::Image,
                }),
            },
            Test {
                name: "missing some formats",
                preset: "image",
                formats: &[
                    Format {
                        content_type: "image/jpeg",
                        variant: 4954,
                    },
                    Format {
                        content_type: "image/webp",
                        variant: 320,
                    },
                ],
                result: Ok(State {
                    biggest_format: Format {
                        content_type: "image/jpeg",
                        variant: 4954,
                    },
                    biggest_format_is_droppable: true,
                    missing_variants: vec![&640, &1280],
                    preset: Preset::Image,
                }),
            },
            Test {
                name: "all formats present",
                preset: "image",
                formats: &[
                    Format {
                        content_type: "image/webp",
                        variant: 320,
                    },
                    Format {
                        content_type: "image/webp",
                        variant: 640,
                    },
                    Format {
                        content_type: "image/webp",
                        variant: 1280,
                    },
                    Format {
                        content_type: "image/jpeg",
                        variant: 4954,
                    },
                ],
                result: Ok(State {
                    biggest_format: Format {
                        content_type: "image/jpeg",
                        variant: 4954,
                    },
                    biggest_format_is_droppable: true,
                    missing_variants: vec![],
                    preset: Preset::Image,
                }),
            },
            Test {
                name: "biggest format is smaller than 1080 but greater than 640",
                preset: "image",
                formats: &[
                    Format {
                        content_type: "image/jpeg",
                        variant: 920,
                    },
                    Format {
                        content_type: "image/webp",
                        variant: 640,
                    },
                ],
                result: Ok(State {
                    biggest_format: Format {
                        content_type: "image/jpeg",
                        variant: 920,
                    },
                    biggest_format_is_droppable: true,
                    missing_variants: vec![&320],
                    preset: Preset::Image,
                }),
            },
            Test {
                name: "biggest format is equal to 1080, but its extension is not webp",
                preset: "image",
                formats: &[Format {
                    content_type: "image/jpeg",
                    variant: 1280,
                }],
                result: Ok(State {
                    biggest_format: Format {
                        content_type: "image/jpeg",
                        variant: 1280,
                    },
                    biggest_format_is_droppable: true,
                    missing_variants: vec![&320, &640, &1280],
                    preset: Preset::Image,
                }),
            },
            Test {
                name: "biggest format is not dropable because its extension is webp",
                preset: "image",
                formats: &[Format {
                    content_type: "image/webp",
                    variant: 1280,
                }],
                result: Ok(State {
                    biggest_format: Format {
                        content_type: "image/webp",
                        variant: 1280,
                    },
                    biggest_format_is_droppable: false,
                    missing_variants: vec![&320, &640],
                    preset: Preset::Image,
                }),
            },
            Test {
                name: "biggest format is dropable because variant is bigger than 1280",
                preset: "image",
                formats: &[Format {
                    content_type: "image/webp",
                    variant: 3000,
                }],
                result: Ok(State {
                    biggest_format: Format {
                        content_type: "image/webp",
                        variant: 3000,
                    },
                    biggest_format_is_droppable: true,
                    missing_variants: vec![&320, &640, &1280],
                    preset: Preset::Image,
                }),
            },
            Test {
                name: "file has no conversion preset",
                preset: "application",
                formats: &[Format {
                    content_type: "application/xml",
                    variant: 0,
                }],
                result: Err(anyhow::anyhow!("file has no conversion preset")),
            },
        ];

        for test in tests {
            let file = File {
                id: 0,
                organization_id: 0,
                name: "".to_string(),
                preset: test.preset.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                formats: test
                    .formats
                    .iter()
                    .map(|f| FileFormat {
                        id: 0,
                        file_id: 0,
                        file_name: "".to_string(),
                        variant: f.variant.clone(),
                        size: 0,
                        width: 0,
                        height: 0,
                        content_type: f.content_type.into(),
                    })
                    .collect(),
            };

            let result = file.get_conversion_state();

            let expected = match test.result {
                Ok(state) => state,
                Err(err) => {
                    result.expect_err(&err.to_string());
                    return;
                }
            };

            let state = result.expect("failed at getting state");

            assert_eq!(
                state.biggest_format.content_type, expected.biggest_format.content_type,
                "[{}]: biggest format content type",
                test.name
            );

            assert_eq!(
                state.biggest_format.variant, expected.biggest_format.variant,
                "[{}]: biggest format variant",
                test.name
            );

            assert_eq!(
                state.biggest_format_is_droppable, expected.biggest_format_is_droppable,
                "[{}]: biggest format is dropable",
                test.name
            );

            assert_eq!(state.preset, expected.preset, "[{}]: preset", test.name);

            assert_eq!(
                state.missing_variants, expected.missing_variants,
                "[{}]: missing variants",
                test.name
            );
        }
    }

    #[test]
    fn test_convert_files() {
        struct Test {
            name: &'static str,
            src: &'static str,
            preset: Preset,
        }

        let tests = [
            Test {
                name: "convert a big photo",
                src: "testdata/files/big_photo.jpg",
                preset: Preset::Image,
            },
            Test {
                name: "convert a big video",
                src: "testdata/files/big_video.mp4",
                preset: Preset::Video,
            },
        ];

        for test in tests {
            let src = PathBuf::from(test.src);

            let outdir = TempDir::new_in("data/tmp")
                .expect(&format!("[{}]: failed creating temp folder", test.name));

            let conversions = test
                .preset
                .convert_file(&src, outdir.path())
                .expect(&format!("[{}]: convertion file", test.name));

            let variants = test.preset.variants();

            assert_eq!(
                conversions.len(),
                variants.len(),
                "[{}]: convertion count mismatch",
                test.name
            );

            for (i, convertion) in conversions.iter().enumerate() {
                assert_eq!(
                    convertion.variant, variants[i],
                    "[{}]: variant mismatch at index {}",
                    test.name, i
                );

                let mime = mime_guess::from_path(&convertion.path)
                    .first()
                    .expect(&format!(
                        "[{}] failed getting mime type of conversion path",
                        test.name
                    ));

                let dimensions = get_dimensions_by_content_type(&convertion.path, &mime).expect(
                    &format!("[{}] failed getting dimensions by content type", test.name),
                );

                assert_eq!(
                    dimensions.width, convertion.width,
                    "[{}]: width mismatch at index {}",
                    test.name, i
                );
                assert_eq!(
                    dimensions.height, convertion.height,
                    "[{}]: height mismatch at index {}",
                    test.name, i
                );
                assert_eq!(
                    dimensions.variant, convertion.variant,
                    "[{}]: variant mismatch at index {}",
                    test.name, i
                );
            }
        }
    }
}
