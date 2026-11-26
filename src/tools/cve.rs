use super::{ToolContext, text_result};
use chrono::NaiveDate;
use rmcp::model::CallToolResult;
use serde::Deserialize;
use tracing::warn;

const CVE_API: &str = "https://cveawg.mitre.org/api/cves";

#[derive(Deserialize)]
pub struct CveInput {
    pub products: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

pub async fn handle(ctx: &ToolContext, input: &CveInput) -> CallToolResult {
    let mut output_parts: Vec<String> = Vec::new();

    for product in &input.products {
        let cves = fetch_cves_for_product(ctx, product, input).await;
        let section = format_cve_section(product, &cves);
        output_parts.push(section);
    }

    text_result(output_parts.join("\n"))
}

async fn fetch_cves_for_product(
    ctx: &ToolContext,
    product: &str,
    input: &CveInput,
) -> Vec<CveEntry> {
    let url = format!("{CVE_API}?keywordSearch={product}");

    let resp = match ctx.http.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("CVE API request failed for '{product}': {e}");
            return Vec::new();
        }
    };

    if !resp.status().is_success() {
        warn!("CVE API returned status {} for '{product}'", resp.status());
        return Vec::new();
    }

    let body: CveApiResponse = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to parse CVE API response for '{product}': {e}");
            return Vec::new();
        }
    };

    let start = input
        .start_date
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let end = input
        .end_date
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    body.vulnerabilities
        .into_iter()
        .filter_map(|vuln| {
            let cve_id = vuln.cve_id?;
            let description = vuln
                .descriptions
                .into_iter()
                .find(|d| d.lang == "en")
                .map(|d| d.value)
                .unwrap_or_default();

            // Parse published date for filtering
            let published = vuln
                .cna
                .as_ref()
                .and_then(|cna| cna.date_publicized.as_deref())
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

            // Date range filter
            if let (Some(s), Some(p)) = (start, published) {
                if p < s {
                    return None;
                }
            }
            if let (Some(e), Some(p)) = (end, published) {
                if p > e {
                    return None;
                }
            }

            let severity = vuln
                .cve
                .as_ref()
                .and_then(|c| c.metrics.as_ref())
                .and_then(|m| {
                    m.cvss_v3_1
                        .as_ref()
                        .or(m.cvss_v4_0.as_ref())
                        .and_then(|m| m.base_score)
                })
                .map(|score| severity_label(score));

            Some(CveEntry {
                cve_id,
                description,
                severity,
            })
        })
        .take(input.limit)
        .collect()
}

#[derive(Deserialize)]
struct CveApiResponse {
    vulnerabilities: Vec<CveVulnerability>,
}

#[derive(Deserialize)]
struct CveVulnerability {
    cve_id: Option<String>,
    descriptions: Vec<CveDescription>,
    cna: Option<CveCna>,
    cve: Option<CveMetricsWrapper>,
}

#[derive(Deserialize)]
struct CveDescription {
    lang: String,
    value: String,
}

#[derive(Deserialize)]
struct CveCna {
    date_publicized: Option<String>,
}

#[derive(Deserialize)]
struct CveMetricsWrapper {
    metrics: Option<CveMetrics>,
}

#[derive(Deserialize)]
struct CveMetrics {
    cvss_v3_1: Option<CveCvss>,
    cvss_v4_0: Option<CveCvss>,
}

#[derive(Deserialize)]
struct CveCvss {
    base_score: Option<f64>,
}

struct CveEntry {
    cve_id: String,
    description: String,
    severity: Option<String>,
}

fn severity_label(score: f64) -> String {
    if score >= 9.0 {
        "Critical".to_string()
    } else if score >= 7.0 {
        "High".to_string()
    } else if score >= 4.0 {
        "Medium".to_string()
    } else if score > 0.0 {
        "Low".to_string()
    } else {
        "None".to_string()
    }
}

fn format_cve_section(product: &str, entries: &[CveEntry]) -> String {
    if entries.is_empty() {
        return format!("{product}:\n  (no CVEs found)");
    }

    let mut lines = vec![format!("{product}:")];
    for e in entries {
        let sev = e.severity.as_deref().unwrap_or("N/A");
        lines.push(format!("  {}: {} (Severity: {})", e.cve_id, e.description, sev));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_labels() {
        assert_eq!(severity_label(9.8), "Critical");
        assert_eq!(severity_label(7.5), "High");
        assert_eq!(severity_label(5.0), "Medium");
        assert_eq!(severity_label(3.0), "Low");
        assert_eq!(severity_label(0.0), "None");
    }

    #[test]
    fn test_format_cve_section() {
        let entries = vec![CveEntry {
            cve_id: "CVE-2026-1234".to_string(),
            description: "A test vulnerability".to_string(),
            severity: Some("High".to_string()),
        }];
        let output = format_cve_section("nginx", &entries);
        assert!(output.contains("nginx:"));
        assert!(output.contains("CVE-2026-1234"));
        assert!(output.contains("High"));
    }

    #[test]
    fn test_format_cve_section_empty() {
        let output = format_cve_section("unknown", &[]);
        assert!(output.contains("no CVEs found"));
    }
}
