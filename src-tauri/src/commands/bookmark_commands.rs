use std::sync::atomic::{AtomicU64, Ordering};

use tauri::webview::WebviewBuilder;
use tauri::{LogicalPosition, LogicalSize, Manager, Url, WebviewUrl};

const BOOKMARK_LABEL: &str = "bookmark";

/// Last-known device-pixel ratio from the main webview. Tauri's
/// `window.scale_factor()` reports integer-only on some Wayland setups
/// where the compositor does fractional scaling, so we accept the real
/// DPR from the frontend (via `window.devicePixelRatio`) and cache it.
#[cfg(target_os = "linux")]
pub(crate) static DPR_BITS: AtomicU64 = AtomicU64::new(0x3ff0_0000_0000_0000); // f64::to_bits(1.0)

#[cfg(target_os = "linux")]
pub(crate) fn current_dpr() -> f64 {
    f64::from_bits(DPR_BITS.load(Ordering::Relaxed))
}

/// Sidebar width in CSS px — must match the `w-56` class on the React
/// sidebar (14 rem × 16 px = 224 logical px). We scale up by the window's
/// device-pixel ratio when handing this to GTK's `size_allocate`, which
/// uses physical pixels.
#[cfg(target_os = "linux")]
const SIDEBAR_WIDTH_CSS: f64 = 224.0;

fn parse_url(s: &str) -> Result<Url, String> {
    let parsed: Url = Url::parse(s).map_err(|e| e.to_string())?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        other => Err(format!("unsupported URL scheme: {other}")),
    }
}

/// Linux-only: after Tauri's `add_child` has placed the bookmark webview
/// into the window's default vbox, move it into the `GtkFixed` the setup
/// step created and absolute-position it next to the main webview. Also
/// shrinks the main webview to sidebar width.
#[cfg(target_os = "linux")]
fn place_bookmark_in_fixed(app: &tauri::AppHandle) {
    let Some(main_webview) = app.get_webview("main") else {
        eprintln!("[bookmark] main webview not found");
        return;
    };
    let app_cl = app.clone();
    let _ = main_webview.with_webview(move |_pw| {
        use gtk::prelude::{Cast, ContainerExt, FixedExt, ObjectExt, WidgetExt};

        let Some(main_webview) = app_cl.get_webview("main") else { return };
        let Ok(vbox) = main_webview.window().default_vbox() else { return };

        // Find our GtkFixed (created at setup).
        let fixed: gtk::Fixed = match vbox
            .children()
            .into_iter()
            .find_map(|c| c.downcast::<gtk::Fixed>().ok())
        {
            Some(f) => f,
            None => {
                eprintln!("[bookmark] GtkFixed not found in vbox — setup may have failed");
                return;
            }
        };

        // The bookmark widget is the loose WebKitWebView that Tauri's
        // add_child just attached to the vbox (the main one is already
        // inside the fixed, not a direct child of vbox anymore).
        let bookmark_widget = vbox
            .children()
            .into_iter()
            .find(|w| w.type_().name() == "WebKitWebView");
        let Some(bookmark_widget) = bookmark_widget else {
            eprintln!("[bookmark] bookmark widget not found in vbox");
            return;
        };

        // Compute sizes — everything in PHYSICAL px (what size_allocate expects).
        let Ok(win_size) = main_webview.window().inner_size() else { return };
        let win_w = win_size.width as i32;
        let win_h = win_size.height as i32;
        // Use the DPR reported by the frontend (via `window.devicePixelRatio`)
        // rather than Tauri's scale_factor, which on some Wayland setups
        // incorrectly rounds to an integer.
        let scale = current_dpr();
        let sidebar_px = (SIDEBAR_WIDTH_CSS * scale).round() as i32;
        eprintln!("[bookmark] dpr={scale} sidebar_px={sidebar_px} win_w(physical)={win_w}");
        let bookmark_w = (win_w - sidebar_px).max(1);

        // Resize main (first child of fixed) to sidebar width. `size_allocate`
        // (instead of set_size_request) avoids setting a sticky minimum that
        // would prevent the window from shrinking below its current size.
        if let Some(main_widget) = fixed
            .children()
            .into_iter()
            .find(|w| w.type_().name() == "WebKitWebView")
        {
            main_widget.size_allocate(&gtk::Allocation::new(0, 0, sidebar_px, win_h));
        }

        // Move bookmark into fixed at (sidebar_px, 0) with size (remaining, full).
        vbox.remove(&bookmark_widget);
        fixed.put(&bookmark_widget, sidebar_px, 0);
        bookmark_widget.size_allocate(&gtk::Allocation::new(
            sidebar_px,
            0,
            bookmark_w,
            win_h,
        ));
        bookmark_widget.show();
        eprintln!(
            "[bookmark] placed in fixed at ({sidebar_px},0) size {bookmark_w}x{win_h}, main at 0,0 size {sidebar_px}x{win_h} (scale={scale})"
        );
    });
}

