use crate::config::Config;
use crate::tools::{self, ToolContext};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, ListToolsResult, ServerCapabilities, ServerConfig,
    Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

pub struct NobodyMcpServer {
    pub config: Config,
    pub ctx: ToolContext,
}

impl NobodyMcpServer {
    pub fn new(config: Config) -> Self {
        let ctx = ToolContext::new(
            config.server_name.clone(),
            config.version.clone(),
        );
        Self { config, ctx }
    }
}

impl Clone for NobodyMcpServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            ctx: self.ctx.clone(),
        }
    }
}

impl ServerHandler for NobodyMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(rmcp::model::Implementation::new(
                &self.config.server_name,
                &self.config.version,
            ))
            .with_instructions("MCP service for nobody")
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: vec![
                Tool::new(
                    "hello",
                    "回傳 service 名稱、用途與版本資訊",
                    Arc::new(rmcp::model::object(json!({
                        "type": "object",
                        "properties": {}
                    }))),
                ),
                Tool::new(
                    "web_search",
                    "透過 DuckDuckGo 搜尋網頁並回傳結果",
                    Arc::new(rmcp::model::object(json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "搜尋關鍵字"
                            },
                            "max_results": {
                                "type": "integer",
                                "description": "回傳結果數量上限",
                                "default": 5
                            }
                        },
                        "required": ["query"]
                    }))),
                ),
                Tool::new(
                    "cve",
                    "根據產品清單從 cve.org 查詢相關的資安漏洞",
                    Arc::new(rmcp::model::object(json!({
                        "type": "object",
                        "properties": {
                            "products": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "產品名稱清單，例: [\"nginx\", \"openssl\"]"
                            },
                            "start_date": {
                                "type": "string",
                                "format": "date",
                                "description": "起始日期 (YYYY-MM-DD)"
                            },
                            "end_date": {
                                "type": "string",
                                "format": "date",
                                "description": "結束日期 (YYYY-MM-DD)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "每個產品回傳筆數上限",
                                "default": 10
                            }
                        },
                        "required": ["products"]
                    }))),
                ),
                Tool::new(
                    "price",
                    "根據產品型號從電商平台查詢價格",
                    Arc::new(rmcp::model::object(json!({
                        "type": "object",
                        "properties": {
                            "model": {
                                "type": "string",
                                "description": "產品型號，例: \"RTX 4090\""
                            },
                            "max_price": {
                                "type": "number",
                                "description": "價格上限（含）"
                            }
                        },
                        "required": ["model"]
                    }))),
                ),
                Tool::new(
                    "s2twp",
                    "將內文的簡體中文用詞轉換為台灣繁體中文用詞",
                    Arc::new(rmcp::model::object(json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "要轉換的簡體中文文字"
                            }
                        },
                        "required": ["text"]
                    }))),
                ),
            ],
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let params = request
            .arguments
            .as_ref()
            .cloned()
            .map(|m| serde_json::Value::Object(m))
            .unwrap_or(serde_json::Value::Null);

        info!("Tool call: {} with params: {}", request.name, params);

        let result = match request.name.as_ref() {
            "hello" => tools::hello::handle(&self.ctx).await,
            "web_search" => {
                let input: tools::web_search::WebSearchInput =
                    serde_json::from_value(params)
                        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                tools::web_search::handle(&self.ctx, &input).await
            }
            "cve" => {
                let input: tools::cve::CveInput = serde_json::from_value(params)
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                tools::cve::handle(&self.ctx, &input).await
            }
            "price" => {
                let input: tools::price::PriceInput = serde_json::from_value(params)
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                tools::price::handle(&self.ctx, &input).await
            }
            "s2twp" => {
                let input: tools::s2twp::S2TwpInput = serde_json::from_value(params)
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                tools::s2twp::handle(&self.ctx, &input).await
            }
            other => {
                return Err(ErrorData::invalid_params(
                    format!("Unknown tool: {other}"),
                    None,
                ))
            }
        };

        Ok(result.into())
    }
}
