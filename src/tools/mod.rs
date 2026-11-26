pub mod cve;
pub mod hello;
pub mod price;
pub mod s2twp;
pub mod web_search;

use rmcp::model::{CallToolResult, ContentBlock, TextContent};

/// Shared context passed to all tool handlers.
#[derive(Clone)]
pub struct ToolContext {
    pub http: reqwest::Client,
    pub server_name: String,
    pub version: String,
}

impl ToolContext {
    pub fn new(server_name: String, version: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            server_name,
            version,
        }
    }
}

/// Create a successful text content result.
pub fn text_result(text: String) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::Text(TextContent::new(text))])
}

/// Create an error text content result.
pub fn error_result(message: String) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::Text(TextContent::new(
        format!("Error: {message}"),
    ))])
}