/// Linux-only: bookmark is being hidden. Hide the bookmark widget and
/// expand the main webview back to the full window.
#[cfg(target_os = "linux")]
fn restore_main_fullwidth(app: &tauri::AppHandle) {
    let Some(main_webview) = app.get_webview("main") else { return };
    let app_cl = app.clone();
    let _ = main_webview.with_webview(move |_pw| {
        use gtk::prelude::{Cast, ContainerExt, ObjectExt, WidgetExt};

        let Some(main_webview) = app_cl.get_webview("main") else { return };
        let Ok(vbox) = main_webview.window().default_vbox() else { return };
        let fixed: gtk::Fixed = match vbox
            .children()
            .into_iter()
            .find_map(|c| c.downcast::<gtk::Fixed>().ok())
        {
            Some(f) => f,
            None => return,
        };

        let Ok(win_size) = main_webview.window().inner_size() else { return };
        let (main_widget, bookmark_widget) = {
            let mut kids = fixed
                .children()
                .into_iter()
                .filter(|w| w.type_().name() == "WebKitWebView");
            (kids.next(), kids.next())
        };

        // Hide the bookmark widget so it doesn't keep rendering underneath /
        // alongside the main webview once we resize main.
        if let Some(bm) = bookmark_widget {
            bm.hide();
        }
        if let Some(mw) = main_widget {
            mw.size_allocate(&gtk::Allocation::new(
                0,
                0,
                win_size.width as i32,
                win_size.height as i32,
            ));
        }
    });
}

#[cfg(not(target_os = "linux"))]
fn place_bookmark_in_fixed(_app: &tauri::AppHandle) {}

#[cfg(not(target_os = "linux"))]
fn restore_main_fullwidth(_app: &tauri::AppHandle) {}

#[tauri::command]
#[specta::specta]
pub fn bookmark_open(
    app: tauri::AppHandle,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    dpr: f64,
) -> Result<(), String> {
    let parsed = parse_url(&url)?;
    #[cfg(target_os = "linux")]
    DPR_BITS.store(dpr.to_bits(), Ordering::Relaxed);
    eprintln!("[bookmark_open] url={url} rect=({x},{y}) {width}x{height} dpr={dpr}");

    if let Some(existing) = app.get_webview(BOOKMARK_LABEL) {
        existing
            .navigate(parsed)
            .map_err(|e: tauri::Error| e.to_string())?;
        existing
            .show()
            .map_err(|e: tauri::Error| e.to_string())?;
        place_bookmark_in_fixed(&app);
        return Ok(());
    }

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let builder = WebviewBuilder::new(BOOKMARK_LABEL, WebviewUrl::External(parsed));
    let bookmark = window
        .as_ref()
        .window()
        .add_child(
            builder,
            LogicalPosition::new(x, y),
            LogicalSize::new(width, height),
        )
        .map_err(|e: tauri::Error| e.to_string())?;
    bookmark
        .show()
        .map_err(|e: tauri::Error| eprintln!("[bookmark_open] show: {e}"))
        .ok();

    place_bookmark_in_fixed(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_set_bounds(
    app: tauri::AppHandle,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let Some(webview) = app.get_webview(BOOKMARK_LABEL) else {
        return Ok(());
    };
    webview
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e: tauri::Error| e.to_string())?;
    webview
        .set_size(LogicalSize::new(width, height))
        .map_err(|e: tauri::Error| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bookmark_hide(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview(BOOKMARK_LABEL) {
        webview.hide().map_err(|e: tauri::Error| e.to_string())?;
    }
    restore_main_fullwidth(&app);
    Ok(())
}
