use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;
use tauri::webview::{NewWindowResponse, WebviewBuilder, WebviewWindowBuilder};
use tauri::{LogicalPosition, LogicalSize, Manager, Url, WebviewUrl};
#[cfg(not(target_os = "linux"))]
use tauri::Rect;
use uuid::Uuid;

use crate::application::AppError;

/// Monotonic counter for unique popup-window labels. Each popup spawned by
/// `window.open` / target=_blank gets a fresh `bookmark-popup-N` label.
static POPUP_LABEL_SEQ: AtomicU64 = AtomicU64::new(0);

/// Last-known device-pixel ratio. Tauri's `window.scale_factor()` reports
/// integer-only on some Wayland setups where the compositor does fractional
/// scaling; we read the real DPR from the frontend (window.devicePixelRatio)
/// and cache it for use by the GTK size-allocate handler.
#[cfg(target_os = "linux")]
static DPR_BITS: AtomicU64 = AtomicU64::new(0x3ff0_0000_0000_0000);

#[cfg(target_os = "linux")]
pub fn current_dpr() -> f64 {
    f64::from_bits(DPR_BITS.load(Ordering::Relaxed))
}

/// Sidebar width in CSS px. React is the source of truth (it can collapse/
/// expand the sidebar) and notifies us via `set_sidebar_width`. No hardcoded
/// fallback — frontend must report before the first bookmark opens.
static SIDEBAR_WIDTH_CSS: Mutex<Option<f64>> = Mutex::new(None);

pub fn current_sidebar_width_css() -> Option<f64> {
    *SIDEBAR_WIDTH_CSS.lock()
}

/// Toolbar height in CSS px. React reports via `set_toolbar_height` (the
/// bootstrap hook on app mount, plus the toolbar webview re-measures itself
/// after it actually renders).
static TOOLBAR_HEIGHT_CSS: Mutex<Option<f64>> = Mutex::new(None);

pub fn current_toolbar_height_css() -> Option<f64> {
    *TOOLBAR_HEIGHT_CSS.lock()
}

fn current_toolbar_height_css_required() -> Result<f64, AppError> {
    current_toolbar_height_css().ok_or_else(|| AppError::Unknown {
        detail: "toolbar height not set; the frontend bootstrap must call set_toolbar_height first"
            .to_string(),
    })
}

/// Label of the currently-shown bookmark, if any. Updated on `bookmark_open`
/// and cleared on `bookmark_hide`. Used by `set_sidebar_width` /
/// `set_toolbar_height` and the window-resize handler to know which bookmark
/// to re-layout.
static ACTIVE_BOOKMARK: Mutex<Option<String>> = Mutex::new(None);

/// Each bookmark gets its own webview, labelled `bookmark:<id>`. The webview
/// stays alive for the app's lifetime — hidden when the user navigates away,
/// re-shown when they come back — so each bookmark keeps its own history,
/// cookies, scroll position, and JS state independently.
const BOOKMARK_LABEL_PREFIX: &str = "bookmark:";

/// A single shared webview hosting the bookmark navigation toolbar (back/
/// forward/reload/home). Re-navigated to the active bookmark's id each time
/// the user switches bookmarks. Only ONE toolbar webview exists for the
/// app's lifetime; sized to a thin strip above the active bookmark.
const TOOLBAR_LABEL: &str = "bookmark-toolbar";

fn label_for(id: &str) -> String {
    format!("{BOOKMARK_LABEL_PREFIX}{id}")
}

fn is_bookmark_label(label: &str) -> bool {
    label.starts_with(BOOKMARK_LABEL_PREFIX)
}

/// True for any webview created by the bookmark feature — the per-URL
/// `bookmark:*` content webviews and the shared `bookmark-toolbar`.
/// Used by `close_all_bookmark_webviews` only; layout/hide logic uses
/// the narrower `is_bookmark_label`.
fn is_bookmark_owned_label(label: &str) -> bool {
    is_bookmark_label(label) || label == TOOLBAR_LABEL
}

