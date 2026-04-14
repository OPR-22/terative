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
    client_commands::{client_create, client_delete, client_get, client_list, client_update},
    data_commands::{data_backup, data_default_backup_dir, data_export, data_restore},
    email_commands::{
        email_test_connection, invoice_send, settings_update_email_config,
        settings_update_email_password,
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
    service_commands::{service_create, service_delete, service_list, service_update},
    settings_commands::{
        settings_get, settings_update_app_preferences, settings_update_currency,
        settings_update_seller_profile,
    },
    tax_commands::{tax_create, tax_delete, tax_list, tax_update},
    template_commands::{
        template_create, template_delete, template_duplicate, template_list, template_preview,
        template_set_default, template_update,
    },
    AppState,
};
use tauri::Manager;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = resolve_app_data_dir(app.handle());
            let db_path = data_dir.join("terative.sqlite");
            let db = adapters::sqlite::open(&db_path)
                .unwrap_or_else(|e| panic!("open sqlite at {db_path:?}: {e}"));
            seed_default_template_if_empty(&db);
            let default_pdf_dir = data_dir.join("invoices");
            let default_backup_dir = data_dir.join("backups");
            app.manage(AppState::new(
                db,
                db_path,
                default_pdf_dir,
                default_backup_dir,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            client_create,
            client_update,
            client_delete,
            client_list,
            client_get,
            service_create,
            service_update,
            service_delete,
            service_list,
            settings_get,
            settings_update_seller_profile,
            settings_update_currency,
            settings_update_app_preferences,
            tax_create,
            tax_update,
            tax_delete,
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
