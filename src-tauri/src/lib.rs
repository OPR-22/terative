pub mod adapters;
pub mod application;
pub mod commands;
pub mod domain;

use std::fs;
use std::path::PathBuf;

use commands::{
    accounting_commands::{
        accounting_aging_report, accounting_client_balance, accounting_client_balances,
        accounting_dashboard_summary, accounting_list_outstanding, accounting_list_overdue,
        accounting_revenue_by_client, accounting_revenue_by_period,
    },
    client_commands::{
        client_archive, client_create, client_get, client_list, client_unarchive, client_update,
    },
    data_commands::{data_backup, data_default_backup_dir, data_export, data_restore},
    email_commands::{
        email_test_connection, invoice_send, settings_update_email_config,
        settings_update_email_password,
    },
    email_template_commands::{
        email_template_create, email_template_delete, email_template_list,
        email_template_set_default, email_template_update,
    },
    invoice_commands::{
        invoice_cancel, invoice_create_draft, invoice_duplicate, invoice_finalize, invoice_get,
        invoice_list, invoice_update_draft,
    },
    notebook_commands::{
        client_notebook_get, client_notebook_save, journal_entry_create, journal_entry_delete,
        journal_entry_get, journal_entry_update, journal_list_for_client,
        notebook_section_count_entries, notebook_section_create, notebook_section_delete,
        notebook_section_list, notebook_section_rename, notebook_section_reorder,
    },
    payment_commands::{
        payment_delete, payment_get, payment_list, payment_record, payment_update,
    },
    catalog_item_commands::{
        catalog_item_archive, catalog_item_create, catalog_item_list, catalog_item_unarchive,
        catalog_item_update,
    },
    settings_commands::{
        settings_get, settings_supported_currencies, settings_update_app_preferences,
        settings_update_currency, settings_update_seller_profile,
    },
    tax_commands::{tax_archive, tax_create, tax_list, tax_unarchive, tax_update},
    template_commands::{
        template_create, template_delete, template_duplicate, template_list, template_preview,
        template_set_default, template_update,
    },
    AppState,
};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

fn resolve_app_data_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("resolve app data dir");
    fs::create_dir_all(&dir).expect("create app data dir");
    dir
}

/// First-launch seed: if the user has no templates yet, insert a default one
/// so finalize and preview work out of the box without requiring the user to
/// create a template first.
fn seed_default_template_if_empty(db: &adapters::sqlite::Db) {
    use application::ports::TemplateRepository;
    let repo = adapters::sqlite::SqliteTemplateRepository::new(db.clone());
    let existing = match repo.list() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("seed default template: list failed: {e}");
            return;
        }
    };
    if !existing.is_empty() {
        return;
    }
    let mut template = application::invoice_usecases::implicit_default_template();
    template.is_default = true;
    if let Err(e) = repo.insert(&template) {
        eprintln!("seed default template: insert failed: {e}");
    }
}

/// First-launch seed: if no email templates exist, insert default ones for
/// both template types so sending works out of the box.
fn seed_default_email_templates_if_empty(db: &adapters::sqlite::Db) {
    use application::ports::EmailTemplateRepository;
    use domain::email_template::{EmailTemplate, EmailTemplateType, NewEmailTemplate};
    let repo = adapters::sqlite::SqliteEmailTemplateRepository::new(db.clone());
    let existing = match repo.list() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("seed default email templates: list failed: {e}");
            return;
        }
    };
    if !existing.is_empty() {
        return;
    }
    let mut ic = EmailTemplate::create(NewEmailTemplate {
        name: "Default".into(),
        template_type: EmailTemplateType::InitialContact,
        subject_template: "Invoice {{number}} from {{seller_name}}".into(),
        body_template: "Hi {{client_name}},\n\nPlease find invoice {{number}} attached. Total: {{total}}.\n\n— {{seller_name}}".into(),
    }).unwrap();
    ic.is_default = true;
    if let Err(e) = repo.insert(&ic) {
        eprintln!("seed default email template (initial_contact): {e}");
    }
    let mut fu = EmailTemplate::create(NewEmailTemplate {
        name: "Default reminder".into(),
        template_type: EmailTemplateType::FollowUp,
        subject_template: "Reminder: Invoice {{number}}".into(),
        body_template: "Hi {{client_name}},\n\nThis is a friendly reminder regarding invoice {{number}} ({{total}}), due on {{due_date}}.\n\n— {{seller_name}}".into(),
    }).unwrap();
    fu.is_default = true;
    if let Err(e) = repo.insert(&fu) {
        eprintln!("seed default email template (follow_up): {e}");
    }
}

