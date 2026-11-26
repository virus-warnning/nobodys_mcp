use super::{ToolContext, error_result, text_result};
use rmcp::model::CallToolResult;
use serde::Deserialize;

use crate::tools::web_search;

#[derive(Deserialize)]
pub struct PriceInput {
    pub model: String,
    pub max_price: Option<f64>,
}

pub async fn handle(ctx: &ToolContext, input: &PriceInput) -> CallToolResult {
    // Search for the product on DuckDuckGo with price-focused query
    let query = format!("{} price buy", input.model);
    let search_input = web_search::WebSearchInput {
        query,
        max_results: 10,
    };

    let result = web_search::handle(ctx, &search_input).await;

    // Extract text from the search result
    let text = match &result.content[0] {
        rmcp::model::ContentBlock::Text(t) => t.text.clone(),
        _ => return error_result("Failed to get search results".to_string()),
    };

    if text.starts_with("No results found") || text.starts_with("Error:") {
        return error_result(format!("Product not found: {}", input.model));
    }

    // Parse price information from search results
    let prices = extract_prices(&text, input.model.as_str());

    // Filter by max_price if specified
    let filtered: Vec<&PriceMatch> = match input.max_price {
        Some(max) => prices.iter().filter(|p| p.price <= max).collect(),
        None => prices.iter().collect(),
    };

    if filtered.is_empty() {
        return text_result(format!(
            "No pricing information found for '{}'.",
            input.model
        ));
    }

    let output = filtered
        .iter()
        .map(|p| format!("Product: {} - Price: ${} (Source: {})", input.model, p.price, p.source))
        .collect::<Vec<_>>()
        .join("\n");

    text_result(output)
}

struct PriceMatch {
    price: f64,
    source: String,
}

/// Try to extract price values from search result text.
/// Looks for patterns like "$1,234", "NT$50,000", "€999" in the snippets.
fn extract_prices(text: &str, _model: &str) -> Vec<PriceMatch> {
    let re = regex::Regex::new(r"(?:\$|NT\$|€|¥)\s*([\d,]+(?:\.\d+)?)").unwrap();
    let mut prices = Vec::new();

    for line in text.lines() {
        for cap in re.captures_iter(line) {
            let amount_str = &cap[1];
            let amount: f64 = amount_str
                .replace(',', "")
                .parse()
                .unwrap_or(0.0);

            if amount > 0.0 && amount < 1_000_000.0 {
                // Extract source (the URL portion of the line)
                let source = line
                    .split(" - ")
                    .nth(1)
                    .map(|s| s.split(" - ").next().unwrap_or(s).to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                prices.push(PriceMatch {
                    price: amount,
                    source,
                });
            }
        }
    }

    // Deduplicate and limit results
    prices.dedup_by(|a, b| a.price == b.price && a.source == b.source);
    prices.truncate(5);
    prices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prices() {
        let text = "1. RTX 4090 - $1599 - https://store.example.com\n2. RTX 4090 - $1499 - https://shop.example.com";
        let prices = extract_prices(text, "RTX 4090");
        assert!(!prices.is_empty());
        assert!(prices[0].price > 0.0);
    }

    #[test]
    fn test_extract_prices_ntd() {
        let text = "1. MacBook Pro - NT$89,900 - https://apple.com/tw";
        let prices = extract_prices(text, "MacBook Pro");
        assert!(!prices.is_empty());
        assert_eq!(prices[0].price, 89900.0);
    }

    #[test]
    fn test_extract_prices_none() {
        let text = "No price information available.";
        let prices = extract_prices(text, "product");
        assert!(prices.is_empty());
    }
}
