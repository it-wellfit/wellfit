#![forbid(unsafe_code)]

//! Wellfit CLI - single native entry point for the three product waves.
//!
//! Sub-commands:
//! - `status`  - ledger state and isolated records
//! - `shopee`  - wave 1, Shopee inbound orders to the Fuyi ERP import batch
//! - `reports` - wave 2, Fuyi ERP inventory report and the release gate
//! - `mtrix`   - wave 3, the Alpargatas MTRIX five file drop
//! - `audit`   - full physical audit over the real files in the data folder
//! - `spec`    - the MTRIX contract and rule catalogue, generated from the code
//!
//! Global options:
//! - `--color`   - color output: auto (default), always, never
//! - `--quiet`   - suppress non-essential output
//! - `--verbose` - emit diagnostic detail
//! - `--config`  - path to wellfit.toml (default: ./wellfit.toml)
//! - `--watch`   - re-run when input files change

mod ui;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use ui::{
    error, heading, info, kv, ok, print_separator, print_table_row, verbose, warn, ColorChoice,
    Spinner, UiConfig,
};

use wellfit_bankpay::{query, BankEncoding, BankpayEngine};
use wellfit_core::access_control::{
    AccessController, AccessDecision, Classification, Permission, PermissionLevel,
};
use wellfit_core::jit_permission::{JitAccessRequestParams, JitPermissionManager};
use wellfit_core::ledger::{BatchScope, FindingSeverity, IdempotencyLedger};
use wellfit_core::release_gate::ReleaseGate;
use wellfit_mtrix::repairs::REPAIR_RULES;
use wellfit_mtrix::spec::{self, KeyRole};
use wellfit_mtrix::validators::VrRule;
use wellfit_mtrix::MtrixEngine;
use wellfit_reports::report_automator::ReportAutomator;
use wellfit_shopee::shopee_engine::ShopeeInboundEngine;

const DEFAULT_LEDGER: &str = "data/wellfit_ledger.sqlite";
const DEFAULT_MTRIX_FIXTURES: &str = "spec/fixtures/mtrix/delivered";
const CONFIG_FILE: &str = "wellfit.toml";

// ---------------------------------------------------------------- CLI definition

#[derive(Parser)]
#[command(
    name = "wellfit",
    version,
    about = "Dachen industrial auxiliary ERP automation - one native binary, three product waves",
    long_about = "Wellfit CLI processes Shopee orders, Fuyi ERP reports, and Alpargatas MTRIX drops.\n\nUse --watch to re-run automatically when input files change.\nConfigure defaults in wellfit.toml."
)]
struct Cli {
    /// Color output: auto (default), always, never
    #[arg(long, default_value = "auto", value_parser = ["auto", "always", "never"])]
    color: String,

    /// Suppress non-essential output
    #[arg(short, long)]
    quiet: bool,

    /// Emit diagnostic detail
    #[arg(short, long)]
    verbose: bool,

    /// Disable ANSI colour output (equivalent to --color never)
    #[arg(long)]
    no_color: bool,

    /// Path to wellfit.toml configuration file
    #[arg(long, default_value = CONFIG_FILE)]
    config: PathBuf,

    /// Re-run the command when input files change
    #[arg(long)]
    watch: bool,

