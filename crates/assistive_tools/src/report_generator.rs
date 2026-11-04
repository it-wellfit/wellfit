#![forbid(unsafe_code)]

//! Report Generator — assisted report rendering in multiple formats.

use crate::AssistError;

/// Supported report output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Html,
    Csv,
    Json,
    Excel,
    Pdf,
}

impl ReportFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportFormat::Html => "HTML",
            ReportFormat::Csv => "CSV",
            ReportFormat::Json => "JSON",
            ReportFormat::Excel => "XLSX",
            ReportFormat::Pdf => "PDF",
        }
    }
}

/// Report specification.
#[derive(Debug, Clone)]
pub struct ReportSpec {
    pub title: String,
    pub format: ReportFormat,
    pub template: Option<String>,
}

impl ReportSpec {
    pub fn new(title: &str, format: ReportFormat) -> Self {
        Self {
            title: title.to_string(),
            format,
            template: None,
        }
    }
}

/// Render a table report in the specified format.
pub fn render_table(
    spec: &ReportSpec,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<String, AssistError> {
    match spec.format {
        ReportFormat::Html => Ok(render_html(spec, headers, rows)),
        ReportFormat::Csv => Ok(render_csv(headers, rows)),
        ReportFormat::Json => Ok(render_json(spec, headers, rows)),
        ReportFormat::Excel | ReportFormat::Pdf => Err(AssistError::NotImplemented(format!(
            "{} export requires additional dependencies (pending ADR)",
            spec.format.as_str()
        ))),
    }
}

fn render_html(spec: &ReportSpec, headers: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html lang=\"zh-Hant\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str(&format!("<title>{}</title>\n", escape_html(&spec.title)));
    out.push_str("</head>\n<body>\n");
    out.push_str(&format!("<h1>{}</h1>\n", escape_html(&spec.title)));
    out.push_str("<table>\n<thead><tr>");
    for h in headers {
        out.push_str(&format!("<th>{}</th>", escape_html(h)));
    }
    out.push_str("</tr></thead>\n<tbody>\n");
    for row in rows {
        out.push_str("<tr>");
        for cell in row {
            out.push_str(&format!("<td>{}</td>", escape_html(cell)));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody>\n</table>\n</body>\n</html>\n");
    out
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_csv(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    // Header row
    let header_line: Vec<String> = headers.iter().map(|h| csv_escape(h)).collect();
    out.push_str(&header_line.join(","));
    out.push('\n');
    // Data rows
    for row in rows {
        let line: Vec<String> = row.iter().map(|c| csv_escape(c)).collect();
        out.push_str(&line.join(","));
        out.push('\n');
    }
    out
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn render_json(spec: &ReportSpec, headers: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{{\n  \"title\": \"{}\",\n",
        json_escape(&spec.title)
    ));
    out.push_str("  \"headers\": [");
    let header_strs: Vec<String> = headers
        .iter()
        .map(|h| format!("\"{}\"", json_escape(h)))
        .collect();
    out.push_str(&header_strs.join(", "));
    out.push_str("],\n  \"rows\": [\n");
    for (i, row) in rows.iter().enumerate() {
        out.push_str("    [");
        let cell_strs: Vec<String> = row
            .iter()
            .map(|c| format!("\"{}\"", json_escape(c)))
            .collect();
        out.push_str(&cell_strs.join(", "));
        out.push(']');
        if i < rows.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_rendering() {
        let spec = ReportSpec::new("Test Report", ReportFormat::Html);
        let headers = vec!["Name".to_string(), "Value".to_string()];
        let rows = vec![vec!["Alice".to_string(), "42".to_string()]];
        let html = render_table(&spec, &headers, &rows).unwrap();
        assert!(html.contains("<h1>Test Report</h1>"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("<td>Alice</td>"));
    }

    #[test]
    fn test_html_escaping() {
        let spec = ReportSpec::new("XSS Test", ReportFormat::Html);
        let headers = vec!["<script>".to_string()];
        let rows = vec![vec!["a & b".to_string()]];
        let html = render_table(&spec, &headers, &rows).unwrap();
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("a &amp; b"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_csv_rendering() {
        let spec = ReportSpec::new("CSV Report", ReportFormat::Csv);
        let headers = vec!["Name".to_string(), "Value".to_string()];
        let rows = vec![
            vec!["Alice".to_string(), "42".to_string()],
            vec!["Bob, Jr.".to_string(), "25".to_string()],
        ];
        let csv = render_table(&spec, &headers, &rows).unwrap();
        assert!(csv.contains("Name,Value"));
        assert!(csv.contains("Alice,42"));
        assert!(csv.contains("\"Bob, Jr.\""));
    }

    #[test]
    fn test_json_rendering() {
        let spec = ReportSpec::new("JSON Report", ReportFormat::Json);
        let headers = vec!["Name".to_string()];
        let rows = vec![vec!["Alice".to_string()]];
        let json = render_table(&spec, &headers, &rows).unwrap();
        assert!(json.contains("\"title\": \"JSON Report\""));
        assert!(json.contains("\"Name\""));
        assert!(json.contains("\"Alice\""));
    }

    #[test]
    fn test_excel_not_implemented() {
        let spec = ReportSpec::new("Excel", ReportFormat::Excel);
        let headers = vec!["A".to_string()];
        let rows = vec![vec!["1".to_string()]];
        let result = render_table(&spec, &headers, &rows);
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_not_implemented() {
        let spec = ReportSpec::new("PDF", ReportFormat::Pdf);
        let headers = vec!["A".to_string()];
        let rows = vec![vec!["1".to_string()]];
        let result = render_table(&spec, &headers, &rows);
        assert!(result.is_err());
    }
}
