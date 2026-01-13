// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI 3.0 specification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    pub servers: Option<Vec<Server>>,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(rename = "get")]
    pub get: Option<Operation>,
    #[serde(rename = "post")]
    pub post: Option<Operation>,
    #[serde(rename = "put")]
    pub put: Option<Operation>,
    #[serde(rename = "delete")]
    pub delete: Option<Operation>,
    #[serde(rename = "patch")]
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    pub tags: Option<Vec<String>>,
    pub security: Option<Vec<SecurityRequirement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Parameter {
    Ref {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
    Definition {
        name: String,
        #[serde(rename = "in")]
        location: ParameterLocation,
        required: Option<bool>,
        description: Option<String>,
        schema: Option<Box<Schema>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Path,
    Header,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub required: Option<bool>,
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Option<Schema>,
    pub example: Option<serde_json::Value>,
    pub examples: Option<HashMap<String, Example>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Ref {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
    Definition {
        description: String,
        content: Option<HashMap<String, MediaType>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Schema {
    Ref {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
    Object {
        #[serde(rename = "type")]
        type_name: Option<String>,
        format: Option<String>,
        items: Option<Box<Schema>>,
        properties: Option<HashMap<String, Schema>>,
        required: Option<Vec<String>>,
        enum_values: Option<Vec<serde_json::Value>>,
        example: Option<serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: Option<HashMap<String, Schema>>,
    pub responses: Option<HashMap<String, Response>>,
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SecurityScheme {
    OAuth2 {
        #[serde(rename = "type")]
        type_name: String,
        flows: OAuth2Flows,
    },
    ApiKey {
        #[serde(rename = "type")]
        type_name: String,
        #[serde(rename = "in")]
        location: String,
        name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Flows {
    pub authorization_code: Option<OAuth2Flow>,
    pub client_credentials: Option<OAuth2Flow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Flow {
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub scopes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    #[serde(flatten)]
    pub requirements: HashMap<String, Vec<String>>,
}

/// Route definition extracted from OpenAPI spec
#[derive(Debug, Clone)]
pub struct RouteDefinition {
    pub method: HttpMethod,
    pub path: String,
    pub operation: Operation,
    pub path_pattern: String, // With :param placeholders
    pub components: Option<Components>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
        }
    }
}