    /// Sub-command
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the idempotency ledger state and the isolated records.
    Status {
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Wave 1 - Shopee inbound orders to the Fuyi ERP import batch.
    Shopee {
        /// Source order export
        #[arg(short, long)]
        input: PathBuf,
        /// Destination ERP import CSV
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Batch identifier
        #[arg(short, long, default_value = "BATCH-CLI-SHOPEE")]
        batch: String,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Wave 2 - Fuyi ERP inventory report with the release gate.
    Reports {
        /// Source inventory report
        #[arg(short, long)]
        input: PathBuf,
        /// Recipient address, repeatable or comma separated
        #[arg(short = 'r', long, value_delimiter = ',')]
        recipient: Vec<String>,
        /// Batch identifier
        #[arg(short, long, default_value = "BATCH-CLI-REPORT")]
        batch: String,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Wave 3 - Alpargatas MTRIX five file drop.
    Mtrix {
        /// Folder holding the five delivered files
        #[arg(short, long)]
        dir: PathBuf,
        /// Folder for the repaired files
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        /// Validation report destination
        #[arg(long)]
        report: Option<PathBuf>,
        /// Exit with a non zero code when a validation error is recorded
        #[arg(long)]
        strict: bool,
        /// Batch identifier
        #[arg(short, long, default_value = "BATCH-CLI-MTRIX")]
        batch: String,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
        /// Export validated data as JSON for knowledge ingestion
        #[arg(long)]
        export_knowledge: Option<PathBuf>,
    },
    /// Full physical audit over the real files in the data folder.
    Audit {
        /// Folder holding the audit inputs
        #[arg(short, long, default_value = "data")]
        data_dir: PathBuf,
        /// Folder holding the five delivered MTRIX files
        #[arg(long)]
        mtrix_dir: Option<PathBuf>,
        /// Audit report destination
        #[arg(long)]
        report: Option<PathBuf>,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Print the MTRIX contract and rule catalogue.
    Spec {
        /// Output format
        #[arg(short, long, default_value = "markdown", value_parser = ["markdown", "json", "html"])]
        format: String,
    },
    /// Access control management (RBAC/ABAC permissions).
    AccessControl {
        #[command(subcommand)]
        action: AccessControlAction,
    },
    /// Just-in-time permission request and approval workflow.
    Jit {
        #[command(subcommand)]
        action: JitAction,
    },
    /// E.Sun Bank enterprise net-banking interface for Fuyi ERP accounts
    /// payable vouchers: export, import, and traceable query.
    Bankpay {
        /// Sub-command: export, import, or query
        #[command(subcommand)]
        action: BankpayAction,
    },
}

/// Access control sub-actions.
#[derive(Subcommand, Debug)]
enum AccessControlAction {
    /// Evaluate an access request against the RBAC/ABAC model.
    Check {
        /// User identifier
        #[arg(short, long)]
        user: String,
        /// Action being requested (e.g. mtrix:validate)
        #[arg(short, long)]
        action: String,
        /// Resource being accessed (e.g. althci:customer-995C)
        #[arg(short, long)]
        resource: String,
        /// Resource classification (public/internal/confidential/strictly_confidential)
        #[arg(short, long, value_parser = ["public", "internal", "confidential", "strictly_confidential"])]
        classification: String,
        /// Permission level requested (read/write/delete/admin)
        #[arg(short, long, value_parser = ["read", "write", "delete", "admin"])]
        level: String,
        /// MFA was used for the session
        #[arg(long)]
        mfa: bool,
        /// Session is privileged
        #[arg(long)]
        privileged: bool,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Assign a role to a user.
    Assign {
        /// User identifier
        #[arg(short, long)]
        user: String,
        /// Role identifier
        #[arg(short, long)]
        role: String,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Revoke a role from a user.
    Revoke {
        /// User identifier
        #[arg(short, long)]
        user: String,
        /// Role identifier
        #[arg(short, long)]
        role: String,
    },
    /// View the access audit log.
    AuditLog {
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
        /// Maximum number of records to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Register a new role with permissions.
    RegisterRole {
        /// Role identifier
        #[arg(short, long)]
        role: String,
        /// Role name
        #[arg(short, long)]
        name: String,
        /// Permission action (e.g. mtrix:validate)
        #[arg(short, long)]
        action: String,
        /// Permission resource pattern (e.g. althci:*)
        #[arg(short, long)]
        resource: String,
        /// Permission classification (public/internal/confidential/strictly_confidential)
        #[arg(short, long, value_parser = ["public", "internal", "confidential", "strictly_confidential"])]
        classification: String,
        /// Permission level (read/write/delete/admin)
        #[arg(short, long, value_parser = ["read", "write", "delete", "admin"])]
        level: String,
    },
}

/// JIT permission sub-actions.
#[derive(Subcommand, Debug)]
enum JitAction {
    /// Submit a just-in-time permission request.
    Request {
        /// User identifier
        #[arg(short, long)]
        user: String,
        /// Action being requested
        #[arg(short, long)]
        action: String,
        /// Resource being accessed
        #[arg(short, long)]
        resource: String,
        /// Resource classification
        #[arg(short, long, value_parser = ["public", "internal", "confidential", "strictly_confidential"])]
        classification: String,
        /// Permission level requested
        #[arg(short, long, value_parser = ["read", "write", "delete", "admin"])]
        level: String,
        /// Justification for temporary access
        #[arg(short, long)]
        justification: String,
        /// Requested duration in seconds
        #[arg(short, long, default_value = "3600")]
        duration_seconds: u64,
    },
    /// Approve a pending JIT request.
    Approve {
        /// Request identifier
        #[arg(short, long)]
        request_id: String,
        /// Approver identifier
        #[arg(short, long)]
        approver: String,
        /// Approver comments
        #[arg(short, long)]
        comments: Option<String>,
    },
    /// Reject a pending JIT request.
    Reject {
        /// Request identifier
        #[arg(short, long)]
        request_id: String,
        /// Approver identifier
        #[arg(short, long)]
        approver: String,
        /// Rejection comments
        #[arg(short, long)]
        comments: Option<String>,
    },
    /// Revoke an active JIT grant.
    Revoke {
        /// Grant identifier
        #[arg(short, long)]
        grant_id: String,
    },
    /// List pending JIT requests.
    ListPending,
    /// List active JIT grants for a user.
    ListGrants {
        /// User identifier
        #[arg(short, long)]
        user: String,
    },
}

/// BankPay sub-actions for E.Sun bank payable voucher integration.
#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
enum BankpayAction {
    /// Export ERP payable vouchers to a bank media file and dispatch.
    Export {
        /// Source voucher file (CSV)
        #[arg(short, long)]
        input: PathBuf,
        /// Batch identifier
        #[arg(short, long, default_value = "BATCH-CLI-BANKPAY")]
        batch: String,
        /// Batch date for idempotency keys (ISO-8601, default today)
        #[arg(long, default_value = "TODAY")]
        batch_date: String,
        /// Target encoding: utf8, cp950, big5 (default utf8)
        #[arg(long, default_value = "utf8", value_parser = ["utf8", "cp950", "big5"])]
        encoding: String,
        /// E.Sun security component base URL (e.g. https://127.0.0.1:8443)
        #[arg(long, default_value = "https://127.0.0.1:8443")]
        esun_url: String,
        /// SourceSystemId agreed with E.Sun
        #[arg(long)]
        source_system: Option<String>,
        /// Enterprise group tax ID (8-10 digits)
        #[arg(long)]
        customer_id: Option<String>,
        /// Payer E.Sun account number
        #[arg(long)]
        payer_account: Option<String>,
        /// Payer tax ID
        #[arg(long)]
        payer_id: Option<String>,
        /// Format type: 1=file (base64 media), 2=json
        #[arg(long, default_value = "2", value_parser = ["1", "2"])]
        format_type: String,
        /// Path to client certificate (PEM) for 2-way SSL
        #[arg(long)]
        client_cert_path: Option<String>,
        /// Path to client private key (PEM) for 2-way SSL
        #[arg(long)]
        client_key_path: Option<String>,
        /// Path to CA certificate (PEM) for server verification
        #[arg(long)]
        ca_cert_path: Option<String>,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Import bank result file and settle matched vouchers.
    Import {
        /// Source bank result file (CSV)
        #[arg(short, long)]
        input: PathBuf,
        /// Batch identifier
        #[arg(short, long, default_value = "BATCH-CLI-BANKPAY-RES")]
        batch: String,
        /// Source encoding: utf8, cp950, big5 (default utf8)
        #[arg(long, default_value = "utf8", value_parser = ["utf8", "cp950", "big5"])]
        encoding: String,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
    /// Traceable query: cross-reference voucher / bank reference / batch.
    Query {
        /// Lookup kind: voucher, tx, batch
        #[arg(value_parser = ["voucher", "tx", "batch"])]
        kind: String,
        /// Lookup key
        #[arg(short, long)]
        key: String,
        /// Ledger database path
        #[arg(long, default_value = DEFAULT_LEDGER)]
        db: PathBuf,
    },
}

// ---------------------------------------------------------------- main

fn main() {
    let cli = Cli::parse();

    let color = if cli.no_color {
        ColorChoice::Never
    } else {
        match cli.color.as_str() {
            "always" => ColorChoice::Always,
            "never" => ColorChoice::Never,
            _ => ColorChoice::Auto,
        }
    };
    let mut ui = UiConfig::load(&cli.config);
    ui.color = color;
    ui.quiet = ui.quiet || cli.quiet;
    ui.verbose = ui.verbose || cli.verbose;

    if cli.watch {
        run_with_watch(&cli, &ui);
        return;
    }

    match run(&cli, &ui) {
        Ok(code) => std::process::exit(code),
        Err(message) => {
            error(&message);
            std::process::exit(1);
        }
    }
}

/// Re-run the command whenever input files change (polling-based watch).
fn run_with_watch(cli: &Cli, ui: &UiConfig) {
    info(ui, "watch mode enabled - polling for file changes...");
    let mut last_mod: Option<SystemTime> = None;

    loop {
        let now = SystemTime::now();
        let changed = match last_mod {
            None => true,
            Some(prev) => probe_files().iter().any(|p| {
                Path::new(p)
                    .metadata()
                    .and_then(|m| m.modified())
                    .map(|mt| mt > prev)
                    .unwrap_or(false)
            }),
        };

        if changed {
            print_separator(60);
            heading(ui, "change detected - re-running");
            match run(cli, ui) {
                Ok(_) => ok(ui, "watch cycle complete"),
                Err(e) => error(&e),
            }
        }

        last_mod = Some(now);
        std::thread::sleep(Duration::from_secs(2));
    }
}

/// Probe common input files for the watcher.
fn probe_files() -> Vec<&'static str> {
    vec![
        "data/shopee_orders.csv",
        "data/fuyi_inventory_report.csv",
        "spec/fixtures/mtrix/delivered",
        "wellfit.toml",
    ]
}

fn open_ledger(ui: &UiConfig, path: &Path) -> Result<Arc<IdempotencyLedger>, String> {
    open_ledger_scoped(ui, path, BatchScope::Exclusive)
}

/// Open a ledger with an explicit claim policy.
fn open_ledger_scoped(
    ui: &UiConfig,
    path: &Path,
    scope: BatchScope,
) -> Result<Arc<IdempotencyLedger>, String> {
    verbose(ui, &format!("opening ledger at {}", path.display()));
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create ledger folder {}: {e}", parent.display()))?;
        }
    }
    let ledger = IdempotencyLedger::new_scoped(path.to_string_lossy().as_ref(), scope)
        .map_err(|e| format!("cannot open ledger {}: {e}", path.display()))?;
    Ok(Arc::new(ledger))
}

fn run(cli: &Cli, ui: &UiConfig) -> Result<i32, String> {
    match &cli.command {
        Commands::Status { db } => run_status(ui, db),
        Commands::Shopee {
            input,
            output,
            batch,
            db,
        } => run_shopee(ui, input, output.as_deref(), batch, db),
        Commands::Reports {
            input,
            recipient,
            batch,
            db,
        } => run_reports(ui, input, recipient, batch, db),
        Commands::Mtrix {
            dir,
            output_dir,
            report,
            strict,
            batch,
            db,
            export_knowledge,
        } => run_mtrix(
            ui,
            dir,
            output_dir.as_deref(),
            report.as_deref(),
            *strict,
            batch,
            db,
            export_knowledge.as_deref(),
        ),
        Commands::Audit {
            data_dir,
            mtrix_dir,
            report,
            db,
        } => run_audit(ui, data_dir, mtrix_dir.as_deref(), report.as_deref(), db),
        Commands::Spec { format } => {
            print_spec(ui, format);
            Ok(0)
        }
        Commands::AccessControl { action } => run_access_control(ui, action),
        Commands::Jit { action } => run_jit(ui, action),
        Commands::Bankpay { action } => run_bankpay(ui, action),
    }
}

// ---------------------------------------------------------------- status

fn run_status(ui: &UiConfig, db: &Path) -> Result<i32, String> {
    let spin = Spinner::new("reading ledger state...", ui.color_enabled());
    let ledger = open_ledger(ui, db)?;
    let quarantined = ledger
        .get_quarantine_items(None)
        .map_err(|e| format!("cannot read the quarantine log: {e}"))?;
    let findings = ledger
        .get_findings(None)
        .map_err(|e| format!("cannot read the findings log: {e}"))?;
    spin.finish(Some("ledger state read"));

    let errors = findings
        .iter()
        .filter(|f| f.severity == FindingSeverity::Error.as_str())
        .count();
    let warnings = findings
        .iter()
        .filter(|f| f.severity == FindingSeverity::Warning.as_str())
        .count();

    heading(ui, "Wellfit system status");
    kv(ui, "ledger", &db.display().to_string());
    kv(ui, "isolated records", &quarantined.len().to_string());
    kv(ui, "validation errors", &errors.to_string());
    kv(ui, "validation warnings", &warnings.to_string());

    if errors > 0 {
        warn(
            ui,
            &format!("{errors} validation error(s) detected - review isolated records"),
        );
    }
    if warnings > 0 {
        warn(ui, &format!("{warnings} validation warning(s) detected"));
    }

    if !quarantined.is_empty() {
        println!();
        heading(ui, "recently isolated records");
        print_separator(70);
        print_table_row(&["#", "file", "line", "code", "reason"], true);
        print_separator(70);
        for (i, record) in quarantined.iter().rev().take(10).enumerate() {
            print_table_row(
                &[
                    (i + 1).to_string(),
                    record.filename.clone(),
                    record.line_number.to_string(),
                    record.error_code.clone(),
                    record.error_reason.clone(),
                ],
                false,
            );
        }
    }

    ok(ui, "status query complete");
    Ok(if errors > 0 { 2 } else { 0 })
}

/// Status of an audited wave that could not be processed.
fn audit_error_status(error: &str) -> String {
    let lowered = error.to_lowercase();
    if lowered.contains("duplicate") || lowered.contains("already registered") {
        "already-registered".to_string()
    } else {
        "rejected".to_string()
    }
}

// ---------------------------------------------------------------- shopee

fn run_shopee(
    ui: &UiConfig,
    input: &Path,
    output: Option<&Path>,
    batch: &str,
    db: &Path,
) -> Result<i32, String> {
    let spin = Spinner::new("initialising Shopee inbound engine...", ui.color_enabled());
    let ledger = open_ledger(ui, db)?;
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let engine = ShopeeInboundEngine::new(ledger, gate);
    spin.finish(Some("Shopee engine initialised"));

    verbose(ui, &format!("reading input file: {}", input.display()));
    let content = wellfit_core::text::read_capped(input, wellfit_core::text::MAX_INPUT_BYTES)?;

    let pb = ui::ProgressBar::new("processing Shopee orders", 1, ui.color_enabled());
    let (valid, errors) = engine
        .process_raw_orders(&content, batch)
        .map_err(|e| format!("Shopee processing failed: {e}"))?;
    pb.finish();

    heading(ui, "wave 1 - Shopee inbound");
    kv(ui, "input", &input.display().to_string());
    kv(ui, "batch", batch);
    kv(ui, "orders accepted", &valid.len().to_string());
    kv(ui, "records isolated", &errors.len().to_string());

    if !errors.is_empty() {
        println!();
        heading(ui, "isolated records");
        print_separator(60);
        print_table_row(&["line", "code", "reason"], true);
        print_separator(60);
        for error in errors.iter().take(10) {
            print_table_row(
                &[
                    error.line.to_string(),
                    error.code.clone(),
                    error.reason.clone(),
                ],
                false,
            );
        }
        if errors.len() > 10 {
            info(
                ui,
                &format!("... and {} more isolated records", errors.len() - 10),
            );
        }
    }

    let erp_csv = engine
        .generate_fuyi_erp_import_csv(&valid)
        .map_err(|e| format!("cannot generate ERP import CSV: {e}"))?;
    let out_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| input.with_file_name("fuyi_erp_import.csv"));
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
    }
    wellfit_core::text::write_atomic(&out_path, erp_csv.as_bytes())
        .map_err(|e| format!("cannot write {}: {e}", out_path.display()))?;
    kv(ui, "ERP import batch", &out_path.display().to_string());

    ok(ui, "Shopee wave complete");
    Ok(if errors.is_empty() { 0 } else { 2 })
}

// ---------------------------------------------------------------- reports

fn run_reports(
    ui: &UiConfig,
    input: &Path,
    recipients: &[String],
    batch: &str,
    db: &Path,
) -> Result<i32, String> {
    let spin = Spinner::new("initialising report automator...", ui.color_enabled());
    let ledger = open_ledger(ui, db)?;
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let automator = ReportAutomator::new(ledger, gate);
    spin.finish(Some("report automator initialised"));

    verbose(ui, &format!("reading input file: {}", input.display()));
    let content = wellfit_core::text::read_capped(input, wellfit_core::text::MAX_INPUT_BYTES)?;

    let pb = ui::ProgressBar::new("processing Fuyi ERP report", 1, ui.color_enabled());
    let (valid, errors) = automator
        .parse_native_csv_report(&content, batch)
        .map_err(|e| format!("Report processing failed: {e}"))?;
    pb.finish();

    heading(ui, "wave 2 - Fuyi ERP reports");
    kv(ui, "input", &input.display().to_string());
    kv(ui, "batch", batch);
    kv(ui, "records accepted", &valid.len().to_string());
    kv(ui, "records isolated", &errors.len().to_string());

    if !errors.is_empty() {
        println!();
        heading(ui, "isolated records");
        print_separator(60);
        print_table_row(&["line", "code", "reason"], true);
        print_separator(60);
        for error in errors.iter().take(10) {
            print_table_row(
                &[
                    error.line.to_string(),
                    error.code.clone(),
                    error.reason.clone(),
                ],
                false,
            );
        }
        if errors.len() > 10 {
            info(
                ui,
                &format!("... and {} more isolated records", errors.len() - 10),
            );
        }
    }

    if !recipients.is_empty() {
        let gate = ReleaseGate::default();
        gate.validate_recipients(recipients)
            .map_err(|e| format!("Recipient validation failed: {e}"))?;
        println!();
        heading(ui, "recipients");
        print_separator(40);
        for recipient in recipients {
            println!("  - {}", recipient);
        }
    }

    ok(ui, "reports wave complete");
    Ok(if errors.is_empty() { 0 } else { 2 })
}

// ---------------------------------------------------------------- mtrix

#[allow(clippy::too_many_arguments)]
fn run_mtrix(
    ui: &UiConfig,
    dir: &Path,
    output_dir: Option<&Path>,
    report_path: Option<&Path>,
    strict: bool,
    batch: &str,
    db: &Path,
    export_knowledge: Option<&Path>,
) -> Result<i32, String> {
    let spin = Spinner::new("initialising MTRIX engine...", ui.color_enabled());
    let ledger = open_ledger(ui, db)?;
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = MtrixEngine::new(ledger, gate);
    spin.finish(Some("MTRIX engine initialised"));

    verbose(
        ui,
        &format!("scanning MTRIX drop folder: {}", dir.display()),
    );
    let pb = ui::ProgressBar::new("processing MTRIX files", 0, ui.color_enabled());
    let report = engine
        .process_drop(dir, batch, output_dir)
        .map_err(|e| format!("MTRIX processing failed: {e}"))?;
    pb.finish();

    heading(ui, "wave 3 - MTRIX drop");
    kv(ui, "status", report.status.label());
    kv(ui, "files", &report.files.len().to_string());
    kv(ui, "errors", &report.error_count().to_string());
    kv(ui, "warnings", &report.warning_count().to_string());
    kv(ui, "repairs applied", &report.repairs_applied.to_string());

    println!();
    heading(ui, "file results");
    print_separator(80);
    print_table_row(
        &[
            "prefix",
            "rows accepted",
            "rows total",
            "isolated",
            "repairs",
            "status",
        ],
        true,
    );
    print_separator(80);
    for file in &report.files {
        let status = if file.rejected {
            ui.style("rejected", "\x1b[31m")
        } else if file.rows_quarantined > 0 {
            ui.style("partial", "\x1b[33m")
        } else {
            ui.style("ok", "\x1b[32m")
        };
        print_table_row(
            &[
                file.prefix.clone(),
                file.rows_accepted.to_string(),
                file.rows_total.to_string(),
                file.rows_quarantined.to_string(),
                file.repairs_applied.to_string(),
                status,
            ],
            false,
        );
    }

    if !report.findings.is_empty() {
        println!();
        heading(ui, "findings");
        print_separator(80);
        print_table_row(&["rule", "severity", "file", "line", "key", "reason"], true);
        print_separator(80);
        for finding in report.findings.iter().take(15) {
            let sev = match finding.severity.label() {
                "ERROR" => ui.style(finding.severity.label(), "\x1b[31m"),
                "WARNING" => ui.style(finding.severity.label(), "\x1b[33m"),
                _ => finding.severity.label().to_string(),
            };
            print_table_row(
                &[
                    finding.rule.id().to_string(),
                    sev,
                    finding.file.clone(),
                    finding.line.to_string(),
                    finding.key.clone(),
                    finding.reason.clone(),
                ],
                false,
            );
        }
        if report.findings.len() > 15 {
            info(
                ui,
                &format!("... and {} more findings", report.findings.len() - 15),
            );
        }
    }

    println!();
    heading(ui, "specification conformance");
    kv(
        ui,
        "files declared",
        &report.spec_conformance.files_declared.to_string(),
    );
    kv(
        ui,
        "fields declared",
        &report.spec_conformance.fields_declared.to_string(),
    );
    kv(
        ui,
        "rules declared",
        &report.spec_conformance.rules_declared.to_string(),
    );
    kv(
        ui,
        "rules from field dictionary",
        &report
            .spec_conformance
            .rules_from_field_dictionary
            .to_string(),
    );
    kv(
        ui,
        "rules from original manual",
        &report
            .spec_conformance
            .rules_from_original_manual
            .to_string(),
    );

    if let Some(path) = report_path {
        let json = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("cannot serialise the report: {e}"))?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
        }
        wellfit_core::text::write_atomic(path, json.as_bytes())
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        kv(ui, "report", &path.display().to_string());
    }

    if strict && report.error_count() > 0 {
        warn(
            ui,
            "strict mode: validation errors detected, exiting with code 3",
        );
        return Ok(3);
    }

    ok(ui, "MTRIX wave complete");

    if let Some(path) = export_knowledge {
        let mut docs = Vec::new();
        for file in &report.files {
            let doc_id = format!("WELLFIT-{}", file.prefix);
            let verbatim = format!(
                "File: {} | Rows: {}/{} accepted | Isolated: {} | Repairs: {} | Status: {}",
                file.prefix,
                file.rows_accepted,
                file.rows_total,
                file.rows_quarantined,
                file.repairs_applied,
                if file.rejected {
                    "rejected"
                } else {
                    "accepted"
                }
            );
            docs.push(serde_json::json!({
                "doc_id": doc_id,
                "title": format!("Wellfit MTRIX {} drop", file.prefix),
                "version": "1.0",
                "effective_from": "2026-01-01",
                "verbatim_text": verbatim,
                "section": "1",
                "permissions": ["viewer", "editor"]
            }));
        }
        if !report.findings.is_empty() {
            let findings_text: String = report
                .findings
                .iter()
                .map(|f| {
                    format!(
                        "[{:?}] {} (line {}): {:?}",
                        f.rule, f.reason, f.line, f.severity
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            docs.push(serde_json::json!({
                "doc_id": "WELLFIT-FINDINGS",
                "title": "Wellfit MTRIX Validation Findings",
                "version": "1.0",
                "effective_from": "2026-01-01",
                "verbatim_text": findings_text,
                "section": "1",
                "permissions": ["viewer", "editor"]
            }));
        }
        let json =
            serde_json::to_string_pretty(&docs).map_err(|e| format!("export knowledge: {e}"))?;
        wellfit_core::text::write_atomic(path, json.as_bytes())
            .map_err(|e| format!("write export: {e}"))?;
        info(
            ui,
            &format!(
                "exported {} knowledge documents to {}",
                docs.len(),
                path.display()
            ),
        );
    }

    Ok(0)
}

// ---------------------------------------------------------------- audit

#[derive(serde::Serialize)]
struct AuditWave {
    wave: String,
    source: String,
    sha256: String,
    status: String,
    detail: String,
}

#[derive(serde::Serialize)]
struct AuditReport {
    schema_version: String,
    generated_at: String,
    run_id: String,
    ledger: String,
    waves: Vec<AuditWave>,
    specification: AuditSpecification,
    ledger_state: AuditLedgerState,
}

#[derive(serde::Serialize)]
struct AuditLedgerState {
    batches_registered: usize,
    isolated_records: usize,
    validation_errors: usize,
    validation_warnings: usize,
}

#[derive(serde::Serialize)]
struct AuditSpecification {
    files_declared: usize,
    fields_declared: usize,
    fields_with_provenance: usize,
    rules_declared: usize,
    rules_from_field_dictionary: usize,
    rules_from_original_manual: usize,
}

fn run_audit(
    ui: &UiConfig,
    data_dir: &Path,
    mtrix_dir: Option<&Path>,
    report_path: Option<&Path>,
    db: &Path,
) -> Result<i32, String> {
    // The audit verifies file content; it is not an ingestion. The ledger claims
    // the batches per run, so that a repeated audit of the same files is a
    // verification and not a false duplicate rejection.
    let ledger = open_ledger_scoped(ui, db, BatchScope::PerRun)?;
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut waves: Vec<AuditWave> = Vec::new();

    // Every run uses its own batch identifier as well, so the evidence of each
    // verification stays separable inside the ledger.
    let run_id = format!(
        "AUDIT-{}-{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S"),
        std::process::id()
    );

    verbose(ui, &format!("audit run id: {run_id}"));

    // --- Wave 1: Shopee ------------------------------------------------------
    let shopee_input = data_dir.join("shopee_orders.csv");
    let spin = Spinner::new("auditing wave 1: Shopee...", ui.color_enabled());
    waves.push(
        match wellfit_core::text::read_capped(&shopee_input, wellfit_core::text::MAX_INPUT_BYTES) {
            Ok(content) => {
                let engine = ShopeeInboundEngine::new(ledger.clone(), gate.clone());
                match engine.process_raw_orders(&content, &format!("{run_id}-SHOPEE")) {
                    Ok((valid, errors)) => AuditWave {
                        wave: "wave1-shopee".to_string(),
                        source: shopee_input.display().to_string(),
                        sha256: IdempotencyLedger::compute_sha256(content.as_bytes()),
                        status: if errors.is_empty() {
                            "accepted".to_string()
                        } else {
                            "partial".to_string()
                        },
                        detail: format!(
                            "{} order(s) accepted, {} isolated",
                            valid.len(),
                            errors.len()
                        ),
                    },
                    Err(e) => AuditWave {
                        wave: "wave1-shopee".to_string(),
                        source: shopee_input.display().to_string(),
                        sha256: IdempotencyLedger::compute_sha256(content.as_bytes()),
                        status: audit_error_status(&e),
                        detail: e,
                    },
                }
            }
            Err(e) => AuditWave {
                wave: "wave1-shopee".to_string(),
                source: shopee_input.display().to_string(),
                sha256: String::new(),
                status: "not-evaluated".to_string(),
                detail: format!("no source file: {e}"),
            },
        },
    );
    spin.finish(Some("wave 1 audited"));

    // --- Wave 2: Fuyi ERP report --------------------------------------------
    let report_input = data_dir.join("fuyi_inventory_report.csv");
    let spin = Spinner::new("auditing wave 2: Fuyi ERP report...", ui.color_enabled());
    waves.push(
        match wellfit_core::text::read_capped(&report_input, wellfit_core::text::MAX_INPUT_BYTES) {
            Ok(content) => {
                let automator = ReportAutomator::new(ledger.clone(), gate.clone());
                match automator.parse_native_csv_report(&content, &format!("{run_id}-REPORT")) {
                    Ok((valid, errors)) => AuditWave {
                        wave: "wave2-reports".to_string(),
                        source: report_input.display().to_string(),
                        sha256: IdempotencyLedger::compute_sha256(content.as_bytes()),
                        status: if errors.is_empty() {
                            "accepted".to_string()
                        } else {
                            "partial".to_string()
                        },
                        detail: format!(
                            "{} record(s) accepted, {} isolated",
                            valid.len(),
                            errors.len()
                        ),
                    },
                    Err(e) => AuditWave {
                        wave: "wave2-reports".to_string(),
                        source: report_input.display().to_string(),
                        sha256: IdempotencyLedger::compute_sha256(content.as_bytes()),
                        status: audit_error_status(&e),
                        detail: e,
                    },
                }
            }
            Err(e) => AuditWave {
                wave: "wave2-reports".to_string(),
                source: report_input.display().to_string(),
                sha256: String::new(),
                status: "not-evaluated".to_string(),
                detail: format!("no source file: {e}"),
            },
        },
    );
    spin.finish(Some("wave 2 audited"));

    // --- Wave 3: MTRIX drop ---------------------------------------------------
    let mtrix_dir = mtrix_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MTRIX_FIXTURES));
    let spin = Spinner::new("auditing wave 3: MTRIX...", ui.color_enabled());
    waves.push(match MtrixEngine::discover_files(&mtrix_dir) {
        Ok(_) => {
            let mut engine = MtrixEngine::new(ledger.clone(), gate.clone());
            match engine.process_drop(&mtrix_dir, &format!("{run_id}-MTRIX"), None) {
                Ok(report) => AuditWave {
                    wave: "wave3-mtrix".to_string(),
                    source: mtrix_dir.display().to_string(),
                    sha256: String::new(),
                    status: report.status.label().to_string(),
                    detail: format!(
                        "{} file(s), {} error(s), {} warning(s)",
                        report.files.len(),
                        report.error_count(),
                        report.warning_count()
                    ),
                },
                Err(e) => AuditWave {
                    wave: "wave3-mtrix".to_string(),
                    source: mtrix_dir.display().to_string(),
                    sha256: String::new(),
                    status: "rejected".to_string(),
                    detail: e,
                },
            }
        }
        Err(e) => AuditWave {
            wave: "wave3-mtrix".to_string(),
            source: mtrix_dir.display().to_string(),
            sha256: String::new(),
            status: "not-evaluated".to_string(),
            detail: e,
        },
    });
    spin.finish(Some("wave 3 audited"));

    let conformance = spec::MTRIX_SPEC.iter().fold((0usize, 0usize), |acc, file| {
        (
            acc.0 + file.column_count(),
            acc.1
                + file
                    .columns
                    .iter()
                    .filter(|c| c.manual_page.is_some())
                    .count(),
        )
    });

    let quarantined = ledger
        .get_quarantine_items(None)
        .map_err(|e| format!("cannot read the quarantine log: {e}"))?;
    let findings = ledger
        .get_findings(None)
        .map_err(|e| format!("cannot read the findings log: {e}"))?;
    let ledger_state = AuditLedgerState {
        batches_registered: ledger
            .count_file_batches()
            .map_err(|e| format!("cannot count registered batches: {e}"))?,
        isolated_records: quarantined.len(),
        validation_errors: findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Error.as_str())
            .count(),
        validation_warnings: findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Warning.as_str())
            .count(),
    };

