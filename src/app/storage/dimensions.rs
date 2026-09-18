use std::cmp::max;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use image::ImageReader;
use mime::Mime;

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub variant: u32,
}

pub fn get_dimensions_by_content_type(path: &PathBuf, content_type: &Mime) -> Result<Dimensions> {
    match content_type.type_() {
        mime::VIDEO => get_video_dimension(&path),
        mime::IMAGE => get_image_dimension(&path),
        other => Err(anyhow!("invalid content type for dimension: {}", other)),
    }
}

pub fn get_image_dimension(path: &PathBuf) -> Result<Dimensions, anyhow::Error> {
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

pub fn get_video_dimension(path: &PathBuf) -> Result<Dimensions> {
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

#[cfg(test)]
mod tests {

    use super::*;
    use std::time::{Duration, Instant};

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
}
