use super::ToolContext;
use crate::tools::text_result;
use rmcp::model::CallToolResult;

pub async fn handle(ctx: &ToolContext) -> CallToolResult {
    text_result(format!(
        "{} v{} - MCP service for nobody",
        ctx.server_name, ctx.version
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hello() {
        let ctx = ToolContext::new("nobodys_mcp".into(), "0.1.0".into());
        let result = handle(&ctx).await;
        assert!(!result.is_error.unwrap());
        let text = match &result.content[0] {
            rmcp::model::ContentBlock::Text(t) => &t.text,
            _ => panic!("expected text content"),
        };
        assert!(text.contains("nobodys_mcp"));
        assert!(text.contains("0.1.0"));
    }
}
