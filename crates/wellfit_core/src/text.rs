//! Text processing utilities for line splitting and record parsing
//! Handles CR, LF, and CRLF line terminators for MTRIX file processing

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Validate a delimited document without retaining its records in memory.
pub fn validate_delimited_csv(input: &str, delimiter: u8) -> Result<(), String> {
    #[derive(Clone, Copy)]
    enum FieldState {
        Start,
        Unquoted,
        Quoted,
        AfterQuote,
    }

    let bytes = input.as_bytes();
    let mut state = FieldState::Start;
    let mut index = 0usize;
    let mut line = 1usize;
    while index < bytes.len() {
        let byte = bytes[index];
        match state {
            FieldState::Start => match byte {
                b'"' => {
                    state = FieldState::Quoted;
                    index += 1;
                }
                b'\r' | b'\n' => {
                    if byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                        index += 2;
                    } else {
                        index += 1;
                    }
                    line += 1;
                }
                _ if byte == delimiter => index += 1,
                _ => {
                    state = FieldState::Unquoted;
                    index += 1;
                }
            },
            FieldState::Unquoted => match byte {
                b'"' => return Err(format!("unexpected quote in unquoted field at line {line}")),
                b'\r' | b'\n' => {
                    if byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                        index += 2;
                    } else {
                        index += 1;
                    }
                    line += 1;
                    state = FieldState::Start;
                }
                _ if byte == delimiter => {
                    index += 1;
                    state = FieldState::Start;
                }
                _ => index += 1,
            },
            FieldState::Quoted => match byte {
                b'"' if bytes.get(index + 1) == Some(&b'"') => index += 2,
                b'"' => {
                    index += 1;
                    state = FieldState::AfterQuote;
                }
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                    index += 2;
                    line += 1;
                }
                b'\r' | b'\n' => {
                    index += 1;
                    line += 1;
                }
                _ => index += 1,
            },
            FieldState::AfterQuote => match byte {
                b'\r' | b'\n' => {
                    if byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                        index += 2;
                    } else {
                        index += 1;
                    }
                    line += 1;
                    state = FieldState::Start;
                }
                _ if byte == delimiter => {
                    index += 1;
                    state = FieldState::Start;
                }
                _ => {
                    return Err(format!(
                        "unexpected character after closing quote at line {line}"
                    ))
                }
            },
        }
    }
    if matches!(state, FieldState::Quoted) {
        return Err(format!("unterminated quoted field at line {line}"));
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(input.as_bytes());
    let mut record = csv::StringRecord::new();
    loop {
        let has_record = reader.read_record(&mut record).map_err(|error| {
            let line = error
                .position()
                .map(|position| position.line().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            format!("invalid delimited input near line {line}: {error}")
        })?;
        if !has_record {
            return Ok(());
        }
    }
}

/// Serialize parsed fields as one canonical CSV record for quarantine evidence.
pub fn serialize_csv_record(record: &csv::StringRecord, delimiter: u8) -> Result<String, String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(Vec::new());
    writer
        .write_record(record.iter())
        .map_err(|error| format!("cannot serialize delimited record: {error}"))?;
    let bytes = writer
        .into_inner()
        .map_err(|error| format!("cannot finish delimited record: {}", error.error()))?;
    let mut serialized = String::from_utf8(bytes)
        .map_err(|error| format!("serialized record is not UTF-8: {error}"))?;
    if serialized.ends_with('\n') {
        serialized.pop();
    }
    Ok(serialized)
}

/// Maximum number of bytes to read from a single input file.
///
/// Prevents unbounded memory allocation from malformed or malicious input.
pub const MAX_INPUT_BYTES: usize = 100 * 1024 * 1024; // 100 MB

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Write a complete file beside its destination and publish it with a rename.
pub fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "destination must include a file name",
        )
    })?;

    let mut created = None;
    for _ in 0..32 {
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut temp_name = std::ffi::OsString::from(".");
        temp_name.push(file_name);
        temp_name.push(format!(".{}.{}.tmp", std::process::id(), sequence));
        let temp_path = parent.join(temp_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => {
                created = Some((temp_path, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (temp_path, mut file) = created.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not allocate a unique temporary file name",
        )
    })?;

    let result = (|| {
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp_path, path)
    })();

    if let Err(error) = result {
        match fs::remove_file(&temp_path) {
            Ok(()) => Err(error),
            Err(cleanup_error) if cleanup_error.kind() == std::io::ErrorKind::NotFound => {
                Err(error)
            }
            Err(cleanup_error) => Err(std::io::Error::new(
                error.kind(),
                format!(
                    "{error}; additionally failed to remove {}: {cleanup_error}",
                    temp_path.display()
                ),
            )),
        }
    } else {
        Ok(())
    }
}

