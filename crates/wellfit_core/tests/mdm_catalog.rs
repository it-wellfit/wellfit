#![forbid(unsafe_code)]

//! Wellfit Core - MDM catalogue integration tests.
//!
//! The authoritative content of the ALTHPD product master is embedded as a
//! constant. The source file itself is not copied into the repository: it is
//! internal customer data that carries customer codes and prices.

use wellfit_core::mdm::{
    join_vendor_country_prefix, normalize_barcode, split_vendor_country_prefix, validate_barcode,
    MdmCatalog, MdmHub, BARCODE_TYPE_DUN14, BARCODE_TYPE_EAN13, CONFIRMED_CHANNELS,
    DEFAULT_CHANNEL_FALLBACK, UNVERIFIED_CHANNELS, UOM_PAIR,
};

/// The authoritative ALTHPD product master, exactly as delivered.
const AUTHORITATIVE_ALTHPD: &str = concat!(
    "TRANSACTION_DATE;DIST_ID;VENDOR_ID;VENDOR_COMPANY_NAME;PRODUCT_CODE;PACKAGE_TYPE;BARCODE;BARCODE_TYPE;DESCRIPTION;PRODUCT_DIV;PRODUCT_STATUS;COUNTRY_ISO\n",
    "20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;8;0;7898656600087 ;1;;RSF;A;USA\n",
    "20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;16;0;7898656600162 ;1;;RSF;A;USA\n",
    "20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;19;0;7898656600193 ;1;;RSF;A;USA\n",
    "20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;20;0;7898656600209 ;1;;RSF;A;USA\n",
);

/// Product code and EAN13 of every product of the authoritative master.
const AUTHORITATIVE_PRODUCTS: &[(&str, &str)] = &[
    ("8", "7898656600087"),
    ("16", "7898656600162"),
    ("19", "7898656600193"),
    ("20", "7898656600209"),
];

#[test]
fn test_authoritative_product_master_registers_four_skus() {
    let catalog = MdmCatalog::new();
    let count = catalog
        .register_from_mtrix_product(AUTHORITATIVE_ALTHPD)
        .expect("the authoritative product master must load");

    assert_eq!(count, 4);
    for (product_code, ean13) in AUTHORITATIVE_PRODUCTS {
        let sku = catalog
            .lookup_sku(product_code)
            .expect("product code must resolve");
        assert_eq!(sku.ean13, *ean13);
        assert_eq!(sku.item_no, *product_code);
        assert_eq!(sku.product_code, *product_code);
        assert_eq!(sku.seller_sku, *product_code);
        assert_eq!(sku.uom, UOM_PAIR);
    }
}

#[test]
fn test_lookup_by_ean13_resolves_the_delivered_barcode() {
    let catalog = MdmCatalog::new();
    let valid_count = catalog
        .register_from_mtrix_product(AUTHORITATIVE_ALTHPD)
        .expect("the authoritative product master must load");
    assert_eq!(valid_count, 4);

    let sku = catalog
        .lookup_by_ean13("7898656600087")
        .expect("barcode must resolve");
    assert_eq!(sku.item_no, "8");

    // The delivered file carries the barcode with a trailing space.
    let with_space = catalog
        .lookup_by_ean13("7898656600087 ")
        .expect("a trailing space must not break the lookup");
    assert_eq!(with_space.item_no, "8");
}

#[test]
fn test_demo_bootstrap_matches_the_authoritative_barcodes() {
    let catalog = MdmCatalog::demo_bootstrap();

    for (product_code, ean13) in AUTHORITATIVE_PRODUCTS {
        let sku = catalog
            .lookup_by_ean13(ean13)
            .expect("authoritative barcode must resolve");
        assert_eq!(sku.item_no, *product_code);
    }

    // The fabricated codes of the former bootstrap are gone.
    for fabricated in ["7890541735003", "7891234567890", "7899876543210"] {
        assert!(
            catalog.lookup_by_ean13(fabricated).is_none(),
            "{fabricated} must not be in the catalogue"
        );
    }
}

#[test]
fn test_dist_id_dot_mask_is_removed() {
    assert_eq!(
        MdmHub::normalize_dist_id("02.986.485.7023.000"),
        "029864857023000"
    );
    assert_eq!(
        MdmHub::normalize_dist_id("029864857023000"),
        "029864857023000"
    );
}

#[test]
fn test_store_type_alias_onlinne() {
    // The delivered ALTHCI row 995D carries "Onlinne".
    assert_eq!(MdmHub::normalize_store_type("Onlinne"), "Online");
    assert_eq!(MdmHub::normalize_store_type("online"), "Online");
    assert_eq!(MdmHub::normalize_store_type("Online"), "Online");
}