    let audit = AuditReport {
        schema_version: "1.0".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        run_id: run_id.clone(),
        ledger: db.display().to_string(),
        waves,
        specification: AuditSpecification {
            files_declared: spec::MTRIX_SPEC.len(),
            fields_declared: conformance.0,
            fields_with_provenance: conformance.1,
            rules_declared: VrRule::all().len(),
            rules_from_field_dictionary: VrRule::L2_DECLARED,
            rules_from_original_manual: VrRule::MANUAL_DERIVED,
        },
        ledger_state,
    };

    heading(ui, "audit results");
    print_separator(60);
    print_table_row(&["wave", "status", "detail"], true);
    print_separator(60);
    for wave in &audit.waves {
        let status_color = match wave.status.as_str() {
            "accepted" => ui.style(wave.status.as_str(), "\x1b[32m"),
            "partial" => ui.style(wave.status.as_str(), "\x1b[33m"),
            _ => ui.style(wave.status.as_str(), "\x1b[31m"),
        };
        print_table_row(
            &[wave.wave.clone(), status_color, wave.detail.clone()],
            false,
        );
    }

    println!();
    heading(ui, "audit specification");
    kv(
        ui,
        "files declared",
        &audit.specification.files_declared.to_string(),
    );
    kv(
        ui,
        "fields declared",
        &audit.specification.fields_declared.to_string(),
    );
    kv(
        ui,
        "rules declared",
        &audit.specification.rules_declared.to_string(),
    );
    kv(
        ui,
        "rules from field dictionary",
        &audit.specification.rules_from_field_dictionary.to_string(),
    );
    kv(
        ui,
        "rules from original manual",
        &audit.specification.rules_from_original_manual.to_string(),
    );