/// Namespace UUID for deriving per-(org, bookmark) data-store identifiers.
/// Picked once at design time and never changes — switching it would
/// orphan every persisted WKWebsiteDataStore on macOS users' disks.
const TERATIVE_WEBVIEW_NAMESPACE: Uuid = Uuid::from_bytes([
    0x54, 0x45, 0x52, 0x41, 0x42, 0x4f, 0x4f, 0x4b,
    0x4d, 0x41, 0x52, 0x4b, 0x57, 0x45, 0x42, 0x56,
]);

/// Stable 16-byte identifier for a (org, bookmark) pair. Used as
/// `WKWebsiteDataStore` identifier on macOS — the WebView-level analogue
/// of a data directory on platforms that don't expose disk paths for
/// per-webview storage. Derived deterministically so reopening the same
/// org+bookmark always lands in the same storage partition.
fn webview_data_store_id(org_code: &str, bookmark_id: &str) -> [u8; 16] {
    let name = format!("{org_code}/{bookmark_id}");
    *Uuid::new_v5(&TERATIVE_WEBVIEW_NAMESPACE, name.as_bytes()).as_bytes()
}

/// Closes every bookmark + toolbar webview. Called when the active org
/// changes so the next bookmark open is built against the new org's
/// `data_directory` / `data_store_identifier`.
///
/// Webviews persist for the app's lifetime by design; the only time they
/// get torn down outside this path is window close.
pub(crate) fn close_all_bookmark_webviews(app: &tauri::AppHandle) {
    *ACTIVE_BOOKMARK.lock() = None;
    let labels: Vec<String> = app
        .webviews()
        .keys()
        .filter(|l| is_bookmark_owned_label(l))
        .cloned()
        .collect();
    for label in labels {
        if let Some(w) = app.get_webview(&label) {
            let _ = w.close();
        }
    }
}

fn parse_url(s: &str) -> Result<Url, AppError> {
    let parsed: Url =
        Url::parse(s).map_err(|_| AppError::from(crate::domain::bookmark::BookmarkError::InvalidUrl))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        _ => Err(AppError::from(
            crate::domain::bookmark::BookmarkError::UnsupportedScheme,
        )),
    }
}

