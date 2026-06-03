import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const OUTPUT_PATH = "src/api/generated.rs";

function usage() {
  return `
Usage:
  npm run api:generate

Inputs, in priority order:
  STEEL_OPENAPI_URL   Fetch an OpenAPI spec from a published URL
  STEEL_OPENAPI_PATH  Read an OpenAPI spec from a local path
`.trim();
}

async function readSpec() {
  const url = process.env.STEEL_OPENAPI_URL?.trim();
  if (url) {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch ${url}: ${response.status} ${response.statusText}`);
    }
    const text = await response.text();
    return JSON.parse(text);
  }

  const filePath = process.env.STEEL_OPENAPI_PATH?.trim();
  if (!filePath) {
    throw new Error(usage());
  }

  try {
    const text = readFileSync(resolve(filePath), "utf8");
    return JSON.parse(text);
  } catch (error) {
    throw new Error(`${usage()}\n\nCould not read ${filePath}: ${error.message}`);
  }
}

function rustString(value) {
  return JSON.stringify(String(value ?? ""));
}

function snakeCase(value) {
  return String(value)
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/[^a-zA-Z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .toLowerCase();
}

function pascalCase(value) {
  return snakeCase(value)
    .split("_")
    .filter(Boolean)
    .map((part) => `${part[0].toUpperCase()}${part.slice(1)}`)
    .join("");
}

function stripV1(path) {
  return path.startsWith("/v1/") ? path.slice(3) : path;
}

function collectOperations(spec) {
  const operations = [];
  for (const [path, pathItem] of Object.entries(spec.paths || {})) {
    for (const [method, operation] of Object.entries(pathItem || {})) {
      const cli = operation["x-steel-cli"];
      if (!operation.operationId || !cli?.command) continue;
      operations.push({
        id: operation.operationId,
        method: method.toUpperCase(),
        path,
        requestPath: stripV1(path),
        summary: operation.summary || operation.operationId,
        command: cli.command.join(" "),
        example: exampleFor(cli.command, operation),
        streaming: cli.follow
          ? {
              transport: cli.follow.transport || "websocket",
              path: cli.follow.path,
            }
          : null,
        queryParams: (operation.parameters || []).filter((parameter) => parameter.in === "query"),
        pathParams: (operation.parameters || []).filter((parameter) => parameter.in === "path"),
      });
    }
  }
  operations.sort((a, b) => a.command.localeCompare(b.command));
  return operations;
}

function exampleFor(command, operation) {
  const base = `steel ${command.join(" ")}`;
  if (command.includes("list")) return `${base} --status live --limit 20`;
  if (command.includes("agent-logs")) return `${base} <session-id> --limit 100`;
  if (operation["x-steel-cli"]?.follow) return `${base} <session-id> --follow`;
  if (operation.parameters?.some((parameter) => parameter.in === "path" && parameter.name === "id")) {
    return `${base} <session-id>`;
  }
  return base;
}

function rustQueryType(parameter) {
  const schema = parameter.schema || {};
  if (schema.type === "array") return "Vec<String>";
  if (schema.type === "boolean") return "Option<bool>";
  if (schema.type === "integer") {
    if (parameter.name === "limit") return "Option<u16>";
    return "Option<u32>";
  }
  return "Option<String>";
}

function queryStructName(operation) {
  return `${pascalCase(operation.id)}Query`;
}

function renderQueryStruct(operation) {
  if (!operation.queryParams.length) return "";
  const fields = operation.queryParams
    .map((parameter) => `    pub ${snakeCase(parameter.name)}: ${rustQueryType(parameter)},`)
    .join("\n");
  return `
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ${queryStructName(operation)} {
${fields}
}
`;
}

function renderMetadata(operations) {
  const entries = operations
    .map((operation) => {
      const streaming = operation.streaming
        ? `Some(StreamingMetadata {
            transport: ${rustString(operation.streaming.transport)},
            path: ${rustString(operation.streaming.path)},
        })`
        : "None";
      return `    OperationMetadata {
        id: ${rustString(operation.id)},
        command: ${rustString(operation.command)},
        status: "implemented",
        method: ${rustString(operation.method)},
        path: ${rustString(operation.path)},
        summary: ${rustString(operation.summary)},
        example: ${rustString(operation.example)},
        streaming: ${streaming},
    },`;
    })
    .join("\n");
  return `pub const CLI_OPERATION_METADATA: &[OperationMetadata] = &[
${entries}
];`;
}

function renderPathBuilder(operation) {
  if (!operation.pathParams.length && !operation.queryParams.length) return "";

  const fnName = `build_${snakeCase(operation.id)}_path`;
  const pathArg = operation.pathParams.some((parameter) => parameter.name === "id")
    ? "session_id: &str"
    : "";
  const args = [pathArg].filter(Boolean).join(", ");
  const allArgs = [args, operation.queryParams.length ? `query: &${queryStructName(operation)}` : ""]
    .filter(Boolean)
    .join(", ");
  const requestPath = operation.requestPath.replace("{id}", "{session_id}");
  const initializer = requestPath.includes("{session_id}")
    ? `format!(${rustString(requestPath)})`
    : `${rustString(requestPath)}.to_string()`;
  const queryPushes = operation.queryParams
    .map((parameter) => {
      const field = snakeCase(parameter.name);
      const key = parameter.name;
      const type = rustQueryType(parameter);
      if (type === "Vec<String>") {
        return `    for value in query.${field}.iter().filter(|value| !value.trim().is_empty()) {
        push_query(&mut path, ${rustString(key)}, value.trim());
    }`;
      }
      if (type.startsWith("Option<String>")) {
        return `    if let Some(value) = query.${field}.as_deref().filter(|value| !value.trim().is_empty()) {
        push_query(&mut path, ${rustString(key)}, value.trim());
    }`;
      }
      return `    if let Some(value) = query.${field} {
        push_query_number(&mut path, ${rustString(key)}, value);
    }`;
    })
    .join("\n");

  if (!queryPushes) {
    return `fn ${fnName}(${allArgs}) -> String {
    ${initializer}
}`;
  }

  return `fn ${fnName}(${allArgs}) -> String {
    let mut path = ${initializer};
${queryPushes}
    path
}`;
}

function methodArgs(operation) {
  const args = [
    "&self",
    "base_url: &str",
    "mode: ApiMode",
    "auth: &Auth",
  ];
  if (operation.pathParams.some((parameter) => parameter.name === "id")) {
    args.push("session_id: &str");
  }
  if (operation.queryParams.length) {
    args.push(`query: &${queryStructName(operation)}`);
  }
  return args.join(",\n        ");
}

function pathBuilderCall(operation) {
  const fnName = `build_${snakeCase(operation.id)}_path`;
  const args = [];
  if (operation.pathParams.some((parameter) => parameter.name === "id")) args.push("session_id");
  if (operation.queryParams.length) args.push("query");
  return `${fnName}(${args.join(", ")})`;
}

function renderClientMethod(operation) {
  const requestPath = operation.queryParams.length || operation.pathParams.length
    ? `&${pathBuilderCall(operation)}`
    : rustString(operation.requestPath);
  return `    pub async fn cli_${snakeCase(operation.id)}(
        ${methodArgs(operation)},
    ) -> Result<Value, ApiError> {
        self.request(
            base_url,
            mode,
            reqwest::Method::${operation.method},
            ${requestPath},
            None,
            auth,
        )
        .await
    }`;
}

function renderTests(operations) {
  const getSessions = operations.find((operation) => operation.id === "get_sessions");
  const logs = operations.find((operation) => operation.id === "get_session_logs");
  if (!getSessions || !logs) return "";
  return `
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_list_query_uses_openapi_names() {
        let path = build_get_sessions_path(&GetSessionsQuery {
            cursor_id: Some("abc".into()),
            limit: Some(25),
            status: Some("live".into()),
        });
        assert_eq!(path, "/sessions?cursorId=abc&limit=25&status=live");
    }

    #[test]
    fn logs_query_repeats_event_types() {
        let path = build_get_session_logs_path(
            "sess",
            &GetSessionLogsQuery {
                namespace: Some("prod".into()),
                event_types: vec!["console".into(), "error".into()],
                limit: Some(10),
                offset: Some(5),
                ..Default::default()
            },
        );
        assert_eq!(
            path,
            "/sessions/sess/logs?namespace=prod&eventTypes=console&eventTypes=error&limit=10&offset=5"
        );
    }
}
`;
}

function renderRust(operations) {
  const queryStructs = operations.map(renderQueryStruct).filter(Boolean).join("\n");
  const pathBuilders = operations.map(renderPathBuilder).filter(Boolean).join("\n\n");
  const methods = operations.map(renderClientMethod).join("\n\n");

  return `//! Generated bindings for CLI-tagged Steel API operations.
//!
//! Source: OpenAPI operations with \`x-steel-cli\`.
//! Regenerate with \`npm run api:generate\`.
//! Keep command UX in \`commands/\`; this module only owns request shape,
//! endpoint paths, and operation metadata.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::client::{ApiError, SteelClient};
use crate::config::auth::Auth;
use crate::config::settings::ApiMode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OperationMetadata {
    pub id: &'static str,
    pub command: &'static str,
    pub status: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
    pub example: &'static str,
    pub streaming: Option<StreamingMetadata>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StreamingMetadata {
    pub transport: &'static str,
    pub path: &'static str,
}

${renderMetadata(operations)}
${queryStructs}
fn push_query(path: &mut String, key: &str, value: &str) {
    if path.contains('?') {
        path.push('&');
    } else {
        path.push('?');
    }
    path.push_str(key);
    path.push('=');
    path.push_str(&urlencoding::encode(value));
}

fn push_query_number(path: &mut String, key: &str, value: impl std::fmt::Display) {
    push_query(path, key, &value.to_string());
}

${pathBuilders}

impl SteelClient {
${methods}
}
${renderTests(operations)}`;
}

const spec = await readSpec();
const operations = collectOperations(spec);
if (!operations.length) {
  throw new Error("No CLI-tagged operations found in OpenAPI spec.");
}

mkdirSync(dirname(resolve(OUTPUT_PATH)), { recursive: true });
writeFileSync(resolve(OUTPUT_PATH), renderRust(operations));
console.log(`Generated ${OUTPUT_PATH} from ${operations.length} CLI OpenAPI operations.`);