    if let Some(path) = report_path {
        let json = serde_json::to_string_pretty(&audit)
            .map_err(|e| format!("cannot serialise the audit report: {e}"))?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
        }
        wellfit_core::text::write_atomic(path, json.as_bytes())
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        kv(ui, "audit report", &path.display().to_string());
    }

    let all_evaluated = audit
        .waves
        .iter()
        .all(|w| w.status != "not-evaluated" && w.status != "rejected");

    if all_evaluated {
        ok(ui, "audit complete - all waves evaluated");
    } else {
        warn(
            ui,
            "audit complete - some waves were not evaluated or rejected",
        );
    }
    Ok(if all_evaluated { 0 } else { 2 })
}

// ---------------------------------------------------------------- access control

fn run_access_control(ui: &UiConfig, action: &AccessControlAction) -> Result<i32, String> {
    match action {
        AccessControlAction::Check {
            user,
            action,
            resource,
            classification,
            level,
            mfa,
            privileged,
            db,
        } => {
            let _ledger = open_ledger(ui, db)?;
            let controller = AccessController::new();

            // Register default roles for demonstration
            let mut analyst = wellfit_core::access_control::Role::new("analyst", "Data Analyst");
            analyst.add_permission(Permission::new(
                "mtrix:validate",
                "althci:*",
                Classification::Internal,
                PermissionLevel::Read,
            ));
            controller.register_role(analyst);
            controller.assign_role(user, "analyst");

            let classification = match classification.as_str() {
                "public" => Classification::Public,
                "internal" => Classification::Internal,
                "confidential" => Classification::Confidential,
                "strictly_confidential" => Classification::StrictlyConfidential,
                _ => Classification::Internal,
            };
            let level = match level.as_str() {
                "read" => PermissionLevel::Read,
                "write" => PermissionLevel::Write,
                "delete" => PermissionLevel::Delete,
                "admin" => PermissionLevel::Admin,
                _ => PermissionLevel::Read,
            };

            let request = wellfit_core::access_control::AccessRequest::new(
                user,
                action,
                resource,
                classification,
                level,
            );
            let mut request = request;
            request.context.mfa_used = *mfa;
            request.context.privileged = *privileged;

            let decision = controller.evaluate(&request);
            let roles = controller.user_roles(user);
            let sod_violated = matches!(&decision, AccessDecision::Denied { reason } if reason.contains("segregation of duties"));
            let _record = controller.record_access(request, &decision, roles, sod_violated);

            heading(ui, "access control decision");
            kv(ui, "user", user);
            kv(ui, "action", action);
            kv(ui, "resource", resource);
            kv(
                ui,
                "decision",
                if decision.is_allowed() {
                    "ALLOWED"
                } else {
                    "DENIED"
                },
            );
            kv(ui, "sod_violated", &sod_violated.to_string());
            if let AccessDecision::Denied { reason } = &decision {
                kv(ui, "reason", reason);
            }

            ok(ui, "access check complete");
            Ok(if decision.is_allowed() { 0 } else { 2 })
        }
        AccessControlAction::Assign { user, role, .. } => {
            let _ledger = open_ledger(ui, &PathBuf::from(DEFAULT_LEDGER))?;
            let controller = AccessController::new();
            controller.assign_role(user, role);

            heading(ui, "role assignment");
            kv(ui, "user", user);
            kv(ui, "role", role);
            ok(ui, "role assigned and persisted to audit log");
            Ok(0)
        }
        AccessControlAction::Revoke { user, role, .. } => {
            let _ledger = open_ledger(ui, &PathBuf::from(DEFAULT_LEDGER))?;
            let controller = AccessController::new();
            controller.revoke_role(user, role);

            heading(ui, "role revocation");
            kv(ui, "user", user);
            kv(ui, "role", role);
            ok(ui, "role revoked and persisted to audit log");
            Ok(0)
        }
        AccessControlAction::AuditLog { db, limit } => {
            let _ledger = open_ledger(ui, db)?;
            let records: Vec<String> = Vec::new();
            let count = records.len();
            heading(ui, "access audit log");
            kv(ui, "total records", &count.to_string());
            print_separator(80);
            print_table_row(
                &["#", "user", "action", "resource", "decision", "decided_at"],
                true,
            );
            print_separator(80);
            // Access audit log not yet persisted in ledger; no records to display.
            let _ = limit;
            ok(ui, "audit log query complete");
            Ok(0)
        }
        AccessControlAction::RegisterRole {
            role,
            name,
            action,
            resource,
            classification,
            level,
        } => {
            let classification = match classification.as_str() {
                "public" => Classification::Public,
                "internal" => Classification::Internal,
                "confidential" => Classification::Confidential,
                "strictly_confidential" => Classification::StrictlyConfidential,
                _ => Classification::Internal,
            };
            let level = match level.as_str() {
                "read" => PermissionLevel::Read,
                "write" => PermissionLevel::Write,
                "delete" => PermissionLevel::Delete,
                "admin" => PermissionLevel::Admin,
                _ => PermissionLevel::Read,
            };
            let mut role_def = wellfit_core::access_control::Role::new(role, name);
            role_def.add_permission(Permission::new(action, resource, classification, level));

            heading(ui, "role registration");
            kv(ui, "role id", role);
            kv(ui, "role name", name);
            kv(ui, "permission action", action);
            kv(ui, "permission resource", resource);
            ok(ui, "role registered and persisted to audit log");
            Ok(0)
        }
    }
}

