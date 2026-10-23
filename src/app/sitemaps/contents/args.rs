use regex::Regex;
use std::collections::HashMap;
use strum_macros::{Display, EnumIter};

use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

#[derive(Type, Deserialize, Serialize, PartialEq, EnumIter, Display)]
pub enum Source {
    #[strum(to_string = "Cadena de consulta")]
    Query,

    #[strum(to_string = "Ruta")]
    Path,

    #[strum(to_string = "Estatico")]
    Static,
}

impl Source {
    pub fn map_args(
        &self,
        query: &HashMap<String, String>,
        page_path: &str,
        request_path: &str,
        args: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, anyhow::Error> {
        match self {
            Source::Path => {
                let re = Regex::new(&page_path.replace("%", "(.+)"))?;

                let captures = re
                    .captures(request_path)
                    .ok_or(anyhow::anyhow!("No match"))?;

                let matches = captures
                    .iter()
                    .skip(1)
                    .filter_map(|c| c.map(|c| c.as_str()))
                    .collect::<Vec<_>>();

                let args = args
                    .keys()
                    .zip(matches)
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<HashMap<String, String>>();

                Ok(args)
            }
            Source::Query => Ok(args
                .iter()
                .filter_map(|(k, v)| query.get(v).cloned().map(|new_v| (k.clone(), new_v)))
                .collect()),
            _ => Ok(args.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        name: &'static str,
        source: Source,
        query_map: &'static [(&'static str, &'static str)],
        page_path: &'static str,
        request_path: &'static str,
        args: &'static [(&'static str, &'static str)],
        expected: &'static [(&'static str, &'static str)],
    }

    #[test]
    fn test_map_args() {
        let tests = vec![
            Test {
                name: "test_query_args",
                source: Source::Query,
                query_map: &[("name", "product"), ("entity", "example"), ("non", "nox")],
                page_path: "",
                request_path: "",
                args: &[("name", "name"), ("entity", "entity")],
                expected: &[("name", "product"), ("entity", "example")],
            },
            Test {
                name: "test_path_args",
                source: Source::Path,
                query_map: &[],
                page_path: "/products/%",
                request_path: "/products/product_slug",
                args: &[("slug", "")],
                expected: &[("slug", "product_slug")],
            },
        ];

        for test in tests {
            let query_map = test
                .query_map
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>();

            let args = test
                .args
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>();

            let result = test
                .source
                .map_args(&query_map, test.page_path, test.request_path, &args)
                .expect("expected to get mapped args");

            let expected = test
                .expected
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>();

            assert_eq!(result, expected, "test {}", test.name);
        }
    }
}
