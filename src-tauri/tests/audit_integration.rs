//! Wiring integration test for the T1.04 audit log.
//!
//! This is the safety net for the typed event bus: a handler that is defined
//! but never `register`ed in `audit_handlers::register_all` would silently
//! drop its events. Here we build a *real* `OrgServices` (real `InProcessEventBus`,
//! real handlers, real `SqliteAuditRepository`), run one use case per entity
//! category, and assert the audit rows actually land.

use terative_lib::adapters::sqlite::open;
use terative_lib::application::ports::PaginationParams;
use terative_lib::commands::OrgServices;
use terative_lib::domain::client::NewClient;
use terative_lib::domain::invoice::NewInvoice;
use terative_lib::domain::line_item::NewLineItem;
use terative_lib::domain::money::{Currency, Money};
use terative_lib::domain::org::OrgCode;
use terative_lib::domain::payment::{NewPayment, PaymentMethod};
use chrono::NaiveDate;
use rust_decimal::Decimal;

fn services(tmp: &std::path::Path) -> OrgServices {
    let db_path = tmp.join("test.sqlite");
    let db = open(&db_path).expect("open in-memory-ish test db");
    OrgServices::new(
        OrgCode::parse("testorg").unwrap(),
        db,
        db_path,
        tmp.join("pdfs"),
        tmp.join("user_backups"),
        tmp.join("system_backups"),
        None,
    )
}

#[test]
fn use_cases_run_through_orgservices_land_in_the_activity_log() {
    let tmp = tempfile::tempdir().unwrap();
    let svc = services(tmp.path());

    // 1. Client.
    let client = svc
        .create_client
        .execute(NewClient {
            name: "Acme".into(),
            ..Default::default()
        })
        .unwrap();

    // 2. Draft invoice for that client.
    svc.create_draft_invoice
        .execute(NewInvoice {
            client_id: client.id,
            template_id: None,
            date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            due_date: None,
            line_items: vec![NewLineItem {
                catalog_item_id: None,
                description: "Consulting".into(),
                quantity: Decimal::from(1),
                unit_price: Money::new(10_000, Currency::Eur),
            }],
            tax_ids: vec![],
            notes: None,
            currency: Currency::Eur,
        })
        .unwrap();

    // 3. Payment (no allocations — needs no finalized invoice).
    svc.record_payment
        .execute(NewPayment {
            client_id: client.id,
            date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            amount: Money::new(5_000, Currency::Eur),
            method: PaymentMethod::BankTransfer,
            reference: None,
            allocations: vec![],
            notes: None,
        })
        .unwrap();

    // 4. Backup — exercises the `CreateBackup` use case + `BackupCreated`.
    svc.create_backup.execute().unwrap();

    // Every category above must have produced an audit row. If a handler
    // were defined but missing from `register_all`, its event_type would be
    // absent here.
    let page = svc
        .paginate_recent_audit
        .execute(PaginationParams { page: 1, per_page: 50 })
        .unwrap();
    let rows = page.data;
    let kinds: Vec<&str> = rows.iter().map(|a| a.event_type.as_str()).collect();
    for expected in [
        "client.created",
        "invoice.draft_created",
        "payment.recorded",
        "backup.created",
    ] {
        assert!(
            kinds.contains(&expected),
            "expected `{expected}` in audit log, got {kinds:?}",
        );
    }

    // Newest-first ordering: the backup happened last.
    assert_eq!(rows.first().map(|a| a.event_type.as_str()), Some("backup.created"));

    // Per-client scoping pulls in the client + invoice + payment rows
    // (the backup is org-wide, so it is *not* in the client view).
    let client_page = svc
        .paginate_audit_for_client
        .execute(client.id, PaginationParams { page: 1, per_page: 50 })
        .unwrap();
    let client_kinds: Vec<&str> = client_page
        .data
        .iter()
        .map(|a| a.event_type.as_str())
        .collect();
    assert!(client_kinds.contains(&"client.created"));
    assert!(client_kinds.contains(&"invoice.draft_created"));
    assert!(client_kinds.contains(&"payment.recorded"));
    assert!(
        !client_kinds.contains(&"backup.created"),
        "backups are org-wide, not client-scoped",
    );
}