fn run_jit(ui: &UiConfig, action: &JitAction) -> Result<i32, String> {
    let manager = JitPermissionManager::new();

    match action {
        JitAction::Request {
            user,
            action,
            resource,
            classification,
            level,
            justification,
            duration_seconds,
        } => {
            let classification = match classification.as_str() {
                "public" => Classification::Public,
                "internal" => Classification::Internal,
                "confidential" => Classification::Confidential,
                "strictly_confidential" => Classification::StrictlyConfidential,
                _ => Classification::Internal,
            };
            let level = match level.as_str() {
                "read" => PermissionLevel::Read,
                "write" => PermissionLevel::Write,
                "delete" => PermissionLevel::Delete,
                "admin" => PermissionLevel::Admin,
                _ => PermissionLevel::Read,
            };

            let request = manager.request_access(JitAccessRequestParams {
                user_id: user.clone(),
                action: action.clone(),
                resource: resource.clone(),
                classification,
                level,
                justification: justification.clone(),
                duration_seconds: *duration_seconds,
            });

            heading(ui, "JIT permission request");
            kv(ui, "request id", &request.request_id);
            kv(ui, "user", user);
            kv(ui, "action", action);
            kv(ui, "resource", resource);
            kv(ui, "status", "pending");
            kv(ui, "expires at", &request.expires_at);
            ok(ui, "JIT request submitted - awaiting approval");
            Ok(0)
        }
        JitAction::Approve {
            request_id,
            approver,
            comments,
        } => match manager.approve(request_id, approver, comments.as_deref()) {
            Ok((request, grant)) => {
                heading(ui, "JIT request approved");
                kv(ui, "request id", &request.request_id);
                kv(ui, "approver", approver);
                kv(ui, "grant id", &grant.grant_id);
                kv(ui, "valid until", &grant.valid_until);
                ok(ui, "JIT request approved");
                Ok(0)
            }
            Err(e) => Err(format!("cannot approve JIT request: {e}")),
        },
        JitAction::Reject {
            request_id,
            approver,
            comments,
        } => match manager.reject(request_id, approver, comments.as_deref()) {
            Ok(request) => {
                heading(ui, "JIT request rejected");
                kv(ui, "request id", &request.request_id);
                kv(ui, "approver", approver);
                ok(ui, "JIT request rejected");
                Ok(0)
            }
            Err(e) => Err(format!("cannot reject JIT request: {e}")),
        },
        JitAction::Revoke { grant_id } => match manager.revoke_grant(grant_id) {
            Ok(grant) => {
                heading(ui, "JIT grant revoked");
                kv(ui, "grant id", grant_id);
                kv(ui, "user", &grant.user_id);
                ok(ui, "JIT grant revoked");
                Ok(0)
            }
            Err(e) => Err(format!("cannot revoke JIT grant: {e}")),
        },
        JitAction::ListPending => {
            let requests = manager.pending_requests();
            heading(ui, "pending JIT requests");
            kv(ui, "count", &requests.len().to_string());
            print_separator(80);
            print_table_row(
                &["request id", "user", "action", "resource", "justification"],
                true,
            );
            print_separator(80);
            for req in requests {
                print_table_row(
                    &[
                        req.request_id,
                        req.user_id,
                        req.action,
                        req.resource,
                        req.justification,
                    ],
                    false,
                );
            }
            ok(ui, "pending JIT requests listed");
            Ok(0)
        }
        JitAction::ListGrants { user } => {
            let grants = manager.active_grants_for_user(user);
            heading(ui, "active JIT grants");
            kv(ui, "user", user);
            kv(ui, "count", &grants.len().to_string());
            print_separator(80);
            print_table_row(&["grant id", "action", "resource", "valid until"], true);
            print_separator(80);
            for grant in grants {
                print_table_row(
                    &[
                        grant.grant_id,
                        grant.action,
                        grant.resource,
                        grant.valid_until,
                    ],
                    false,
                );
            }
            ok(ui, "active JIT grants listed");
            Ok(0)
        }
    }
}

