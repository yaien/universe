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
        var_args: &HashMap<String, String>,
        query_required_args: &[&str],
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

                if query_required_args.len() != matches.len() {
                    return Err(anyhow::anyhow!(
                        "expected {} matches, got {}",
                        query_required_args.len(),
                        matches.len()
                    ));
                }

                let args = query_required_args
                    .iter()
                    .zip(matches)
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<HashMap<String, String>>();

                Ok(args)
            }
            Source::Query => {
                let args = var_args
                    .iter()
                    .filter_map(|(k, v)| query.get(v).cloned().map(|new_v| (k.clone(), new_v)))
                    .collect::<HashMap<String, String>>();

                Ok(args)
            }
            _ => Ok(var_args.clone()),
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
        var_args: &'static [(&'static str, &'static str)],
        query_required_args: &'static [&'static str],
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
                var_args: &[("name", "name"), ("entity", "entity")],
                query_required_args: &[],
                expected: &[("name", "product"), ("entity", "example")],
            },
            Test {
                name: "test_path_args",
                source: Source::Path,
                query_map: &[],
                page_path: "/products/%",
                request_path: "/products/product_slug",
                var_args: &[],
                query_required_args: &["slug"],
                expected: &[("slug", "product_slug")],
            },
        ];

        for test in tests {
            let query_map = test
                .query_map
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>();

            let var_args = test
                .var_args
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>();

            let result = test
                .source
                .map_args(
                    &query_map,
                    test.page_path,
                    test.request_path,
                    &var_args,
                    &test.query_required_args,
                )
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