/// Lay out the toolbar and active bookmark webviews from the cached sidebar
/// width + toolbar height + current window size. Idempotent — safe to call
/// from any code path that changes one of those inputs (sidebar collapse,
/// toolbar re-measure, window resize, bookmark open).
///
/// Layout: `[main | toolbar]` over `[main | bookmark]`, with the main webview
/// hosting the React sidebar in its left strip.
///
/// Linux uses GTK widget reparenting into a `GtkFixed`; macOS/Windows use
/// Tauri's native `set_position`/`set_size` on the overlay child webviews.
#[cfg(target_os = "linux")]
fn apply_bookmark_layout(app: &tauri::AppHandle, bookmark_label: &str) {
    eprintln!("[place] entry for {bookmark_label}");
    let Some(main_webview) = app.get_webview("main") else {
        eprintln!("[place] main webview missing");
        return;
    };
    let app_cl = app.clone();
    let bookmark_label = bookmark_label.to_string();
    let _ = main_webview.with_webview(move |_pw| {
        use gtk::prelude::{Cast, ContainerExt, FixedExt, ObjectExt, WidgetExt};

        eprintln!("[place] closure running for {bookmark_label}");
        let Some(main_webview) = app_cl.get_webview("main") else {
            eprintln!("[place] main webview gone in closure");
            return;
        };
        let Ok(vbox) = main_webview.window().default_vbox() else {
            eprintln!("[place] vbox lookup failed");
            return;
        };
        let vbox_kids: Vec<_> = vbox
            .children()
            .into_iter()
            .map(|c| (c.type_().name().to_string(), c.widget_name().as_str().to_string()))
            .collect();
        eprintln!("[place] vbox children: {vbox_kids:?}");
        let Some(fixed): Option<gtk::Fixed> = vbox
            .children()
            .into_iter()
            .find_map(|c| c.downcast::<gtk::Fixed>().ok())
        else {
            eprintln!("[place] no GtkFixed in vbox");
            return;
        };
        let fixed_kids: Vec<_> = fixed
            .children()
            .into_iter()
            .map(|c| (c.type_().name().to_string(), c.widget_name().as_str().to_string()))
            .collect();
        eprintln!("[place] fixed children: {fixed_kids:?}");

        let Ok(win_size) = main_webview.window().inner_size() else { return };
        let win_w = win_size.width as i32;
        let win_h = win_size.height as i32;
        let dpr = current_dpr();
        let Some(sidebar_css) = current_sidebar_width_css() else {
            eprintln!("[place] sidebar width not yet provided by frontend; skipping layout");
            return;
        };
        let Some(toolbar_css) = current_toolbar_height_css() else {
            eprintln!("[place] toolbar height not yet provided by frontend; skipping layout");
            return;
        };
        let sidebar_px = (sidebar_css * dpr).round() as i32;
        let toolbar_px = (toolbar_css * dpr).round() as i32;
        let right_w = (win_w - sidebar_px).max(1);
        let bookmark_h = (win_h - toolbar_px).max(1);

        // GTK3's default widget_name is the widget's type name (e.g.
        // "WebKitWebView"), NOT an empty string — that's how we distinguish
        // an untagged widget from one we've already claimed.
        let is_untagged = |w: &gtk::Widget| w.widget_name().as_str() == w.type_().name();

        // Helper: find a webview widget tagged with `name` in the fixed, or
        // claim the next untagged WebKitWebView from vbox (just-created via
        // add_child) and reparent + tag it.
        let claim_widget = |name: &str| -> Option<gtk::Widget> {
            if let Some(w) = fixed
                .children()
                .into_iter()
                .find(|c| c.widget_name().as_str() == name)
            {
                return Some(w);
            }
            let candidate = vbox
                .children()
                .into_iter()
                .find(|c| c.type_().name() == "WebKitWebView" && is_untagged(c))?;
            if let Some(parent) = candidate.parent() {
                if let Ok(container) = parent.downcast::<gtk::Container>() {
                    container.remove(&candidate);
                }
            }
            fixed.put(&candidate, 0, 0); // overridden by size_allocate below
            candidate.set_widget_name(name);
            Some(candidate)
        };

        // Main webview is already in the fixed (since startup), and untagged.
        if let Some(main_widget) = fixed
            .children()
            .into_iter()
            .find(|w| w.type_().name() == "WebKitWebView" && is_untagged(w))
        {
            main_widget.size_allocate(&gtk::Allocation::new(0, 0, sidebar_px, win_h));
        }

        if let Some(toolbar_widget) = claim_widget(TOOLBAR_LABEL) {
            fixed.move_(&toolbar_widget, sidebar_px, 0);
            toolbar_widget.size_allocate(&gtk::Allocation::new(
                sidebar_px,
                0,
                right_w,
                toolbar_px,
            ));
            toolbar_widget.show();
        }

        if let Some(bookmark_widget) = claim_widget(&bookmark_label) {
            fixed.move_(&bookmark_widget, sidebar_px, toolbar_px);
            bookmark_widget.size_allocate(&gtk::Allocation::new(
                sidebar_px,
                toolbar_px,
                right_w,
                bookmark_h,
            ));
            bookmark_widget.show();
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn apply_bookmark_layout(app: &tauri::AppHandle, bookmark_label: &str) {
    // We can't use `get_webview_window("main")` here: that only returns Some
    // when every webview in the window shares the window's label, which stops
    // being true the moment we `add_child` the bookmark/toolbar webviews. Go
    // through the main webview and pull its window instead.
    let Some(main_webview) = app.get_webview("main") else {
        eprintln!("[bookmark-layout] main webview missing");
        return;
    };
    let main_window = main_webview.window();
    let Ok(scale) = main_window.scale_factor() else {
        eprintln!("[bookmark-layout] scale_factor query failed");
        return;
    };
    let Ok(physical) = main_window.inner_size() else {
        eprintln!("[bookmark-layout] inner_size query failed");
        return;
    };
    let logical: tauri::LogicalSize<f64> = physical.to_logical(scale);
    let win_w = logical.width;
    let win_h = logical.height;

    let Some(sidebar) = current_sidebar_width_css() else {
        eprintln!("[bookmark-layout] sidebar width not yet set by frontend");
        return;
    };
    let Some(toolbar_h) = current_toolbar_height_css() else {
        eprintln!("[bookmark-layout] toolbar height not yet set by frontend");
        return;
    };

    let right_w = (win_w - sidebar).max(1.0);
    let bookmark_h = (win_h - toolbar_h).max(1.0);

    // Use `set_bounds` for an atomic position+size update. Calling
    // `set_position` then `set_size` separately routes through two IPC
    // messages and produces a transient frame at the wrong size, which on
    // macOS WKWebView can leave the child webview in a stale layout.
    let toolbar_rect = Rect {
        position: LogicalPosition::new(sidebar, 0.0).into(),
        size: LogicalSize::new(right_w, toolbar_h).into(),
    };
    let bookmark_rect = Rect {
        position: LogicalPosition::new(sidebar, toolbar_h).into(),
        size: LogicalSize::new(right_w, bookmark_h).into(),
    };

    if let Some(toolbar) = app.get_webview(TOOLBAR_LABEL) {
        if let Err(e) = toolbar.set_bounds(toolbar_rect) {
            eprintln!("[bookmark-layout] toolbar set_bounds failed: {e}");
        }
    } else {
        eprintln!("[bookmark-layout] toolbar webview not found");
    }
    if let Some(bookmark) = app.get_webview(bookmark_label) {
        if let Err(e) = bookmark.set_bounds(bookmark_rect) {
            eprintln!("[bookmark-layout] bookmark set_bounds failed: {e}");
        }
    } else {
        eprintln!("[bookmark-layout] bookmark webview '{bookmark_label}' not found");
    }
}

/// Re-applies layout for whichever bookmark is currently visible (if any).
/// No-op when no bookmark is shown — the toolbar and bookmark webviews are
/// hidden in that case and don't need positioning.
pub fn apply_active_bookmark_layout(app: &tauri::AppHandle) {
    let Some(label) = ACTIVE_BOOKMARK.lock().clone() else {
        return;
    };
    apply_bookmark_layout(app, &label);
}

/// Hides every bookmark webview except the one with `keep_label` (pass an
/// empty str to hide them all). Used to ensure only one bookmark is visible
/// at a time while preserving each webview's state.
fn hide_other_bookmarks(app: &tauri::AppHandle, keep_label: &str) {
    for (label, webview) in app.webviews() {
        if is_bookmark_label(&label) && label != keep_label {
            let _ = webview.hide();
        }
    }
}

/// Open or navigate the shared toolbar webview to point at the given
/// bookmark id. Created lazily on first call. Reused thereafter — we just
/// `eval` a `location.assign` to swap routes (cheap, the toolbar UI is tiny).
fn ensure_toolbar_webview(
    app: &tauri::AppHandle,
    bookmark_id: &str,
    initial_pos: LogicalPosition<f64>,
    initial_size: LogicalSize<f64>,
) -> Result<(), AppError> {
    let route = format!("bookmark-toolbar/{bookmark_id}");
    if let Some(existing) = app.get_webview(TOOLBAR_LABEL) {
        existing
            .eval(&format!("window.location.assign('/{route}')"))
            .map_err(|e: tauri::Error| e.to_string())?;
        existing
            .show()
            .map_err(|e: tauri::Error| e.to_string())?;
        return Ok(());
    }
    let main_webview = app
        .get_webview("main")
        .ok_or_else(|| "main webview not found".to_string())?;
    let builder =
        WebviewBuilder::new(TOOLBAR_LABEL, WebviewUrl::App(route.into()));
    let toolbar = main_webview
        .window()
        .add_child(builder, initial_pos, initial_size)
        .map_err(|e: tauri::Error| e.to_string())?;
    toolbar
        .show()
        .map_err(|e: tauri::Error| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_nav_open(
    app: tauri::AppHandle,
    state: tauri::State<'_, super::AppState>,
    id: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    dpr: f64,
) -> Result<(), AppError> {
    let org_code = state.active_code().ok_or_else(AppError::no_active_org)?;
    let parsed = parse_url(&url)?;
    let label = label_for(&id);
    hide_other_bookmarks(&app, &label);
    *ACTIVE_BOOKMARK.lock() = Some(label.clone());
    #[cfg(target_os = "linux")]
    {
        DPR_BITS.store(dpr.to_bits(), Ordering::Relaxed);
    }
    #[cfg(not(target_os = "linux"))]
    let _ = dpr;

    // The frontend is the source of truth for layout dimensions. React must
    // call `set_toolbar_height` (and `set_sidebar_width`) before opening any
    // bookmark; otherwise we have no size to give `add_child` and refuse.
    let toolbar_h = current_toolbar_height_css_required()?;

    ensure_toolbar_webview(
        &app,
        &id,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, toolbar_h),
    )?;

    let bookmark_y = y + toolbar_h;
    let bookmark_h = (height - toolbar_h).max(1.0);

    if let Some(existing) = app.get_webview(&label) {
        let _ = parsed;
        existing
            .show()
            .map_err(|e: tauri::Error| e.to_string())?;
        apply_bookmark_layout(&app, &label);
        return Ok(());
    }

    let main_webview = app
        .get_webview("main")
        .ok_or_else(|| "main webview not found".to_string())?;

    // Per-org webview storage: cookies, localStorage, and service-worker
    // caches all live under <orgs_root>/<code>/webviews/<bookmark_id>/.
    // Different orgs → different login sessions for the same site.
    let data_dir: PathBuf = state.org_registry.bookmark_webview_dir(&org_code, &id);
    std::fs::create_dir_all(&data_dir).map_err(AppError::from)?;
    let store_id = webview_data_store_id(org_code.as_str(), &id);

    // Native engine-level callback for `window.open` and `target=_blank`
    // links. Tauri forwards this to wry's `with_new_window_req_handler`,
    // which fires inside WKWebView/webkit2gtk/WebView2 *before* the popup
    // is created. We host the popup in a fresh top-level Tauri window so
    // OAuth flows (window.opener / postMessage round-trips) keep working —
    // `window_features(features)` carries the platform-linking glue
    // (related_view on Linux, environment on Windows, webview_configuration
    // on macOS) that wry needs to keep the popup associated with its opener.
    //
    // Popups MUST share storage with the parent bookmark webview — OAuth
    // callbacks need to read/write cookies in the parent's partition.
    let app_for_popups = app.clone();
    let popup_data_dir = data_dir.clone();
    let popup_store_id = store_id;
    let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed))
        .data_directory(data_dir.clone())
        .data_store_identifier(store_id)
        .on_new_window(move |target_url: tauri::Url, features| {
            let n = POPUP_LABEL_SEQ.fetch_add(1, Ordering::Relaxed);
            let popup_label = format!("bookmark-popup-{n}");
            // Show the host as the initial title so the OS window doesn't
            // briefly read "Tauri App" while the page is loading. Once the
            // page sets its own document.title, mirror that to the window.
            let initial_title = target_url
                .host_str()
                .unwrap_or("")
                .to_string();
            match WebviewWindowBuilder::new(
                &app_for_popups,
                popup_label,
                WebviewUrl::External(target_url),
            )
            .data_directory(popup_data_dir.clone())
            .data_store_identifier(popup_store_id)
            .window_features(features)
            .title(initial_title)
            .on_document_title_changed(|window, title| {
                let _ = window.set_title(&title);
            })
            .build()
            {
                Ok(window) => NewWindowResponse::Create { window },
                Err(e) => {
                    eprintln!("[bookmark] popup window build failed: {e}");
                    NewWindowResponse::Deny
                }
            }
        });
    let bookmark = main_webview
        .window()
        .add_child(
            builder,
            LogicalPosition::new(x, bookmark_y),
            LogicalSize::new(width, bookmark_h),
        )
        .map_err(|e: tauri::Error| e.to_string())?;
    bookmark
        .show()
        .map_err(|e: tauri::Error| e.to_string())?;

    apply_bookmark_layout(&app, &label);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_layout_set_bounds(
    app: tauri::AppHandle,
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    dpr: f64,
) -> Result<(), AppError> {
    let label = label_for(&id);
    let _ = (x, y, width, height); // bounds are derived from the cached sidebar/toolbar/window dims
    #[cfg(target_os = "linux")]
    {
        DPR_BITS.store(dpr.to_bits(), Ordering::Relaxed);
    }
    #[cfg(not(target_os = "linux"))]
    let _ = dpr;
    apply_bookmark_layout(&app, &label);
    Ok(())
}

/// Force the bookmark to navigate to a URL (typically the home URL via the
/// refresh button in the sidebar).
#[tauri::command]
#[specta::specta]
pub fn bookmark_nav_to(
    app: tauri::AppHandle,
    id: String,
    url: String,
) -> Result<(), AppError> {
    let parsed = parse_url(&url)?;
    let Some(webview) = app.get_webview(&label_for(&id)) else {
        return Ok(());
    };
    webview
        .navigate(parsed)
        .map_err(|e: tauri::Error| e.to_string())?;
    Ok(())
}

/// Reload the bookmark's current page (toolbar reload button).
#[tauri::command]
#[specta::specta]
pub fn bookmark_nav_reload(app: tauri::AppHandle, id: String) -> Result<(), AppError> {
    if let Some(webview) = app.get_webview(&label_for(&id)) {
        webview
            .reload()
            .map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

/// Step the bookmark's session history backward (toolbar back button).
/// Tauri's Webview doesn't expose `go_back` directly, so we evaluate the
/// standard `history.back()` JS API in the bookmark's webview.
#[tauri::command]
#[specta::specta]
pub fn bookmark_nav_back(app: tauri::AppHandle, id: String) -> Result<(), AppError> {
    if let Some(webview) = app.get_webview(&label_for(&id)) {
        webview
            .eval("history.back()")
            .map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_nav_forward(app: tauri::AppHandle, id: String) -> Result<(), AppError> {
    if let Some(webview) = app.get_webview(&label_for(&id)) {
        webview
            .eval("history.forward()")
            .map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

/// React-side notification of the sidebar's current width (CSS px). React is
/// the source of truth — it tells us on app boot and again whenever the
/// sidebar collapses/expands. We cache the value and, if a bookmark is
/// currently shown, re-run layout so the bookmark reclaims/yields space.
#[tauri::command]
#[specta::specta]
pub fn bookmark_layout_set_sidebar_width(
    app: tauri::AppHandle,
    width: f64,
) -> Result<(), AppError> {
    *SIDEBAR_WIDTH_CSS.lock() = Some(width);
    apply_active_bookmark_layout(&app);
    Ok(())
}

/// React-side notification of the toolbar's measured height (CSS px). The
/// bookmark-toolbar webview measures itself after mount and reports here.
/// Triggers a re-layout if a bookmark is active.
#[tauri::command]
#[specta::specta]
pub fn bookmark_layout_set_toolbar_height(
    app: tauri::AppHandle,
    height: f64,
) -> Result<(), AppError> {
    *TOOLBAR_HEIGHT_CSS.lock() = Some(height);
    apply_active_bookmark_layout(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_nav_hide(app: tauri::AppHandle) -> Result<(), AppError> {
    hide_other_bookmarks(&app, "");
    if let Some(toolbar) = app.get_webview(TOOLBAR_LABEL) {
        toolbar
            .hide()
            .map_err(|e: tauri::Error| e.to_string())?;
    }
    *ACTIVE_BOOKMARK.lock() = None;
    Ok(())
}

// ---- CRUD over the bookmarks list (manageable in Settings) ----

use crate::application::dto::{BookmarkDto, NewBookmarkDto, UpdateBookmarkDto};
use crate::domain::bookmark::BookmarkId;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn bookmark_list(
    state: State<'_, super::AppState>,
) -> Result<Vec<BookmarkDto>, AppError> {
    state.org()?
        .list_bookmarks
        .execute()
        .map(|list| list.iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_create(
    state: State<'_, super::AppState>,
    input: NewBookmarkDto,
) -> Result<BookmarkDto, AppError> {
    state.org()?
        .create_bookmark
        .execute(input.into())
        .map(|b| (&b).into())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_update(
    state: State<'_, super::AppState>,
    input: UpdateBookmarkDto,
) -> Result<BookmarkDto, AppError> {
    state.org()?
        .update_bookmark
        .execute(input.into())
        .map(|b| (&b).into())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_delete(
    state: State<'_, super::AppState>,
    id: Uuid,
) -> Result<(), AppError> {
    state.org()?
        .delete_bookmark
        .execute(BookmarkId(id))
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_reorder(
    state: State<'_, super::AppState>,
    ordered_ids: Vec<Uuid>,
) -> Result<(), AppError> {
    state.org()?
        .reorder_bookmarks
        .execute(ordered_ids.into_iter().map(BookmarkId).collect())
}
