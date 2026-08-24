use std::collections::HashSet;

use anyhow::anyhow;

use crate::app::file::conversions::*;
use crate::app::{File, FileFormat};

#[derive(Debug)]
pub struct ConversionState<'a> {
    pub biggest_format: &'a FileFormat,
    pub biggest_format_is_droppable: bool,
    pub missing_variants: Vec<u32>,
    pub preset: Preset,
}

/// get_conversion_state returns the conversion state of the file.
pub fn get_conversion_state<'a>(file: &'a File) -> Result<ConversionState<'a>, anyhow::Error> {
    let preset: Preset = file.preset.parse()?;

    let mut already = HashSet::new();

    let biggest_format = file
        .formats
        .iter()
        .max_by_key(|f| f.variant)
        .ok_or_else(|| anyhow!("missing max format"))?;

    let biggest_format_is_dropable = &biggest_format.variant > preset.max_variant();

    for format in &file.formats {
        if format.content_type == preset.target_content_type() {
            already.insert(format.variant);
        }
    }

    let mut missing_variants = Vec::new();

    for variant in preset.variants() {
        if already.contains(variant) || *variant > biggest_format.variant {
            continue;
        }

        missing_variants.push(variant.clone());
    }

    Ok(ConversionState {
        biggest_format,
        biggest_format_is_droppable: biggest_format_is_dropable,
        missing_variants,
        preset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
            missing_variants: Vec<u32>,
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
                    missing_variants: vec![320, 640, 1280],
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
                    missing_variants: vec![640, 1280],
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
                    missing_variants: vec![320],
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
                    missing_variants: vec![320, 640, 1280],
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
                    missing_variants: vec![320, 640],
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
                    missing_variants: vec![320, 640, 1280],
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
                name: "".to_string(),
                preset: test.preset.to_string(),
                formats: test
                    .formats
                    .iter()
                    .map(|f| FileFormat {
                        id: 0,
                        file_name: "".to_string(),
                        variant: f.variant.clone(),
                        size: 0,
                        width: 0,
                        height: 0,
                        content_type: f.content_type.into(),
                    })
                    .collect(),
            };

            let result = get_conversion_state(&file);

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
}
