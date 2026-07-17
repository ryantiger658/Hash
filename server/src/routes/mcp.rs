//! Stateless Streamable HTTP MCP endpoint.
//!
//! This implementation uses JSON-RPC request/response POSTs, which is the
//! simplest valid Streamable HTTP mode: no in-memory session or SSE stream is
//! needed for read-only vault tools.

use crate::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::{json, Value};
use std::sync::Arc;

const PROTOCOL_VERSION: &str = "2025-03-26";

pub async fn post_mcp(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> impl IntoResponse {
    if !origin_is_allowed(&state, &headers) {
        return (
            StatusCode::FORBIDDEN,
            Json(jsonrpc_error(Value::Null, -32000, "Origin is not allowed")),
        )
            .into_response();
    }

    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return Json(jsonrpc_error(id, -32600, "Invalid JSON-RPC request")).into_response();
    };
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

    let response = match method {
        "initialize" => jsonrpc_result(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "serverInfo": { "name": "hash", "version": env!("CARGO_PKG_VERSION") },
                "capabilities": { "tools": {} },
                "instructions": "Search and read the #ash vault. Search results include source URLs that open the original note."
            }),
        ),
        "notifications/initialized" => return StatusCode::ACCEPTED.into_response(),
        "ping" => jsonrpc_result(id, json!({})),
        "tools/list" => jsonrpc_result(id, tools_list()),
        "tools/call" => call_tool(&state, &headers, id, params),
        _ => jsonrpc_error(id, -32601, "Method not found"),
    };
    Json(response).into_response()
}

/// Stateless MCP does not maintain a server-to-client SSE stream.
pub async fn get_mcp() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

fn tools_list() -> Value {
    json!({ "tools": [
        {
            "name": "search_notes",
            "description": "Search #ash notes. Every result includes source_url, a link that opens the source note in #ash.",
            "inputSchema": { "type": "object", "properties": {
                "query": { "type": "string", "description": "Full-text query; supports tag:, title:, and path: prefixes." },
                "limit": { "type": "integer", "minimum": 1, "maximum": 50, "default": 10 }
            }, "required": ["query"] }
        },
        {
            "name": "read_note",
            "description": "Read a Markdown note by vault-relative path.",
            "inputSchema": { "type": "object", "properties": {
                "path": { "type": "string", "description": "Vault-relative Markdown path." }
            }, "required": ["path"] }
        },
    ]})
}

fn call_tool(state: &AppState, headers: &HeaderMap, id: Value, params: Value) -> Value {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return jsonrpc_error(id, -32602, "tools/call requires a tool name");
    };
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let result = match name {
        "search_notes" => search_notes(state, headers, args),
        "read_note" => read_note(state, headers, args),
        _ => Err(format!("Unknown tool: {name}")),
    };
    match result {
        Ok(value) => jsonrpc_result(id, tool_result(value, false)),
        Err(message) => jsonrpc_result(id, tool_result(json!({ "error": message }), true)),
    }
}

fn search_notes(state: &AppState, headers: &HeaderMap, args: Value) -> Result<Value, String> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if query.is_empty() {
        return Err("query must not be empty".into());
    }
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(10)
        .clamp(1, 50) as usize;
    let response = super::search::search_for_mcp(state, query, limit);
    let results: Vec<Value> = response
        .results
        .into_iter()
        .map(|result| {
            json!({
                "path": result.path,
                "title": result.title,
                "score": result.score,
                "snippets": result.snippets,
                "source_url": note_url(state, headers, &result.path),
            })
        })
        .collect();
    Ok(json!({ "total": response.total, "results": results }))
}

fn read_note(state: &AppState, headers: &HeaderMap, args: Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or("path is required")?;
    if !path.ends_with(".md") {
        return Err("Only Markdown notes can be read".into());
    }
    let content = state
        .vault
        .read_file(path)
        .map_err(|_| "Note not found".to_string())?;
    let content = String::from_utf8(content).map_err(|_| "Note is not UTF-8".to_string())?;
    Ok(json!({ "path": path, "source_url": note_url(state, headers, path), "content": content }))
}

fn tool_result(value: Value, is_error: bool) -> Value {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into());
    json!({ "content": [{ "type": "text", "text": text }], "structuredContent": value, "isError": is_error })
}

fn note_url(state: &AppState, headers: &HeaderMap, path: &str) -> String {
    let base = state
        .config
        .server
        .public_url
        .as_deref()
        .map(|url| url.trim_end_matches('/').to_string())
        .unwrap_or_else(|| {
            format!(
                "http://{}",
                headers
                    .get("host")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("localhost:3535")
            )
        });
    format!("{base}/?note={}", percent_encode(path))
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn origin_is_allowed(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get("origin").and_then(|value| value.to_str().ok()) else {
        return true;
    };
    if let Some(public_url) = &state.config.server.public_url {
        return origin == public_url.trim_end_matches('/');
    }
    let Some(host) = headers.get("host").and_then(|value| value.to_str().ok()) else {
        return false;
    };
    origin == format!("http://{host}") || origin == format!("https://{host}")
}

fn jsonrpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}
fn jsonrpc_error(id: Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