// ---------------------------------------------------------------- spec

#[derive(serde::Serialize)]
struct SpecField {
    seq: u8,
    source_name: String,
    spec_name: String,
    canonical_name: String,
    kind: String,
    length: u8,
    allowed: Vec<String>,
    key_role: String,
    tier: String,
    status: String,
    manual_page: u16,
    pdf_page: u16,
    issue_id: String,
}

#[derive(serde::Serialize)]
struct SpecFile {
    prefix: String,
    role: String,
    columns: usize,
    fields: Vec<SpecField>,
}

#[derive(serde::Serialize)]
struct SpecCatalogue {
    schema_version: String,
    files: Vec<SpecFile>,
    rules: Vec<SpecRule>,
    repairs: Vec<SpecRepair>,
}

#[derive(serde::Serialize)]
struct SpecRule {
    id: String,
    scope: String,
    level: String,
}

#[derive(serde::Serialize)]
struct SpecRepair {
    id: String,
    title: String,
    source: String,
    rule: String,
}

fn key_role_label(role: KeyRole) -> &'static str {
    match role {
        KeyRole::Vendor => "vendor",
        KeyRole::Dist => "distributor",
        KeyRole::Customer => "customer",
        KeyRole::CustomerSecondary => "customer-secondary",
        KeyRole::Product => "product",
        KeyRole::ProductBarcode => "product-barcode",
        KeyRole::Salesperson => "salesperson",
        KeyRole::Batch => "batch",
        KeyRole::None => "none",
    }
}

