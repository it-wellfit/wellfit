#![forbid(unsafe_code)]

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use wellfit_core::ledger::{BatchScope, IdempotencyLedger};
use wellfit_core::release_gate::ReleaseGate;
use wellfit_mtrix::spec::{FieldKind, FieldSpec, KeyRole, MTRIX_SPEC};
use wellfit_mtrix::MtrixEngine;

struct TempDirectory(std::path::PathBuf);

impl Drop for TempDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "could not clean synthetic MTRIX directory {}: {error}",
                    self.0.display()
                );
            }
        }
    }
}

fn synthetic_value(field: &FieldSpec, row: usize) -> String {
    match field.source_name {
        "COUNTRY" | "COUNTRY_ISO" => return "USA".to_string(),
        "BARCODE" => return String::new(),
        "DOCUMENT_NUMBER" => return format!("DOC{row:09}"),
        _ => {}
    }

    match field.key_role {
        KeyRole::Vendor => "VENDOR000001".to_string(),
        KeyRole::Dist => "DIST000001".to_string(),
        KeyRole::Customer | KeyRole::CustomerSecondary => format!("CUS{row:09}"),
        KeyRole::Product | KeyRole::Batch => format!("ITEM{row:09}"),
        KeyRole::ProductBarcode => String::new(),
        KeyRole::Salesperson => format!("SP{row:09}"),
        KeyRole::None => match field.kind {
            FieldKind::Text => format!("SYN{row:09}"),
            FieldKind::Date => "20261102".to_string(),
            FieldKind::Number => "1".to_string(),
            FieldKind::Code => field
                .allowed
                .and_then(|values| values.first())
                .copied()
                .unwrap_or("SYN")
                .to_string(),
        },
    }
}

#[test]
#[ignore = "manual MTRIX contract-derived synthetic capacity benchmark"]
fn mtrix_synthetic_drop_capacity_benchmark() {
    let rows_per_file = std::env::var("MTRIX_BENCH_ROWS_PER_FILE")
        .map(|value| {
            value
                .parse::<usize>()
                .expect("MTRIX_BENCH_ROWS_PER_FILE must be an integer")
        })
        .unwrap_or(50_000);
    assert!(
        (1..=250_000).contains(&rows_per_file),
        "rows per file must be between 1 and 250,000"
    );

    let root = std::env::temp_dir().join(format!(
        "wellfit_mtrix_synthetic_{}_{}",
        std::process::id(),
        rows_per_file
    ));
    std::fs::create_dir(&root).expect("create synthetic MTRIX directory");
    let _cleanup = TempDirectory(root.clone());

    let mut total_input_bytes = 0u64;
    for file_spec in MTRIX_SPEC {
        let filename = format!("{}02112026120000000", file_spec.prefix);
        let path = root.join(filename);
        let file = std::fs::File::create(&path).expect("create synthetic MTRIX input");
        let mut writer = std::io::BufWriter::new(file);
        let header = file_spec
            .columns
            .iter()
            .map(|field| field.source_name)
            .collect::<Vec<_>>()
            .join(";");
        writeln!(writer, "{header}").expect("write synthetic MTRIX header");

        for row in 0..rows_per_file {
            let values = file_spec
                .columns
                .iter()
                .map(|field| synthetic_value(field, row))
                .collect::<Vec<_>>();
            writeln!(writer, "{}", values.join(";")).expect("write synthetic MTRIX row");
        }
        writer.flush().expect("flush synthetic MTRIX file");
        total_input_bytes += std::fs::metadata(&path)
            .expect("stat synthetic MTRIX file")
            .len();
    }

    let ledger = Arc::new(
        IdempotencyLedger::new_scoped(":memory:", BatchScope::Exclusive)
            .expect("open in-memory MTRIX ledger"),
    );
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = MtrixEngine::new(ledger, gate);
    let batch_id = format!("MTRIX-SYN-{}-{rows_per_file}", std::process::id());
    let started = Instant::now();
    let report = engine
        .process_drop(&root, &batch_id, None)
        .expect("synthetic MTRIX drop must process");
    let elapsed = started.elapsed();
    let expected_rows = rows_per_file * MTRIX_SPEC.len();

    assert_eq!(report.files.len(), MTRIX_SPEC.len());
    for file in &report.files {
        assert_eq!(file.rows_total, rows_per_file, "{}", file.prefix);
        assert_eq!(file.rows_accepted, rows_per_file, "{}", file.prefix);
        assert_eq!(file.rows_quarantined, 0, "{}", file.prefix);
    }
    assert_eq!(
        report.error_count(),
        0,
        "unexpected MTRIX errors: {:?}",
        report
            .findings
            .iter()
            .filter(|finding| finding.severity == wellfit_mtrix::Severity::Error)
            .take(10)
            .collect::<Vec<_>>()
    );

    println!(
        "mtrix_synthetic_capacity rows_per_file={rows_per_file} total_rows={expected_rows} input_bytes={total_input_bytes} accepted_rows={} quarantined_rows={} errors={} warnings={} elapsed_ms={} rows_per_second={:.2} file_sha256={}",
        report.files.iter().map(|file| file.rows_accepted).sum::<usize>(),
        report.files.iter().map(|file| file.rows_quarantined).sum::<usize>(),
        report.error_count(),
        report.warning_count(),
        elapsed.as_millis(),
        expected_rows as f64 / elapsed.as_secs_f64(),
        report.files.iter().map(|file| format!("{}:{}", file.prefix, file.sha256)).collect::<Vec<_>>().join(",")
    );
}