/// Builds the specta `Builder` with every registered command. Extracted so
/// that both `run()` and the bindings-export test can call it without
/// duplicating the command list.
fn build_specta() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        client_create,
        client_update,
        client_archive,
        client_unarchive,
        client_list,
        client_get,
        catalog_item_create,
        catalog_item_update,
        catalog_item_archive,
        catalog_item_unarchive,
        catalog_item_list,
        settings_get,
        settings_supported_currencies,
        settings_update_seller_profile,
        settings_update_currency,
        settings_update_app_preferences,
        tax_create,
        tax_update,
        tax_archive,
        tax_unarchive,
        tax_list,
        template_create,
        template_update,
        template_delete,
        template_duplicate,
        template_set_default,
        template_list,
        template_preview,
        invoice_create_draft,
        invoice_update_draft,
        invoice_finalize,
        invoice_duplicate,
        invoice_cancel,
        invoice_list,
        invoice_get,
        settings_update_email_config,
        settings_update_email_password,
        email_test_connection,
        invoice_send,
        email_template_create,
        email_template_update,
        email_template_delete,
        email_template_set_default,
        email_template_list,
        payment_record,
        payment_update,
        payment_delete,
        payment_list,
        payment_get,
        accounting_list_outstanding,
        accounting_list_overdue,
        accounting_revenue_by_period,
        accounting_revenue_by_client,
        accounting_client_balance,
        accounting_client_balances,
        accounting_aging_report,
        accounting_dashboard_summary,
        data_export,
        data_backup,
        data_restore,
        data_default_backup_dir,
        notebook_section_create,
        notebook_section_rename,
        notebook_section_delete,
        notebook_section_count_entries,
        notebook_section_reorder,
        notebook_section_list,
        client_notebook_get,
        client_notebook_save,
        journal_entry_create,
        journal_entry_update,
        journal_entry_delete,
        journal_list_for_client,
        journal_entry_get,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = build_specta();

    // Export TS bindings in debug builds so the frontend always has a fresh
    // contract. Release builds skip this entirely.
    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/ipc/bindings.ts",
        )
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let data_dir = resolve_app_data_dir(app.handle());
            let db_path = data_dir.join("terative.sqlite");
            let db = adapters::sqlite::open(&db_path)
                .unwrap_or_else(|e| panic!("open sqlite at {db_path:?}: {e}"));
            seed_default_template_if_empty(&db);
            seed_default_email_templates_if_empty(&db);
            let default_pdf_dir = data_dir.join("invoices");
            let default_backup_dir = data_dir.join("backups");
            app.manage(AppState::new(
                db,
                db_path,
                default_pdf_dir,
                default_backup_dir,
            ));
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod specta_bindings {
    use super::*;

    /// Regenerates `src/ipc/bindings.ts` on every `cargo test` run. Keeps the
    /// frontend contract in sync with the Rust command set without needing
    /// to launch the full app.
    #[test]
    fn export_typescript_bindings() {
        build_specta()
            .export(
                specta_typescript::Typescript::default(),
                "../src/ipc/bindings.ts",
            )
            .expect("failed to export typescript bindings");
    }
}
