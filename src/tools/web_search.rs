use super::{ToolContext, error_result, text_result};
use rmcp::model::CallToolResult;
use scraper::{Html, Selector};

const DDG_URL: &str = "https://html.duckduckgo.com/html/";

#[derive(serde::Deserialize)]
pub struct WebSearchInput {
    pub query: String,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

fn default_max_results() -> usize {
    5
}

pub async fn handle(ctx: &ToolContext, input: &WebSearchInput) -> CallToolResult {
    let url = format!("{DDG_URL}?q={}", urlencoding::encode(&input.query));

    let body = match ctx.http.get(&url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => return error_result(format!("Failed to read response: {e}")),
        },
        Err(e) => return error_result(format!("Network request failed: {e}")),
    };

    let results = parse_ddg_results(&body, input.max_results);

    if results.is_empty() {
        return text_result("No results found.".to_string());
    }

    let output = results
        .iter()
        .enumerate()
        .map(|(i, r)| format!("{}. {} - {} - {}", i + 1, r.title, r.url, r.snippet))
        .collect::<Vec<_>>()
        .join("\n");

    text_result(output)
}

struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

fn parse_ddg_results(html: &str, max: usize) -> Vec<SearchResult> {
    let document = Html::parse_fragment(html);
    let mut results = Vec::new();

    let result_sel = Selector::parse(".result").unwrap();
    let title_sel = Selector::parse(".result__a").unwrap();
    let snippet_sel = Selector::parse(".result__snippet").unwrap();

    for element in document.select(&result_sel) {
        let title_el = element.select(&title_sel).next();
        let snippet_el = element.select(&snippet_sel).next();

        if let Some(title_el) = title_el {
            let title = title_el.text().collect::<String>().trim().to_string();
            let url = title_el
                .attr("href")
                .map(|s| s.to_string())
                .unwrap_or_default();

            // DuckDuckGo wraps URLs in a redirect; extract the real URL
            let url = extract_real_url(&url);

            let snippet = snippet_el
                .map(|s| s.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            results.push(SearchResult {
                title,
                url,
                snippet,
            });

            if results.len() >= max {
                break;
            }
        }
    }

    results
}

/// DuckDuckGo HTML results use redirect URLs like `//duckduckgo.com/l/?uddg=<encoded>`.
/// Extract the actual target URL.
fn extract_real_url(raw: &str) -> String {
    if let Some(pos) = raw.find("uddg=") {
        let encoded = &raw[pos + 5..];
        // Take up to the next & if present
        let encoded = encoded.split('&').next().unwrap_or(encoded);
        urlencoding::decode(encoded)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| raw.to_string())
    } else if raw.starts_with("//") {
        format!("https:{raw}")
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_real_url() {
        let raw = "//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpage&rut=abc";
        let result = extract_real_url(raw);
        assert_eq!(result, "https://example.com/page");
    }

    #[test]
    fn test_extract_real_url_direct() {
        let raw = "https://example.com";
        assert_eq!(extract_real_url(raw), "https://example.com");
    }

    #[test]
    fn test_parse_ddg_results() {
        let html = r#"
        <div>
            <div class="result">
                <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Ftest">Example Title</a>
                <a class="result__snippet">This is a snippet.</a>
            </div>
            <div class="result">
                <a class="result__a" href="https://another.com">Another</a>
                <a class="result__snippet">Another snippet.</a>
            </div>
        </div>
        "#;
        let results = parse_ddg_results(html, 5);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Example Title");
        assert_eq!(results[0].url, "https://example.com/test");
        assert_eq!(results[0].snippet, "This is a snippet.");
    }

    #[test]
    fn test_parse_ddg_results_limit() {
        let html = r#"
        <div>
            <div class="result"><a class="result__a" href="https://a.com">A</a><a class="result__snippet">sa</a></div>
            <div class="result"><a class="result__a" href="https://b.com">B</a><a class="result__snippet">sb</a></div>
            <div class="result"><a class="result__a" href="https://c.com">C</a><a class="result__snippet">sc</a></div>
        </div>
        "#;
        let results = parse_ddg_results(html, 2);
        assert_eq!(results.len(), 2);
    }
}