/// Read a file with a byte cap to prevent unbounded memory allocation.
///
/// The returned buffer never contains more than `max_bytes` bytes.
pub fn read_bytes_capped(path: &Path, max_bytes: usize) -> Result<Vec<u8>, String> {
    let file =
        fs::File::open(path).map_err(|e| format!("cannot open file {}: {e}", path.display()))?;
    let max_bytes_u64 = u64::try_from(max_bytes)
        .ok()
        .ok_or_else(|| format!("byte limit is too large: {max_bytes}"))?;
    let read_limit = max_bytes_u64
        .checked_add(1)
        .ok_or_else(|| format!("byte limit is too large: {max_bytes}"))?;
    let file_size = file
        .metadata()
        .map_err(|e| format!("cannot inspect file {}: {e}", path.display()))?
        .len();
    if file_size > max_bytes_u64 {
        return Err(format!(
            "file {} exceeds the {max_bytes} byte limit",
            path.display()
        ));
    }

    let mut buffer = Vec::with_capacity(8 * 1024);
    // The capped read also rejects a file that grows after the metadata check.
    let bytes_read = file
        .take(read_limit)
        .read_to_end(&mut buffer)
        .map_err(|e| format!("cannot read file {}: {e}", path.display()))?;

    if bytes_read > max_bytes {
        return Err(format!(
            "file {} exceeds the {max_bytes} byte limit",
            path.display()
        ));
    }

    Ok(buffer)
}

/// Read a UTF-8 file with a byte cap to prevent unbounded memory allocation.
pub fn read_capped(path: &Path, max_bytes: usize) -> Result<String, String> {
    let buffer = read_bytes_capped(path, max_bytes)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_bytes(bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "wellfit_capped_read_{}_{}",
            std::process::id(),
            TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn capped_byte_read_accepts_exact_limit_and_preserves_binary_data() {
        let path = write_temp_bytes(&[0x00, 0xFF, 0x80]);

        let result = read_bytes_capped(&path, 3).unwrap();

        assert_eq!(result, [0x00, 0xFF, 0x80]);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn capped_byte_read_rejects_input_over_limit() {
        let path = write_temp_bytes(b"four");

        let error = read_bytes_capped(&path, 3).unwrap_err();

        assert!(error.contains("exceeds the 3 byte limit"), "{error}");
        fs::remove_file(path).unwrap();
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn capped_byte_read_rejects_limits_that_cannot_be_incremented() {
        let path = write_temp_bytes(b"");

        let error = read_bytes_capped(&path, usize::MAX).unwrap_err();

        assert!(error.contains("byte limit is too large"), "{error}");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn delimited_csv_validation_accepts_quoted_delimiters_and_embedded_newlines() {
        let input = "name;description\r\nitem;\"first line\r\nsecond line\"\r\n";
        assert!(validate_delimited_csv(input, b';').is_ok());
    }

    #[test]
    fn delimited_csv_validation_accepts_bare_carriage_return_terminators() {
        assert!(validate_delimited_csv("name,value\ritem,1\r", b',').is_ok());
    }

    #[test]
    fn delimited_csv_validation_rejects_malformed_quoting() {
        let error = validate_delimited_csv("name,value\nitem,\"unterminated", b',').unwrap_err();
        assert!(error.contains("line"), "{error}");
    }

    #[test]
    fn record_serialization_preserves_csv_field_values() {
        let record = csv::StringRecord::from(vec!["A, B", "line 1\nline 2", "quote: \""]);
        let serialized = serialize_csv_record(&record, b',').unwrap();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(serialized.as_bytes());
        let parsed = reader.records().next().unwrap().unwrap();
        assert_eq!(parsed, record);
    }

    #[test]
    fn write_atomic_replaces_complete_destination() {
        let path = std::env::temp_dir().join(format!(
            "wellfit_atomic_write_{}_{}.txt",
            std::process::id(),
            TEMP_FILE_SEQUENCE.load(Ordering::Relaxed)
        ));
        fs::write(&path, "old content").unwrap();

        write_atomic(&path, b"new complete content").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new complete content");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn write_atomic_does_not_replace_a_directory() {
        let path = std::env::temp_dir().join(format!(
            "wellfit_atomic_target_{}_{}",
            std::process::id(),
            TEMP_FILE_SEQUENCE.load(Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();

        assert!(write_atomic(&path, b"content").is_err());
        assert!(path.is_dir());
        let temp_files = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_name().to_string_lossy().starts_with(&format!(
                    ".{}.{}",
                    path.file_name().unwrap().to_string_lossy(),
                    std::process::id()
                ))
            })
            .count();
        assert_eq!(temp_files, 0);
        fs::remove_dir(path).unwrap();
    }
}
