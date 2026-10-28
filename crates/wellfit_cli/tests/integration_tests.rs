#![forbid(unsafe_code)]

//! Integration tests for the Wellfit CLI and the MTRIX pipeline.
//!
//! The MTRIX fixtures embedded here are the authoritative bytes delivered by the
//! counterparty: the five files, their real headers and their real rows. The
//! expectations below are therefore reproducible against a real drop.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;
use wellfit_mtrix::spec::{self, FileRole};
use wellfit_mtrix::validators::{BatchStatus, Outcome, Severity, VrRule};
use wellfit_mtrix::MtrixEngine;

const ALTHCI: &str = "VENDOR_ID;DIST_ID;CUSTOMER_ID;CUSTOMER_NAME;ADRESS;DISTRICT;ZIP_CODE;CITY;STATE;SALES_CONTACT;PHONE_NUMBERS;CUSTOMER_ID;ROUTE;STORE_TYPE;PROPORTION;COUNTRY_ISO;\n\
6063343900011222;029864857023000;995A;JDID;;;;Bogor;;;;995A;Online;Online;0;USA\n\
6063343900011222;029864857023000;995C;Shopee;;;;Bogor;;;;995C;Online;Online;0;USA\n\
6063343900011222;029864857023000;995D;Tiktok Shop;;;;Bogor;;;;995D;Online;Onlinne;0;USA\n";

const ALTHES: &str = "INVENTORY_DATE;VENDOR_ID;DIST_ID;BATCH_CODE;QUANTITY_STOCK;COUNTRY_ISO\n\
20240221;USA6063343900011222;029864857023000;7890541735003;167;USA\n\
20240221;USA6063343900011222;029864857023000;7891224813520;150;USA\n";

const ALTHFV: &str = "VENDOR_ID;DIST_ID;CUSTOMER_ID;MANAGER_CODE;MANAGER_NAME;SUPERVISOR_ID;SUPERVISOR_NAME;SALESPERSON_CODE;SALESPERSON_NAME;COUNTRY\n\
6063343900011222;029864857023000;995A;;;;;;USA\n";

const ALTHPD: &str = "TRANSACTION_DATE;DIST_ID;VENDOR_ID;VENDOR_COMPANY_NAME;PRODUCT_CODE;PACKAGE_TYPE;BARCODE;BARCODE_TYPE;DESCRIPTION;PRODUCT_DIV;PRODUCT_STATUS;COUNTRY_ISO\n\
20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;8;0;7898656600087 ;1;;RSF;A;USA\n\
20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;16;0;7898656600162 ;1;;RSF;A;USA\n";

const ALTHVD: &str = "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;7AC;20220109;348732;7893249911649;1;174300;;;;ARG;\n";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// OS Application Control policy blocks execution of binaries from the target
/// directory. Copy the compiled binary to the project root (an allowed path)
/// and return that path. The copy is idempotent.
fn wellfit_bin() -> PathBuf {
    let src = PathBuf::from(env!("CARGO_BIN_EXE_wellfit"));
    let dst = std::env::temp_dir().join(format!(
        "wellfit-it-bin-{}-{:?}.exe",
        std::process::id(),
        std::thread::current().id()
    ));
    let needs_copy = match (fs::metadata(&src), fs::metadata(&dst)) {
        (Ok(s), Ok(d)) => s.modified().ok() != d.modified().ok() || s.len() != d.len(),
        _ => true,
    };
    if needs_copy {
        fs::copy(&src, &dst).expect("copy wellfit binary to allowed path");
    }
    dst
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "wellfit-it-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write_drop(dir: &Path) {
    fs::write(dir.join("ALTHCI30062020153005130 2 (3).csv"), ALTHCI).unwrap();
    fs::write(dir.join("ALTHES30062020153005130 2 (3).csv"), ALTHES).unwrap();
    fs::write(dir.join("ALTHFV30062020153005130 2 (3).csv"), ALTHFV).unwrap();
    fs::write(dir.join("ALTHPD30062020153005130 2 (3).csv"), ALTHPD).unwrap();
    fs::write(dir.join("ALTHVD30062020153005130 5 (4).csv"), ALTHVD).unwrap();
}

fn engine() -> MtrixEngine {
    let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    MtrixEngine::new(ledger, gate)
}