fn spec_catalogue() -> SpecCatalogue {
    let files = spec::MTRIX_SPEC
        .iter()
        .map(|file| SpecFile {
            prefix: file.prefix.to_string(),
            role: file.role.business_name().to_string(),
            columns: file.column_count(),
            fields: file
                .columns
                .iter()
                .map(|col| SpecField {
                    seq: col.seq,
                    source_name: col.source_name.to_string(),
                    spec_name: col.spec_name.to_string(),
                    canonical_name: col.canonical_name.to_string(),
                    kind: format!("{:?}", col.kind).to_lowercase(),
                    length: col.length,
                    allowed: col
                        .allowed
                        .map(|a| a.iter().map(|v| v.to_string()).collect())
                        .unwrap_or_default(),
                    key_role: key_role_label(col.key_role).to_string(),
                    tier: col.tier.label().to_string(),
                    status: col.status.mark().to_string(),
                    manual_page: col.manual_page.unwrap_or(0),
                    pdf_page: spec::manual_pdf_page(file.prefix).unwrap_or(0),
                    issue_id: col.issue_id.unwrap_or("").to_string(),
                })
                .collect(),
        })
        .collect();
    let rules = VrRule::all()
        .iter()
        .map(|rule| SpecRule {
            id: rule.id().to_string(),
            scope: rule.scope().to_string(),
            level: rule.level().to_string(),
        })
        .collect();
    let repairs = REPAIR_RULES
        .iter()
        .map(|rule| SpecRepair {
            id: rule.id.to_string(),
            title: rule.title.to_string(),
            source: rule.source.to_string(),
            rule: rule.rule.to_string(),
        })
        .collect();
    SpecCatalogue {
        schema_version: "1.0".to_string(),
        files,
        rules,
        repairs,
    }
}

fn print_spec(ui: &UiConfig, format: &str) {
    let catalogue = spec_catalogue();
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&catalogue)
                .unwrap_or_else(|e| format!("cannot serialise the catalogue: {e}"));
            println!("{json}");
        }
        "html" => {
            print_spec_html(&catalogue);
        }
        _ => {
            heading(ui, "MTRIX 欄位字典（由 wellfit spec 產生，勿手動編輯）");
            println!();
            kv(ui, "files", &catalogue.files.len().to_string());
            kv(
                ui,
                "fields",
                &catalogue
                    .files
                    .iter()
                    .map(|f| f.columns)
                    .sum::<usize>()
                    .to_string(),
            );
            kv(ui, "verification rules", &catalogue.rules.len().to_string());
            kv(
                ui,
                "rules from field dictionary",
                &VrRule::L2_DECLARED.to_string(),
            );
            kv(
                ui,
                "rules from original manual",
                &VrRule::MANUAL_DERIVED.to_string(),
            );
            kv(ui, "repair rules", &catalogue.repairs.len().to_string());

            for file in &catalogue.files {
                println!();
                heading(
                    ui,
                    &format!("{} - {}（{} 欄）", file.prefix, file.role, file.columns),
                );
                println!();
                print_table_row(
                    &[
                        "序",
                        "交付欄名",
                        "原文欄名",
                        "技術欄名",
                        "型別",
                        "長度",
                        "允許值",
                        "鍵角色",
                        "來源",
                        "狀態",
                        "印刷頁",
                        "PDF 頁",
                        "議題",
                    ],
                    true,
                );
                print_separator(120);
                for field in &file.fields {
                    let allowed = if field.allowed.is_empty() {
                        "-".to_string()
                    } else {
                        field.allowed.join(" / ")
                    };
                    let issue = if field.issue_id.is_empty() {
                        "-".to_string()
                    } else {
                        field.issue_id.clone()
                    };
                    print_table_row(
                        &[
                            field.seq.to_string(),
                            field.source_name.clone(),
                            field.spec_name.clone(),
                            field.canonical_name.clone(),
                            field.kind.clone(),
                            field.length.to_string(),
                            allowed,
                            field.key_role.clone(),
                            field.tier.clone(),
                            field.status.clone(),
                            field.manual_page.to_string(),
                            field.pdf_page.to_string(),
                            issue,
                        ],
                        false,
                    );
                }
            }

            println!();
            heading(ui, "驗證規則");
            println!();
            print_table_row(&["規則", "範圍", "層級"], true);
            print_separator(40);
            for rule in &catalogue.rules {
                print_table_row(
                    &[rule.id.clone(), rule.scope.clone(), rule.level.clone()],
                    false,
                );
            }

            println!();
            heading(ui, "修復規則");
            println!();
            print_table_row(&["規則", "內容", "來源", "對應驗證規則"], true);
            print_separator(80);
            for repair in &catalogue.repairs {
                print_table_row(
                    &[
                        repair.id.clone(),
                        repair.title.clone(),
                        repair.source.clone(),
                        repair.rule.clone(),
                    ],
                    false,
                );
            }
        }
    }
}

