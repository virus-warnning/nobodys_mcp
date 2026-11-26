use super::ToolContext;
use crate::tools::{error_result, text_result};
use opencc::OpenCC;
use rmcp::model::CallToolResult;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct S2TwpInput {
    pub text: String,
}

/// Create an OpenCC converter configured for S2TWP
/// (Simplified → Traditional, Taiwan variant with phrase rules).
fn s2twp_converter() -> OpenCC {
    OpenCC::new("s2twp.json")
}

pub async fn handle(_ctx: &ToolContext, input: &S2TwpInput) -> CallToolResult {
    let text = input.text.trim();

    if text.is_empty() {
        return error_result("輸入文字為空".to_string());
    }

    let converter = s2twp_converter();
    let converted = converter.convert(text);

    if converted.is_empty() {
        return error_result("轉換失敗，請確認輸入文字".to_string());
    }

    text_result(converted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_s2twp_basic() {
        let ctx = ToolContext::new("nobodys_mcp".into(), "0.1.0".into());
        let input = S2TwpInput {
            text: "电脑".to_string(),
        };
        let result = handle(&ctx, &input).await;
        assert!(!result.is_error.unwrap());
        let text = match &result.content[0] {
            rmcp::model::ContentBlock::Text(t) => &t.text,
            _ => panic!("expected text content"),
        };
        assert_eq!(text, "電腦");
    }

    #[tokio::test]
    async fn test_s2twp_empty() {
        let ctx = ToolContext::new("nobodys_mcp".into(), "0.1.0".into());
        let input = S2TwpInput {
            text: "   ".to_string(),
        };
        let result = handle(&ctx, &input).await;
        assert!(result.is_error.unwrap());
    }
}