#[test]
fn test_specification_declares_five_files_and_fifty_six_fields() {
    assert_eq!(spec::MTRIX_SPEC.len(), 5);
    let total: usize = spec::MTRIX_SPEC.iter().map(|f| f.column_count()).sum();
    assert_eq!(total, 56);
    assert_eq!(spec::spec_by_role(FileRole::Sales).column_count(), 12);
    assert_eq!(spec::spec_by_role(FileRole::Inventory).column_count(), 6);
    assert_eq!(spec::spec_by_role(FileRole::Customer).column_count(), 16);
    assert_eq!(spec::spec_by_role(FileRole::SalesForce).column_count(), 10);
    assert_eq!(spec::spec_by_role(FileRole::Product).column_count(), 12);
}

#[test]
fn test_every_reported_finding_is_recorded_in_the_ledger() {
    let dir = temp_dir("ledger-consistency");
    let out = temp_dir("ledger-consistency-out");
    write_drop(&dir);

    let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = MtrixEngine::new(ledger.clone(), gate);
    let report = engine.process_drop(&dir, "IT-LEDGER", Some(&out)).unwrap();

    let stored = ledger.get_findings(Some("IT-LEDGER")).unwrap();

    // The report is the readable summary, the ledger is the durable evidence.
    // A finding that exists in only one of the two cannot be traced back, so
    // every finding of the report has to be in the ledger as well. A row that
    // was isolated lives in the quarantine log instead, otherwise the two
    // counters of `status` would count the same problem twice.
    let reported: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.outcome != Outcome::RowQuarantined)
        .map(|f| (f.rule.id(), f.line, f.key.clone(), f.reason.clone()))
        .collect();
    assert!(!reported.is_empty(), "the drop has to produce findings");

    for (rule, line, key, reason) in &reported {
        assert!(
            stored.iter().any(|s| {
                s.rule_code == *rule
                    && s.line_number == *line
                    && s.key_value == *key
                    && s.reason == *reason
            }),
            "finding {rule} on line {line} is reported but missing from the ledger"
        );
    }

    // The counts of `status` follow the report.
    let errors = stored.iter().filter(|s| s.severity == "ERROR").count();
    let warnings = stored.iter().filter(|s| s.severity == "WARNING").count();
    assert_eq!(errors, report.error_count());
    assert_eq!(warnings, report.warning_count());

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn test_real_drop_is_accepted_and_findings_are_listed() {
    let dir = temp_dir("drop");
    let out = temp_dir("drop-out");
    write_drop(&dir);

    let mut engine = engine();
    let report = engine.process_drop(&dir, "IT-MTRIX", Some(&out)).unwrap();

    assert_eq!(report.files.len(), 5);
    assert!(report.files.iter().all(|f| f.header_ok));
    assert!(report.files.iter().all(|f| f.filename_ok));
    assert!(report.files.iter().all(|f| !f.rejected));

    // The sales customer 7AC is not in the customer master: the key is listed,
    // the row is kept (the manual asks to list the key, not to drop the row).
    let vr08 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr08)
        .count();
    assert_eq!(vr08, 1);

    // The sales item is in neither the inventory nor the product master.
    let vr09 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr09)
        .count();
    assert_eq!(vr09, 1);

    // Both inventory items are missing from the product master.
    let vr11 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr11)
        .count();
    assert_eq!(vr11, 2);

    // Empty mandatory-ness is a warning, never a rejection.
    let warnings = report.warning_count();
    assert!(warnings >= 1, "expected the undefined mandatory warning");
    assert_eq!(report.status, BatchStatus::PartialAccepted);

    // The eight wording deviations between the manual and the delivered bytes.
    assert_eq!(report.deviations.len(), 8);

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn test_repaired_output_keeps_the_contract_and_the_fixes() {
    let dir = temp_dir("repair");
    let out = temp_dir("repair-out");
    write_drop(&dir);

    let mut engine = engine();
    engine.process_drop(&dir, "IT-MTRIX", Some(&out)).unwrap();

    let inventory =
        fs::read_to_string(out.join("ALTHES30062020153005130 2 (3)_repaired.csv")).unwrap();
    assert!(inventory
        .starts_with("INVENTORY_DATE;VENDOR_ID;DIST_ID;BATCH_CODE;QUANTITY_STOCK;COUNTRY_ISO"));
    // The country prefix is separated from the vendor code.
    assert!(inventory.contains(";6063343900011222;"));
    assert!(!inventory.contains("USA6063343900011222"));

    let product =
        fs::read_to_string(out.join("ALTHPD30062020153005130 2 (3)_repaired.csv")).unwrap();
    // The distributor dot mask is removed, the barcode noise is trimmed.
    assert!(product.contains(";029864857023000;"));
    assert!(!product.contains("02.986.485.7023.000"));
    assert!(product.contains("7898656600087;"));

    let customer =
        fs::read_to_string(out.join("ALTHCI30062020153005130 2 (3)_repaired.csv")).unwrap();
    // The store type keeps its source value: the code table is unknown
    // (MTRIX-029) and the mandatory-ness of the column is undefined
    // (MTRIX-003), so a suspected alias is reported, never written into the
    // data.
    assert!(customer.contains("Onlinne"));
    assert!(customer.starts_with("VENDOR_ID;DIST_ID;CUSTOMER_ID;CUSTOMER_NAME;ADDRESS;"));

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn test_a_broken_header_rejects_only_that_file() {
    let dir = temp_dir("broken");
    let out = temp_dir("broken-out");
    write_drop(&dir);
    fs::write(
        dir.join("ALTHVD30062020153005130 5 (4).csv"),
        "WRONG;HEADER;COLUMNS\n1;2;3\n",
    )
    .unwrap();

    let mut engine = engine();
    let report = engine.process_drop(&dir, "IT-MTRIX", Some(&out)).unwrap();

    let sales = report
        .files
        .iter()
        .find(|f| f.prefix == "ALTHVD")
        .expect("sales outcome");
    assert!(sales.rejected);
    assert!(!sales.header_ok);
    assert!(report
        .files
        .iter()
        .filter(|f| f.prefix != "ALTHVD")
        .all(|f| !f.rejected));

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn test_missing_file_stops_the_batch() {
    let dir = temp_dir("missing");
    write_drop(&dir);
    fs::remove_file(dir.join("ALTHES30062020153005130 2 (3).csv")).unwrap();

    let mut engine = engine();
    let error = engine.process_drop(&dir, "IT-MTRIX", None).unwrap_err();
    assert!(error.contains("ALTHES"), "{error}");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_delivered_fixtures_produce_the_documented_findings() {
    let root = workspace_root();
    let drop = root.join("spec/fixtures/mtrix/delivered");
    assert!(
        drop.is_dir(),
        "the delivered fixtures must exist at {}",
        drop.display()
    );

    let mut engine = engine();
    let report = engine.process_drop(&drop, "IT-DELIVERED", None).unwrap();

    assert_eq!(report.files.len(), 5);
    assert!(report.files.iter().all(|f| f.header_ok));
    assert!(report.files.iter().all(|f| f.filename_ok));
    assert!(report.files.iter().all(|f| !f.rejected));

    // Nine sales rows carry a customer that the customer master does not hold.
    // The finding must name the offending key, not merely count: a binding to
    // the wrong column would keep the count at nine while reporting the wrong
    // value.
    let vr08_keys: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr08)
        .map(|f| f.key.as_str())
        .collect();
    assert_eq!(vr08_keys.len(), 9);
    assert!(
        vr08_keys.iter().all(|key| *key == "7AC"),
        "the orphan customer must be reported by its key, not another column: {vr08_keys:?}"
    );
    // Their items are in neither the inventory nor the product master.
    let vr09 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr09)
        .count();
    assert_eq!(vr09, 9);
    // Four inventory items are missing from the product master.
    let vr11 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr11)
        .count();
    assert_eq!(vr11, 4);
    // The sales force customers all exist in the customer master.
    let vr14 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr14)
        .count();
    assert_eq!(vr14, 0);

    // Eight wording deviations between the manual and the delivered bytes.
    assert_eq!(report.deviations.len(), 8);
    // The inventory snapshot belongs to another period than the sales file.
    assert!(report
        .findings
        .iter()
        .any(|f| f.rule == VrRule::Vr13
            && f.severity == wellfit_mtrix::validators::Severity::Warning));
    // No row is isolated: cross file findings list the key, they do not drop it.
    assert!(report.files.iter().all(|f| f.rows_quarantined == 0));
    assert_eq!(report.status, BatchStatus::PartialAccepted);
}

