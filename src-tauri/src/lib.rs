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
    client_commands::client_attribute_values,
    bookmark_commands::{
        bookmark_create, bookmark_delete, bookmark_layout_set_bounds,
        bookmark_layout_set_sidebar_width, bookmark_layout_set_toolbar_height, bookmark_list,
        bookmark_nav_back, bookmark_nav_forward, bookmark_nav_hide, bookmark_nav_open,
        bookmark_nav_reload, bookmark_nav_to, bookmark_reorder, bookmark_update,
    },
    data_commands::{
        data_backup, data_delete_backup, data_export, data_list_backups, data_restore,
        data_user_backup_dir,
    },
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
        client_attribute_values,
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
        data_list_backups,
        data_delete_backup,
        data_user_backup_dir,
        bookmark_nav_open,
        bookmark_nav_hide,
        bookmark_nav_to,
        bookmark_nav_reload,
        bookmark_nav_back,
        bookmark_nav_forward,
        bookmark_layout_set_bounds,
        bookmark_layout_set_sidebar_width,
        bookmark_layout_set_toolbar_height,
        bookmark_list,
        bookmark_create,
        bookmark_update,
        bookmark_delete,
        bookmark_reorder,
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
            let default_pdf_dir = data_dir.join("invoices");
            let backups_root = data_dir.join("backups");
            let user_backup_dir = backups_root.join("user");
            let system_backup_dir = backups_root.join("system");

            // Snapshot BEFORE opening: if migrations are pending, the open()
            // below will apply them in-place, so the pre-migration state has
            // to be captured first.
            match adapters::sqlite::snapshot_pre_migration_if_pending(
                &db_path,
                &system_backup_dir,
            ) {
                Ok(Some(path)) => {
                    eprintln!("pre-migration backup written to {}", path.display());
                }
                Ok(None) => {}
                Err(e) => {
                    panic!("pre-migration backup failed at {db_path:?}: {e}");
                }
            }

            let db = adapters::sqlite::open(&db_path)
                .unwrap_or_else(|e| panic!("open sqlite at {db_path:?}: {e}"));
            seed_default_template_if_empty(&db);
            seed_default_email_templates_if_empty(&db);
            app.manage(AppState::new(
                db,
                db_path,
                default_pdf_dir,
                user_backup_dir,
                system_backup_dir,
            ));
            builder.mount_events(app);
            spawn_auto_backup_ticker(app.handle().clone());
            #[cfg(target_os = "linux")]
            wrap_main_webview_in_gtk_fixed(app.handle());
            #[cfg(not(target_os = "linux"))]
            register_main_window_resize_handler(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Fires an initial `auto_backup_if_due` check and then re-checks every 15
/// minutes while the app is running. The 15-min tick is deliberately shorter
/// than the configurable auto-backup interval so users who leave the app
/// open for days (common on macOS) still get their daily backups, and a
/// post-sleep tick catches up without needing to fire at the exact moment.
/// A plain std thread avoids pulling tokio in as a direct dependency.
fn spawn_auto_backup_ticker(app: tauri::AppHandle) {
    use std::time::Duration;
    use tauri::Manager;
    const TICK: Duration = Duration::from_secs(15 * 60);

    std::thread::spawn(move || loop {
        let dm = app.state::<commands::AppState>().data_management.clone();
        if let Err(e) = dm.auto_backup_if_due() {
            eprintln!("auto-backup check failed: {e}");
        }
        std::thread::sleep(TICK);
    });
}

/// On Linux, `GtkBox` packing fights the webkit2gtk widget's natural sizing —
/// we can't reliably constrain the main webview to a fixed width when sharing
/// a box with a bookmark child webview. The wry-recommended workaround is to
/// host both webviews inside a `GtkFixed` container that uses absolute
/// positioning. This function reparents the main webview from the default
/// vbox into a `GtkFixed`; subsequent webviews (bookmarks) get placed in the
/// same `GtkFixed` via their own `build_gtk` calls.
///
/// Called once at app setup, before any child webview is created.
#[cfg(target_os = "linux")]
fn wrap_main_webview_in_gtk_fixed(app: &tauri::AppHandle) {
    use gtk::prelude::{BoxExt, ContainerExt, FixedExt, ObjectExt, WidgetExt};
    use tauri::Manager;

    let Some(main) = app.get_webview_window("main") else {
        eprintln!("[fixed-setup] main window not found");
        return;
    };
    let vbox = match main.as_ref().window().default_vbox() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[fixed-setup] default_vbox err: {e}");
            return;
        }
    };

    // Locate the main webview widget among the vbox's children.
    let main_widget = vbox
        .children()
        .into_iter()
        .find(|w| w.type_().name() == "WebKitWebView");
    let Some(main_widget) = main_widget else {
        eprintln!("[fixed-setup] main webview widget not found in vbox");
        return;
    };

    let fixed = gtk::Fixed::new();
    fixed.show();

    // Reparent: keep a strong ref so the widget survives between remove + add.
    let widget_ref = main_widget.clone();
    vbox.remove(&main_widget);
    fixed.put(&widget_ref, 0, 0);

    // The fixed container expands to fill the vbox; its children are sized
    // explicitly via set_size_request (wry's gtk_multiwebview example pattern).
    vbox.pack_start(&fixed, true, true, 0);

    // Initial main-webview size: use the window's physical inner dimensions.
    // GTK size_allocate operates in physical px here; converting to logical
    // would undersize the widget on fractional-DPI displays.
    let (win_w, win_h) = main.as_ref().window().inner_size().map(|s| {
        (s.width as i32, s.height as i32)
    }).unwrap_or((800, 600));
    widget_ref.size_allocate(&gtk::Allocation::new(0, 0, win_w, win_h));

    // Keep webview sizes in sync with the window. GTK signal callbacks run
    // on the main thread, so capturing gtk widgets is safe (no Send needed).
    // The sidebar's CSS width is owned by React (it can collapse/expand) and
    // pulled live from the cached value.
    let Ok(gtk_window) = main.as_ref().window().gtk_window() else { return };
    let fixed_for_resize = fixed.clone();
    let main_widget_for_resize = widget_ref.clone();
    gtk_window.connect_size_allocate(move |_win, alloc| {
        use gtk::prelude::{Cast, ContainerExt, WidgetExt};
        let w = alloc.width();
        let h = alloc.height();
        let scale = commands::bookmark_commands::current_dpr();
        // Both sidebar width and toolbar height come from React. Until both
        // are known we just size main to the full window.
        let (Some(sidebar_css), Some(toolbar_css)) = (
            commands::bookmark_commands::current_sidebar_width_css(),
            commands::bookmark_commands::current_toolbar_height_css(),
        ) else {
            main_widget_for_resize.size_allocate(&gtk::Allocation::new(0, 0, w, h));
            return;
        };
        let sidebar_px = (sidebar_css * scale).round() as i32;
        let toolbar_px = (toolbar_css * scale).round() as i32;
        let main_w = main_widget_for_resize.upcast_ref::<gtk::Widget>();
        // Find a visible widget tagged `bookmark:<id>` (active bookmark page).
        let visible_bookmark = fixed_for_resize.children().into_iter().find(|c| {
            c != main_w
                && c.get_visible()
                && c.widget_name().as_str().starts_with("bookmark:")
        });
        if let Some(bookmark) = visible_bookmark {
            let right_w = (w - sidebar_px).max(1);
            let bookmark_h = (h - toolbar_px).max(1);
            main_widget_for_resize
                .size_allocate(&gtk::Allocation::new(0, 0, sidebar_px, h));
            // Toolbar widget tagged `bookmark-toolbar`.
            if let Some(toolbar) = fixed_for_resize
                .children()
                .into_iter()
                .find(|c| c.widget_name().as_str() == "bookmark-toolbar")
            {
                toolbar.size_allocate(&gtk::Allocation::new(
                    sidebar_px, 0, right_w, toolbar_px,
                ));
            }
            bookmark.size_allocate(&gtk::Allocation::new(
                sidebar_px,
                toolbar_px,
                right_w,
                bookmark_h,
            ));
        } else {
            main_widget_for_resize.size_allocate(&gtk::Allocation::new(0, 0, w, h));
        }
    });
}

/// macOS/Windows equivalent of the Linux GTK `size_allocate` handler. Tauri's
/// child webviews (the bookmark toolbar + bookmark page) are absolute-
/// positioned overlays that don't auto-resize with the window, so we re-apply
/// the bookmark layout on every `WindowEvent::Resized`. No-op when no
/// bookmark is currently active (the overlays are hidden in that case).
#[cfg(not(target_os = "linux"))]
fn register_main_window_resize_handler(app: &tauri::AppHandle) {
    let Some(main) = app.get_webview_window("main") else {
        eprintln!("[resize-handler] main window not found");
        return;
    };
    let app_handle = app.clone();
    main.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Resized(_)) {
            commands::bookmark_commands::apply_active_bookmark_layout(&app_handle);
        }
    });
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
