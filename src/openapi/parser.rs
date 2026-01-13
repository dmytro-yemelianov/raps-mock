// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use crate::error::Result;
use crate::openapi::types::{HttpMethod, OpenApiSpec, RouteDefinition};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// Regex to convert OpenAPI path params {param} to Axum format :param
static PATH_PARAM_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([^}]+)\}").expect("Invalid path param regex"));

/// Regex to find camelCase boundaries for conversion to snake_case
static CAMEL_CASE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-z])([A-Z])").expect("Invalid camelCase regex"));

/// Parser for OpenAPI 3.0 specifications
pub struct OpenApiParser;

impl OpenApiParser {
    /// Parse all OpenAPI specs from a directory
    pub fn parse_directory(dir: &Path) -> Result<Vec<(String, OpenApiSpec)>> {
        let mut specs = Vec::new();

        if !dir.exists() {
            tracing::warn!("OpenAPI directory does not exist: {}", dir.display());
            return Ok(specs);
        }

        Self::walk_dir(dir, dir, &mut specs)?;

        Ok(specs)
    }

    fn walk_dir(
        base_dir: &Path,
        current_dir: &Path,
        specs: &mut Vec<(String, OpenApiSpec)>,
    ) -> Result<()> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::walk_dir(base_dir, &path, specs)?;
            } else if path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml" || ext == "json")
            {
                match Self::parse_file(&path) {
                    Ok(spec) => {
                        let rel_path = path.strip_prefix(base_dir).unwrap_or(&path);
                        let name = rel_path
                            .to_string_lossy()
                            .replace('\\', "/")
                            .replace(".yaml", "")
                            .replace(".yml", "")
                            .replace(".json", "");
                        specs.push((name, spec));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse a single OpenAPI YAML file
    pub fn parse_file(path: &Path) -> Result<OpenApiSpec> {
        let content = fs::read_to_string(path)?;
        let spec: OpenApiSpec = serde_yaml::from_str(&content)?;
        Ok(spec)
    }

    /// Extract route definitions from an OpenAPI spec
    pub fn extract_routes(spec: &OpenApiSpec) -> Vec<RouteDefinition> {
        let mut routes = Vec::new();

        for (path, path_item) in &spec.paths {
            let path_pattern = Self::convert_path_to_pattern(path);

            // Extract GET operation
            if let Some(op) = &path_item.get {
                routes.push(RouteDefinition {
                    method: HttpMethod::Get,
                    path: path.clone(),
                    operation: op.clone(),
                    path_pattern: path_pattern.clone(),
                    components: spec.components.clone(),
                });
            }

            // Extract POST operation
            if let Some(op) = &path_item.post {
                routes.push(RouteDefinition {
                    method: HttpMethod::Post,
                    path: path.clone(),
                    operation: op.clone(),
                    path_pattern: path_pattern.clone(),
                    components: spec.components.clone(),
                });
            }

            // Extract PUT operation
            if let Some(op) = &path_item.put {
                routes.push(RouteDefinition {
                    method: HttpMethod::Put,
                    path: path.clone(),
                    operation: op.clone(),
                    path_pattern: path_pattern.clone(),
                    components: spec.components.clone(),
                });
            }

            // Extract DELETE operation
            if let Some(op) = &path_item.delete {
                routes.push(RouteDefinition {
                    method: HttpMethod::Delete,
                    path: path.clone(),
                    operation: op.clone(),
                    path_pattern: path_pattern.clone(),
                    components: spec.components.clone(),
                });
            }

            // Extract PATCH operation
            if let Some(op) = &path_item.patch {
                routes.push(RouteDefinition {
                    method: HttpMethod::Patch,
                    path: path.clone(),
                    operation: op.clone(),
                    path_pattern: path_pattern.clone(),
                    components: spec.components.clone(),
                });
            }
        }

        routes
    }

    /// Convert OpenAPI path pattern to Axum-compatible pattern
    /// e.g., /buckets/{bucketKey} -> /buckets/:bucket_key
    /// Normalizes parameter names to snake_case to avoid Axum routing conflicts
    fn convert_path_to_pattern(path: &str) -> String {
        // OpenAPI uses {param}, Axum uses :param
        // Also normalize camelCase to snake_case to avoid conflicts like :hubId vs :hub_id
        PATH_PARAM_REGEX
            .replace_all(path, |caps: &regex::Captures| {
                let param_name = &caps[1];
                let snake_case = CAMEL_CASE_REGEX
                    .replace_all(param_name, "${1}_${2}")
                    .to_lowercase();
                format!(":{}", snake_case)
            })
            .to_string()
    }
}