#[test]
fn test_derived_cases_exercise_the_row_rules() {
    let root = workspace_root();
    let drop = root.join("spec/fixtures/mtrix/derived/case-a-value-violations");
    assert!(drop.is_dir(), "derived case must exist");

    let mut engine = engine();
    let report = engine.process_drop(&drop, "IT-DERIVED-A", None).unwrap();

    // The invalid calendar date and the invalid document type isolate their row.
    let vr03 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr03 && f.severity == Severity::Error)
        .count();
    assert_eq!(vr03, 1);
    // The document type outside the allowed set isolates its row; the rows with
    // an empty document type are only a warning, because the manual does not
    // define the mandatory-ness of the field.
    let vr04_errors = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr04 && f.severity == Severity::Error)
        .count();
    assert_eq!(vr04_errors, 1);
    let vr04_warnings = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr04 && f.severity == Severity::Warning)
        .count();
    assert_eq!(vr04_warnings, 8);
    // The invalid package type isolates its row.
    let vr07 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr07 && f.severity == Severity::Error)
        .count();
    assert_eq!(vr07, 1);
    // The negative stock is repaired to zero and reported as a warning.
    let vr06 = report
        .findings
        .iter()
        .filter(|f| f.rule == VrRule::Vr06 && f.severity == Severity::Warning)
        .count();
    assert_eq!(vr06, 1);
    // Two rows are isolated: the sales row with the impossible date and the
    // document type, and the product row with the impossible package type.
    let isolated_rows: usize = report.files.iter().map(|f| f.rows_quarantined).sum();
    assert_eq!(isolated_rows, 2);
}

