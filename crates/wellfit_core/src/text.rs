//! Text processing utilities for line splitting and record parsing
//! Handles CR, LF, and CRLF line terminators for MTRIX file processing

use std::fs;
use std::io::Read;
use std::path::Path;

/// Maximum number of bytes to read from a single input file.
///
/// Prevents unbounded memory allocation from malformed or malicious input.
pub const MAX_INPUT_BYTES: usize = 100 * 1024 * 1024; // 100 MB

/// Read a file with a byte cap to prevent unbounded memory allocation.
///
/// Returns the file contents as a string, failing if the file exceeds
/// `MAX_INPUT_BYTES` or is not valid UTF-8.
pub fn read_capped(path: &Path, max_bytes: usize) -> Result<String, String> {
    let file =
        fs::File::open(path).map_err(|e| format!("cannot open file {}: {e}", path.display()))?;

    let mut buffer = Vec::with_capacity(8 * 1024);
    let bytes_read = file
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut buffer)
        .map_err(|e| format!("cannot read file {}: {e}", path.display()))?;

    if bytes_read > max_bytes {
        return Err(format!(
            "file {} exceeds the {} byte limit",
            path.display(),
            max_bytes
        ));
    }

    String::from_utf8(buffer)
        .map_err(|e| format!("file {} is not valid UTF-8: {e}", path.display()))
}

/// Splits a document into records, keeping the line number of every record.
/// Handles LF, CRLF, and CR line terminators.
pub fn split_records(content: &str) -> Vec<(usize, &str)> {
    let mut records = Vec::new();
    let mut line_number = 0usize;
    for line in split_lines_keeping_blanks(content) {
        line_number += 1;
        if line.trim().is_empty() {
            continue;
        }
        records.push((line_number, line));
    }
    records
}

/// Splits a document into lines, accepting LF, CRLF and CR terminators.
/// A leading UTF-8 byte order mark is removed from the first line.
/// Blank lines are kept, so that the caller can still report correct line numbers.
pub fn split_lines(content: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = split_lines_keeping_blanks(content)
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect();
    lines.shrink_to_fit();
    lines
}

/// Splits a document into lines, accepting LF, CRLF and CR terminators.
/// Blank lines are kept. A leading UTF-8 byte order mark is removed from the first line.
fn split_lines_keeping_blanks(content: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    let bytes = content.as_bytes();

    while start <= bytes.len() {
        // Find the next line terminator (LF, CR, or CRLF)
        let end = find_next_line_end(content, start);

        if end >= bytes.len() {
            // Last segment (possibly empty)
            let segment = if start < bytes.len() {
                &content[start..]
            } else {
                ""
            };
            if !segment.is_empty() || lines.is_empty() {
                lines.push(segment);
            }
            break;
        }

        lines.push(&content[start..end]);

        // Move past the line terminator
        start = end + line_terminator_len(content, end);

        // Handle case where content ends with a terminator
        if start > bytes.len() {
            break;
        }
    }

    // Strip BOM from first line if present
    if !lines.is_empty() {
        lines[0] = strip_bom(lines[0]);
    }

    lines
}

/// Finds the end position of the next line (excluding the terminator)
fn find_next_line_end(content: &str, start: usize) -> usize {
    let bytes = content.as_bytes();
    let mut i = start;

    while i < bytes.len() {
        match bytes[i] {
            b'\n' => return i,
            b'\r' => return i,
            _ => i += 1,
        }
    }

    i
}

/// Returns the length of the line terminator starting at the given position
fn line_terminator_len(content: &str, pos: usize) -> usize {
    let bytes = content.as_bytes();
    if bytes[pos] == b'\r' && pos + 1 < bytes.len() && bytes[pos + 1] == b'\n' {
        2 // CRLF
    } else {
        1 // CR or LF
    }
}

/// Split one record into its fields.
pub fn split_fields(line: &str, separator: char) -> Vec<&str> {
    line.split(separator).map(|field| field.trim()).collect()
}

/// Remove a leading UTF-8 byte order mark, if present.
fn strip_bom(line: &str) -> &str {
    line.strip_prefix('\u{FEFF}').unwrap_or(line)
}