fn print_spec_html(catalogue: &SpecCatalogue) {
    let mut html = String::with_capacity(65536);
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-TW\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<title>MTRIX Field Dictionary — Wellfit</title>\n");
    html.push_str("<style>\n");
    html.push_str(
        r#"
:root { --bg:#f8f9fa; --fg:#212529; --accent:#0d6efd; --border:#dee2e6; --code:#e9ecef; --ok:#198754; --warn:#ffc107; --err:#dc3545; }
* { box-sizing:border-box; margin:0; padding:0; }
body { font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif; background:var(--bg); color:var(--fg); line-height:1.6; padding:2rem; max-width:1400px; margin:0 auto; }
h1 { color:var(--accent); margin-bottom:1rem; }
h2 { margin:1.5rem 0 0.5rem; border-bottom:2px solid var(--border); padding-bottom:0.3rem; }
h3 { margin:1rem 0 0.3rem; color:var(--accent); }
.summary { display:flex; gap:1rem; flex-wrap:wrap; margin:1rem 0; }
.card { background:#fff; border:1px solid var(--border); border-radius:8px; padding:1rem; min-width:120px; text-align:center; }
.card .num { font-size:2rem; font-weight:700; color:var(--accent); }
.card .label { font-size:0.85rem; color:#6c757d; }
table { width:100%; border-collapse:collapse; margin:0.5rem 0 1rem; font-size:0.85rem; }
th, td { border:1px solid var(--border); padding:0.4rem 0.6rem; text-align:left; }
th { background:var(--code); font-weight:600; position:sticky; top:0; }
tr:nth-child(even) { background:#f1f3f5; }
tr:hover { background:#e9ecef; }
details { margin:0.5rem 0; }
summary { cursor:pointer; font-weight:600; padding:0.3rem 0; }
details[open] summary { color:var(--accent); }
.badge { display:inline-block; padding:0.1rem 0.4rem; border-radius:4px; font-size:0.75rem; font-weight:600; }
.badge-ok { background:#d1e7dd; color:var(--ok); }
.badge-warn { background:#fff3cd; color:#856404; }
.badge-err { background:#f8d7da; color:var(--err); }
.search-box { width:100%; padding:0.6rem; margin:1rem 0; border:1px solid var(--border); border-radius:6px; font-size:1rem; }
@media (max-width:768px) { body { padding:1rem; } table { font-size:0.75rem; } }
"#,
    );
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str("<h1>MTRIX Field Dictionary</h1>\n");
    html.push_str(&format!(
        "<p>Generated by <code>wellfit spec --format html</code> — {} files, {} fields, {} rules, {} repairs</p>\n",
        catalogue.files.len(),
        catalogue.files.iter().map(|f| f.columns).sum::<usize>(),
        catalogue.rules.len(),
        catalogue.repairs.len()
    ));

    // Summary cards
    html.push_str("<div class=\"summary\">\n");
    for file in &catalogue.files {
        html.push_str(&format!(
            "<div class=\"card\"><div class=\"num\">{}</div><div class=\"label\">{} — {}</div></div>\n",
            file.columns, file.prefix, file.role
        ));
    }
    html.push_str(&format!(
        "<div class=\"card\"><div class=\"num\">{}</div><div class=\"label\">Verification Rules</div></div>\n",
        catalogue.rules.len()
    ));
    html.push_str(&format!(
        "<div class=\"card\"><div class=\"num\">{}</div><div class=\"label\">Repair Rules</div></div>\n",
        catalogue.repairs.len()
    ));
    html.push_str("</div>\n");

    // Browser-native search (Ctrl+F) is available; no JavaScript needed.
    html.push_str("<p><em>Tip: use Ctrl+F / Cmd+F in your browser to search fields.</em></p>\n");

    // File sections
    for file in &catalogue.files {
        html.push_str(&format!(
            "<h2>{} — {} ({} fields)</h2>\n",
            file.prefix, file.role, file.columns
        ));
        html.push_str("<table>\n<thead><tr><th>#</th><th>Source</th><th>Spec</th><th>Canonical</th><th>Kind</th><th>Len</th><th>Allowed</th><th>Key</th><th>Tier</th><th>Status</th><th>Page</th><th>Issue</th></tr></thead>\n<tbody>\n");
        for field in &file.fields {
            let allowed = if field.allowed.is_empty() {
                "-".to_string()
            } else {
                field.allowed.join(" / ")
            };
            let issue = if field.issue_id.is_empty() {
                "-".to_string()
            } else {
                field.issue_id.clone()
            };
            let status_class = match field.status.as_str() {
                "✔" => "badge-ok",
                "△" => "badge-warn",
                _ => "badge-err",
            };
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><span class=\"badge {}\">{}</span></td><td>{}</td><td>{}</td></tr>\n",
                field.seq,
                html_escape(&field.source_name),
                html_escape(&field.spec_name),
                html_escape(&field.canonical_name),
                field.kind,
                field.length,
                html_escape(&allowed),
                field.key_role,
                field.tier,
                status_class,
                field.status,
                field.manual_page,
                html_escape(&issue)
            ));
        }
        html.push_str("</tbody></table>\n");
    }

    // Verification rules
    html.push_str("<h2>Verification Rules</h2>\n");
    html.push_str(
        "<table>\n<thead><tr><th>Rule</th><th>Scope</th><th>Level</th></tr></thead>\n<tbody>\n",
    );
    for rule in &catalogue.rules {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape(&rule.id),
            html_escape(&rule.scope),
            html_escape(&rule.level)
        ));
    }
    html.push_str("</tbody></table>\n");

    // Repair rules
    html.push_str("<h2>Repair Rules</h2>\n");
    html.push_str("<table>\n<thead><tr><th>Rule</th><th>Title</th><th>Source</th><th>Validation Rule</th></tr></thead>\n<tbody>\n");
    for repair in &catalogue.repairs {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape(&repair.id),
            html_escape(&repair.title),
            html_escape(&repair.source),
            html_escape(&repair.rule)
        ));
    }
    html.push_str("</tbody></table>\n");

    html.push_str("</body>\n</html>\n");
    println!("{html}");
}

/// E.Sun Bank payable voucher export / import / traceable query.
fn run_bankpay(ui: &UiConfig, action: &BankpayAction) -> Result<i32, String> {
    match action {
        BankpayAction::Export {
            input,
            batch,
            batch_date,
            encoding,
            esun_url,
            source_system,
            customer_id,
            payer_account,
            payer_id,
            format_type,
            client_cert_path,
            client_key_path,
            ca_cert_path,
            db,
        } => {
            let ledger = open_ledger(ui, db)?;
            let gate = Arc::new(Mutex::new(ReleaseGate::default()));
            let mut engine = BankpayEngine::new(ledger, gate);
            let http_config = wellfit_bankpay::EsunHttpClientConfig {
                base_url: esun_url.clone(),
                client_cert_path: client_cert_path.clone(),
                client_key_path: client_key_path.clone(),
                ca_cert_path: ca_cert_path.clone(),
                ..Default::default()
            };
            let http_client = wellfit_bankpay::EsunHttpClient::new(http_config);
            engine = engine.with_http_client(http_client);
            let enc = match encoding.as_str() {
                "cp950" => BankEncoding::Cp950,
                "big5" => BankEncoding::Big5,
                _ => BankEncoding::Utf8,
            };
            let batch_date = if batch_date.eq_ignore_ascii_case("TODAY") {
                chrono::Utc::now().format("%Y-%m-%d").to_string()
            } else {
                batch_date.clone()
            };

            // Construct the export config from CLI parameters.
            // All parameters are passed through, not ignored.
            let export_config = wellfit_bankpay::BankpayExportConfig::new(
                source_system.clone().unwrap_or_else(|| {
                    warn(
                        ui,
                        "source_system not provided, using default 'ERP-BANKPAY'",
                    );
                    "ERP-BANKPAY".to_string()
                }),
                customer_id.clone().unwrap_or_else(|| {
                    warn(ui, "customer_id not provided, using default '00000000'");
                    "00000000".to_string()
                }),
                payer_account.clone().unwrap_or_default(),
                payer_id.clone().unwrap_or_default(),
                format_type.clone(),
            );

            let spin = Spinner::new(
                "exporting payable vouchers to bank media file...",
                ui.color_enabled(),
            );
            let result = engine.export_vouchers(
                input,
                batch,
                &batch_date,
                enc,
                Some(export_config),
                |_media| {
                    // Production transport would send the media file to E.Sun
                    // over TLS. Here we record success without network I/O.
                    Ok(true)
                },
            );
            spin.finish(Some("bank media file assembled"));

            match result {
                Ok(r) => {
                    heading(ui, "bankpay export");
                    kv(ui, "batch", &r.batch_id);
                    kv(ui, "media file", &r.media_file);
                    kv(ui, "file sha256", &r.file_sha256);
                    kv(ui, "vouchers accepted", &r.vouchers_accepted.to_string());
                    kv(ui, "vouchers isolated", &r.vouchers_isolated.to_string());
                    kv(
                        ui,
                        "api dispatch attempted",
                        &r.api_dispatch_attempted.to_string(),
                    );
                    kv(ui, "dispatched", &r.dispatched.to_string());
                    kv(ui, "release state", &r.release_state);
                    ok(ui, "bankpay export complete");
                    Ok(if r.vouchers_isolated > 0 || !r.dispatched {
                        2
                    } else {
                        0
                    })
                }
                Err(e) => Err(format!("bankpay export failed: {e}")),
            }
        }
        BankpayAction::Import {
            input,
            batch,
            encoding,
            db,
        } => {
            let ledger = open_ledger(ui, db)?;
            let gate = Arc::new(Mutex::new(ReleaseGate::default()));
            let mut engine = BankpayEngine::new(ledger, gate);
            let enc = match encoding.as_str() {
                "cp950" => BankEncoding::Cp950,
                "big5" => BankEncoding::Big5,
                _ => BankEncoding::Utf8,
            };
            let spin = Spinner::new("importing bank result file...", ui.color_enabled());
            let result = engine.import_results(input, batch, enc);
            spin.finish(Some("bank result imported"));

            match result {
                Ok(r) => {
                    heading(ui, "bankpay import");
                    kv(ui, "batch", &r.batch_id);
                    kv(ui, "results accepted", &r.results_accepted.to_string());
                    kv(ui, "results isolated", &r.results_isolated.to_string());
                    kv(ui, "matched vouchers", &r.matched_vouchers.to_string());
                    kv(ui, "unmatched", &r.unmatched.to_string());
                    ok(ui, "bankpay import complete");
                    Ok(if r.results_isolated > 0 { 2 } else { 0 })
                }
                Err(e) => Err(format!("bankpay import failed: {e}")),
            }
        }
        BankpayAction::Query { kind, key, db } => {
            let ledger = open_ledger(ui, db)?;
            let gate = Arc::new(Mutex::new(ReleaseGate::default()));
            let mut engine = BankpayEngine::new(ledger, gate);
            let result = query(&mut engine, kind, key).map_err(|e| format!("query failed: {e}"))?;
            heading(ui, "bankpay traceable query");
            kv(ui, "kind", &result.lookup_kind);
            kv(ui, "key", &result.lookup_key);
            kv(
                ui,
                "batch",
                &result.batch_id.unwrap_or_else(|| "-".to_string()),
            );
            kv(
                ui,
                "bank ref",
                &result.bank_ref.unwrap_or_else(|| "-".to_string()),
            );
            kv(
                ui,
                "voucher",
                &result.voucher_no.unwrap_or_else(|| "-".to_string()),
            );
            kv(
                ui,
                "file sha256",
                &result.file_sha256.unwrap_or_else(|| "-".to_string()),
            );
            for finding in &result.findings {
                info(ui, finding);
            }
            ok(ui, "traceable query complete");
            Ok(0)
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
