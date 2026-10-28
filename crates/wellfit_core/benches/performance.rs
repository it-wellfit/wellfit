//! Performance benchmarks for wellfit_core.
//!
//! Run with `cargo bench -p wellfit_core`. Uses a plain `main` (harness = false)
//! so it works on the stable toolchain without the nightly `test` feature.

use std::time::Instant;
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::mdm::MdmCatalog;

fn bench_ledger_insert() {
    let ledger = IdempotencyLedger::in_memory().expect("ledger");
    let start = Instant::now();
    for i in 0..1000 {
        let _ = ledger.register_file_batch(
            &format!("BATCH-{i}"),
            "mtrix",
            &format!("file_{i}.csv"),
            b"data",
            10,
        );
    }
    let elapsed = start.elapsed();
    println!("ledger register_file_batch x1000: {:?}", elapsed);
}

fn bench_mdm_lookup() {
    let catalog = MdmCatalog::new();
    let src = "TRANSACTION_DATE;DIST_ID;VENDOR_ID;VENDOR_COMPANY_NAME;PRODUCT_CODE;PACKAGE_TYPE;BARCODE;BARCODE_TYPE;DESCRIPTION;PRODUCT_DIV;PRODUCT_STATUS;COUNTRY_ISO\n\
20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;8;0;7898656600087 ;1;;RSF;A;USA\n";
    let _ = catalog.register_from_mtrix_product(src);
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = catalog.lookup_sku("8");
    }
    let elapsed = start.elapsed();
    println!("mdm lookup_sku x10000: {:?}", elapsed);
}

fn bench_sha256() {
    let data = vec![0u8; 1024 * 1024];
    let start = Instant::now();
    for _ in 0..10 {
        let _ = IdempotencyLedger::compute_sha256(&data);
    }
    let elapsed = start.elapsed();
    println!("sha256 1MB x10: {:?}", elapsed);
}

fn main() {
    bench_ledger_insert();
    bench_mdm_lookup();
    bench_sha256();
    println!("benchmarks complete");
}
