use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use anyhow::{Result, bail};

use crate::app::storage::dimensions::*;

#[derive(Debug, PartialEq)]
pub enum Preset {
    Image,
    Video,
    Audio,
}

pub struct Conversion {
    pub variant: u32,
    pub file_name: String,
    pub content_type: &'static str,
    pub size: u32,
    pub width: u32,
    pub height: u32,
}

impl Preset {
    pub fn max_variant<'a>(&'a self) -> &'a u32 {
        self.variants().last().unwrap()
    }

    pub fn variants(&self) -> &[u32] {
        use Preset::*;
        match self {
            Image => &[320, 640, 1280],
            Video => &[480, 720],
            Audio => &[96],
        }
    }

    pub fn target_content_type(&self) -> &'static str {
        use Preset::*;
        match self {
            Image => "image/webp",
            Video => "video/webm",
            Audio => "audio/ogg",
        }
    }
}

impl FromStr for Preset {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "image" => Ok(Preset::Image),
            "video" => Ok(Preset::Video),
            "audio" => Ok(Preset::Audio),
            _ => bail!("invalid preset: {}", s),
        }
    }
}

pub fn convert_file(
    src: &Path,
    outdir: &Path,
    preset: &Preset,
    variants: &[u32],
) -> Result<Vec<Conversion>> {
    use Preset::*;
    match preset {
        Image => convert_image(src, outdir, variants),
        Video => convert_video(src, outdir, variants),
        Audio => convert_audio(src, outdir, variants),
    }
}

fn convert_image(src: &Path, outdir: &Path, variants: &[u32]) -> Result<Vec<Conversion>> {
    let mut conversions = Vec::new();

    for variant in variants {
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
            content_type: "image/webp",
            size: metadata.len() as u32,
            variant: dimensions.variant,
            width: dimensions.width,
            height: dimensions.height,
        });
    }

    Ok(conversions)
}

/// convert_video converts a video file to a different format or quality. it relies on ffmpeg for the actual conversion,
fn convert_video(src: &Path, outdir: &Path, variants: &[u32]) -> Result<Vec<Conversion>> {
    let mut conversions = Vec::new();
    for variant in variants {
        let file_name = format!("{}.webm", uuid::Uuid::now_v7());

        let output_file_path = outdir.join(&file_name);

        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(src)
            .arg("-vf")
            .arg(format!("scale=-2:{}", variant))
            .arg("-c:v")
            .arg("libvpx-vp9")
            .arg("-preset")
            .arg("medium")
            .arg("-crf")
            .arg("30")
            .arg("-movflags")
            .arg("+faststart")
            .arg("-c:a")
            .arg("libopus")
            .arg("-b:a")
            .arg("96k")
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
            content_type: "video/webm",
            variant: dimensions.variant,
            size: metadata.len() as u32,
            width: dimensions.width,
            height: dimensions.height,
        });
    }

    Ok(conversions)
}

/// Converts an audio file to the specified variants.
pub fn convert_audio(src: &Path, outdir: &Path, variants: &[u32]) -> Result<Vec<Conversion>> {
    let mut conversions = Vec::new();

    for &variant in variants {
        let file_name = format!("{}.ogg", uuid::Uuid::now_v7());
        let output_file_path = outdir.join(&file_name);

        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(src)
            .arg("-vn")
            .arg("-c:a")
            .arg("libopus")
            .arg("-b:a")
            .arg(format!("{}k", variant))
            .arg(&output_file_path)
            .output()
            .expect("failed to convert audio");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("failed executing ffprobe: {}", stderr.trim());
        }

        let metadata = fs::metadata(&output_file_path)?;

        let dimensions = get_audio_dimension(&output_file_path)?;

        conversions.push(Conversion {
            file_name,
            content_type: "audio/ogg",
            variant: dimensions.variant,
            size: metadata.len() as u32,
            width: dimensions.width,
            height: dimensions.height,
        });
    }

    Ok(conversions)
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_convert_files() {
        struct Test {
            name: &'static str,
            src: &'static str,
            preset: Preset,
            is_exact: bool,
        }

        let tests = [
            Test {
                name: "convert a big photo",
                src: "testdata/files/big_photo.jpg",
                preset: Preset::Image,
                is_exact: true,
            },
            Test {
                name: "convert a big video",
                src: "testdata/files/big_video.mp4",
                preset: Preset::Video,
                is_exact: true,
            },
            Test {
                name: "convert a big audio",
                src: "testdata/files/big_audio.mp3",
                preset: Preset::Audio,
                is_exact: false,
            },
        ];

        for test in tests {
            let src = PathBuf::from(test.src);

            let outdir = TempDir::new_in("data/tmp")
                .expect(&format!("[{}]: failed creating temp folder", test.name));

            let conversions =
                convert_file(&src, outdir.path(), &test.preset, test.preset.variants())
                    .expect(&format!("[{}]: convertion file", test.name));

            let variants = test.preset.variants();

            assert_eq!(
                conversions.len(),
                variants.len(),
                "[{}]: convertion count mismatch",
                test.name
            );

            for (i, convertion) in conversions.iter().enumerate() {
                if test.is_exact {
                    assert_eq!(
                        convertion.variant, variants[i],
                        "[{}]: variant mismatch at index {}",
                        test.name, i
                    );
                }

                let outfile = outdir.path().join(&convertion.file_name);

                let mime = mime_guess::from_path(&outfile).first().expect(&format!(
                    "[{}] failed getting mime type of conversion path",
                    test.name
                ));

                let dimensions = get_dimensions_by_content_type(&outfile, &mime).expect(&format!(
                    "[{}] failed getting dimensions by content type",
                    test.name
                ));

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