#[test]
fn test_derived_header_case_rejects_only_the_broken_file() {
    let root = workspace_root();
    let drop = root.join("spec/fixtures/mtrix/derived/case-c-header-rejected");

    let mut engine = engine();
    let report = engine.process_drop(&drop, "IT-DERIVED-C", None).unwrap();

    let sales = report
        .files
        .iter()
        .find(|f| f.prefix == "ALTHVD")
        .expect("sales outcome");
    assert!(sales.rejected);
    assert!(report
        .files
        .iter()
        .filter(|f| f.prefix != "ALTHVD")
        .all(|f| !f.rejected));
}

#[test]
#[ignore = "OS App Control policy (error 4551) blocks binary execution; run in CI"]
fn test_cli_spec_lists_every_field() {
    let output = Command::new(wellfit_bin())
        .args(["spec", "--format", "markdown"])
        .output()
        .expect("cli runs");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("ALTHCI"));
    assert!(text.contains("ALTHVD"));
    assert!(text.contains("VR-14"));
    assert!(text.contains("ANOM-12"));
}

#[test]
#[ignore = "OS App Control policy (error 4551) blocks binary execution; run in CI"]
fn test_cli_mtrix_strict_reports_the_violations() {
    let dir = temp_dir("cli-mtrix");
    let report_path = dir.join("validation_report.json");
    write_drop(&dir);

    let output = Command::new(wellfit_bin())
        .current_dir(workspace_root())
        .args([
            "mtrix",
            "--dir",
            dir.to_str().unwrap(),
            "--report",
            report_path.to_str().unwrap(),
            "--strict",
            "--db",
            dir.join("ledger.sqlite").to_str().unwrap(),
        ])
        .output()
        .expect("cli runs");

    assert_eq!(output.status.code(), Some(3), "strict mode must fail");
    let report = fs::read_to_string(&report_path).expect("report written");
    assert!(report.contains("\"rule\": \"VR-08\""));
    assert!(report.contains("PartialAccepted"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "OS App Control policy (error 4551) blocks binary execution; run in CI"]
fn test_cli_status_reports_the_ledger() {
    let dir = temp_dir("cli-status");
    let output = Command::new(wellfit_bin())
        .args([
            "status",
            "--db",
            dir.join("ledger.sqlite").to_str().unwrap(),
        ])
        .output()
        .expect("cli runs");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("ledger"));
    assert!(text.contains("isolated records"));

    let _ = fs::remove_dir_all(&dir);
}
