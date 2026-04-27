use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;
use tauri::webview::WebviewBuilder;
use tauri::{LogicalPosition, LogicalSize, Manager, Url, WebviewUrl};

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

fn current_toolbar_height_css_required() -> Result<f64, String> {
    current_toolbar_height_css().ok_or_else(|| {
        "toolbar height not set; the frontend bootstrap must call set_toolbar_height first"
            .to_string()
    })
}

/// Label of the currently-shown bookmark, if any. Updated on `bookmark_open`
/// and cleared on `bookmark_hide`. Used by `set_sidebar_width` /
/// `set_toolbar_height` to know which bookmark to re-layout.
#[cfg(target_os = "linux")]
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

fn parse_url(s: &str) -> Result<Url, String> {
    let parsed: Url = Url::parse(s).map_err(|e| e.to_string())?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        other => Err(format!("unsupported URL scheme: {other}")),
    }
}

/// Linux-only: lay out [main | toolbar | bookmark] inside the GtkFixed.
///
/// - main webview: left strip, sidebar width, full height.
/// - toolbar webview: top-right strip, full remaining width × TOOLBAR_HEIGHT.
/// - bookmark webview: bottom-right, full remaining width × remaining height.
///
/// All three are non-overlapping rectangles inside the fixed. The toolbar
/// and bookmark widgets are reparented from Tauri's default vbox into the
/// fixed on first use, then tagged with their labels via `widget_name` so
/// subsequent calls find them.
#[cfg(target_os = "linux")]
fn place_bookmark_in_fixed(app: &tauri::AppHandle, bookmark_label: &str) {
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
fn place_bookmark_in_fixed(_app: &tauri::AppHandle, _bookmark_label: &str) {}

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
) -> Result<(), String> {
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
pub fn bookmark_open(
    app: tauri::AppHandle,
    id: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    dpr: f64,
) -> Result<(), String> {
    let parsed = parse_url(&url)?;
    let label = label_for(&id);
    hide_other_bookmarks(&app, &label);
    #[cfg(target_os = "linux")]
    {
        DPR_BITS.store(dpr.to_bits(), Ordering::Relaxed);
        *ACTIVE_BOOKMARK.lock() = Some(label.clone());
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
        #[cfg(target_os = "linux")]
        place_bookmark_in_fixed(&app, &label);
        #[cfg(not(target_os = "linux"))]
        {
            existing
                .set_position(LogicalPosition::new(x, bookmark_y))
                .map_err(|e: tauri::Error| e.to_string())?;
            existing
                .set_size(LogicalSize::new(width, bookmark_h))
                .map_err(|e: tauri::Error| e.to_string())?;
        }
        return Ok(());
    }

    let main_webview = app
        .get_webview("main")
        .ok_or_else(|| "main webview not found".to_string())?;
    let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed));
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

    #[cfg(target_os = "linux")]
    place_bookmark_in_fixed(&app, &label);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_set_bounds(
    app: tauri::AppHandle,
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    dpr: f64,
) -> Result<(), String> {
    let label = label_for(&id);
    #[cfg(target_os = "linux")]
    {
        let _ = (x, y, width, height); // bookmark position is derived from sidebar width on Linux
        DPR_BITS.store(dpr.to_bits(), Ordering::Relaxed);
        place_bookmark_in_fixed(&app, &label);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = dpr;
        let Some(webview) = app.get_webview(&label) else { return Ok(()) };
        webview
            .set_position(LogicalPosition::new(x, y))
            .map_err(|e: tauri::Error| e.to_string())?;
        webview
            .set_size(LogicalSize::new(width, height))
            .map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

/// Force the bookmark to navigate to a URL (typically the home URL via the
/// refresh button in the sidebar).
#[tauri::command]
#[specta::specta]
pub fn bookmark_navigate(
    app: tauri::AppHandle,
    id: String,
    url: String,
) -> Result<(), String> {
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
pub fn bookmark_reload(app: tauri::AppHandle, id: String) -> Result<(), String> {
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
pub fn bookmark_back(app: tauri::AppHandle, id: String) -> Result<(), String> {
    if let Some(webview) = app.get_webview(&label_for(&id)) {
        webview
            .eval("history.back()")
            .map_err(|e: tauri::Error| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_forward(app: tauri::AppHandle, id: String) -> Result<(), String> {
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
pub fn set_sidebar_width(app: tauri::AppHandle, width: f64) -> Result<(), String> {
    *SIDEBAR_WIDTH_CSS.lock() = Some(width);
    #[cfg(target_os = "linux")]
    if let Some(label) = ACTIVE_BOOKMARK.lock().clone() {
        place_bookmark_in_fixed(&app, &label);
    }
    #[cfg(not(target_os = "linux"))]
    let _ = app;
    Ok(())
}

/// React-side notification of the toolbar's measured height (CSS px). The
/// bookmark-toolbar webview measures itself after mount and reports here.
/// Triggers a re-layout if a bookmark is active.
#[tauri::command]
#[specta::specta]
pub fn set_toolbar_height(app: tauri::AppHandle, height: f64) -> Result<(), String> {
    *TOOLBAR_HEIGHT_CSS.lock() = Some(height);
    #[cfg(target_os = "linux")]
    if let Some(label) = ACTIVE_BOOKMARK.lock().clone() {
        place_bookmark_in_fixed(&app, &label);
    }
    #[cfg(not(target_os = "linux"))]
    let _ = app;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_hide(app: tauri::AppHandle) -> Result<(), String> {
    hide_other_bookmarks(&app, "");
    if let Some(toolbar) = app.get_webview(TOOLBAR_LABEL) {
        toolbar
            .hide()
            .map_err(|e: tauri::Error| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        *ACTIVE_BOOKMARK.lock() = None;
    }
    Ok(())
}