#[test]
fn test_barcode_normalization_keeps_leading_zeros() {
    assert_eq!(normalize_barcode("007898656600087 "), "007898656600087");
    assert_eq!(normalize_barcode(" 7898656600087 ;;\n"), "7898656600087");
    // A non numeric identifier must not be mangled.
    assert_eq!(normalize_barcode("ABC-123"), "ABC-123");
    assert_eq!(normalize_barcode(""), "");
}

#[test]
fn test_strict_barcode_validation() {
    assert!(validate_barcode("7898656600087", BARCODE_TYPE_EAN13).is_ok());
    assert!(validate_barcode("7898656600087", BARCODE_TYPE_DUN14).is_err());
    assert!(validate_barcode("78986566000870", BARCODE_TYPE_DUN14).is_ok());
}

#[test]
fn test_unknown_channel_resolves_to_none() {
    let catalog = MdmCatalog::demo_bootstrap();

    assert_eq!(
        catalog.resolve_channel_exact("Shopee").as_deref(),
        Some("995C")
    );
    assert_eq!(
        catalog.resolve_channel_exact("Tiktok Shop").as_deref(),
        Some("995D")
    );
    assert_eq!(
        catalog.resolve_channel_exact("JDID").as_deref(),
        Some("995A")
    );
    assert_eq!(
        catalog.resolve_channel_exact("Lazada").as_deref(),
        Some("995B")
    );
    assert!(catalog.resolve_channel_exact("No Such Channel").is_none());

    // The compatibility function keeps the legacy fallback.
    assert_eq!(
        catalog.resolve_channel("No Such Channel"),
        DEFAULT_CHANNEL_FALLBACK
    );
}

#[test]
fn test_unverified_channels_are_marked_as_such() {
    assert!(UNVERIFIED_CHANNELS.contains(&("MOMO", "995E")));
    assert!(UNVERIFIED_CHANNELS.contains(&("PCHOME", "995F")));
    assert!(UNVERIFIED_CHANNELS.contains(&("COUPANG", "995G")));

    for confirmed in CONFIRMED_CHANNELS {
        assert!(
            !UNVERIFIED_CHANNELS.contains(confirmed),
            "{confirmed:?} is confirmed by the ALTHCI master"
        );
    }
}

#[test]
fn test_vendor_country_prefix_split_and_join() {
    let (code, prefix) = split_vendor_country_prefix("USA6063343900011222");
    assert_eq!(code, "6063343900011222");
    assert_eq!(prefix, "USA");
    assert_eq!(
        join_vendor_country_prefix(&code, &prefix),
        "USA6063343900011222"
    );

    // The manual specifies a two letter ISO prefix ...
    let (code, prefix) = split_vendor_country_prefix("BR6063343900011222");
    assert_eq!(code, "6063343900011222");
    assert_eq!(prefix, "BR");

    // ... while the delivered files carry the manufacturer code only.
    let (code, prefix) = split_vendor_country_prefix("6063343900011222");
    assert_eq!(code, "6063343900011222");
    assert!(prefix.is_empty());
}

#[test]
fn test_columns_are_resolved_by_name_and_never_by_position() {
    // Same columns, different order.
    let shuffled = "COUNTRY_ISO;BARCODE;DESCRIPTION;PRODUCT_CODE\n\
                    USA;7898656600087;;8\n";
    let catalog = MdmCatalog::new();
    let count = catalog
        .register_from_mtrix_product(shuffled)
        .expect("a reordered header must still load");
    assert_eq!(count, 1);
    assert_eq!(catalog.lookup_sku("8").unwrap().ean13, "7898656600087");
}

#[test]
fn test_missing_required_column_is_rejected() {
    let catalog = MdmCatalog::new();
    let error = catalog
        .register_from_mtrix_product("PRODUCT_CODE;DESCRIPTION\n8;\n")
        .expect_err("a missing BARCODE column must fail");
    assert!(error.contains("BARCODE"), "{error}");
}

#[test]
fn test_barcode_length_must_match_the_declared_type() {
    let catalog = MdmCatalog::new();
    let error = catalog
        .register_from_mtrix_product(concat!(
            "PRODUCT_CODE;BARCODE;BARCODE_TYPE;DESCRIPTION\n",
            "8;789865660008;1;\n",
        ))
        .expect_err("a barcode that is not an EAN13 must fail");
    assert!(error.contains("EAN13"), "{error}");
}
