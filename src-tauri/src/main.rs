// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    webview::NewWindowResponse,
    WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_updater::UpdaterExt;

const APP_URL: &str = "https://docs.tomgreen.uk";
const APP_HOST: &str = "docs.tomgreen.uk";
const OTHER_HOST: &str = "dash.tomgreen.uk";
const OTHER_SCHEME: &str = "tg-dash";

fn open_url(url: &str) {
    eprintln!("[tg-docs] open_url: {}", url);
    let _ = Command::new("/usr/bin/open").arg(url).spawn();
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // --- Build menu ---
            let hard_refresh = MenuItemBuilder::with_id("hard_refresh", "Hard Refresh")
                .accelerator("CmdOrCtrl+Shift+R")
                .build(app)?;
            let return_home = MenuItemBuilder::with_id("return_home", "Return to Docs")
                .build(app)?;
            let sign_out =
                MenuItemBuilder::with_id("sign_out", "Sign Out").build(app)?;
            let clear_data =
                MenuItemBuilder::with_id("clear_data", "Clear Local Data").build(app)?;
            let version_item = MenuItemBuilder::with_id(
                "version",
                &format!("Version {}", app.package_info().version),
            )
            .enabled(false)
            .build(app)?;

            let app_submenu = SubmenuBuilder::new(app, "App")
                .item(&hard_refresh)
                .item(&return_home)
                .separator()
                .item(&sign_out)
                .item(&clear_data)
                .separator()
                .item(&version_item)
                .quit()
                .build()?;

            let edit_submenu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            let window_submenu = SubmenuBuilder::new(app, "Window")
                .minimize()
                .close_window()
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_submenu)
                .item(&edit_submenu)
                .item(&window_submenu)
                .build()?;

            app.set_menu(menu)?;

            // --- Create window ---
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("TG Docs")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                // JS that runs on every page load (including after redirects)
                .initialization_script(r#"
                    (function() {
                        var host = window.location.hostname;
                        if (host !== 'dash.tomgreen.uk' && host !== 'docs.tomgreen.uk') return;

                        // Fix drag-to-reorder showing copy/insert instead of move in WKWebView
                        document.addEventListener('dragstart', function(e) {
                            if (e.target.closest('.tiptap') || e.target.closest('.drag-handle-icon')) {
                                e.dataTransfer.effectAllowed = 'move';
                            }
                        }, true);
                        document.addEventListener('dragover', function(e) {
                            if (e.target.closest('.tiptap')) {
                                e.dataTransfer.dropEffect = 'move';
                            }
                        }, true);
                    })();
                "#)
                // Catch target="_blank" clicks and window.open() calls
                .on_new_window(move |url, _features| {
                    let url_str = url.as_str();
                    eprintln!("[tg-docs] on_new_window: {}", url_str);

                    // Cross-app: other domain OR /tg-dash path on same domain
                    let is_other_app = url.host_str() == Some(OTHER_HOST)
                        || (url.host_str() == Some(APP_HOST) && url.path().starts_with("/tg-dash"));

                    if is_other_app {
                        let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                        let deep = format!("{}://{}{}", OTHER_SCHEME, url.path(), query);
                        open_url(&deep);
                    } else {
                        open_url(url_str);
                    }

                    NewWindowResponse::Deny
                })
                // Catch regular (non-_blank) cross-domain navigations
                .on_navigation(move |url| {
                    let url_str = url.as_str();
                    eprintln!("[tg-docs] on_navigation: {}", url_str);

                    // Cross-app: other domain OR /tg-dash path on same domain
                    let is_other_app = url.host_str() == Some(OTHER_HOST)
                        || (url.host_str() == Some(APP_HOST) && url.path().starts_with("/tg-dash"));

                    if is_other_app {
                        let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                        let deep = format!("{}://{}{}", OTHER_SCHEME, url.path(), query);
                        open_url(&deep);
                        return false;
                    }

                    // Same-domain → allow
                    if url.host_str() == Some(APP_HOST) {
                        return true;
                    }

                    // External https/http → open in browser
                    if url.scheme() == "https" || url.scheme() == "http" {
                        open_url(url_str);
                        return false;
                    }

                    true // allow tauri://, about:blank, etc.
                })
                .build()?;

            // --- Handle menu events ---
            let menu_window = window.clone();
            app.on_menu_event(move |_app_handle, event| {
                match event.id().as_ref() {
                    "clear_data" | "sign_out" => {
                        let _ = menu_window.clear_all_browsing_data();
                        let _ = menu_window.eval(&format!(
                            "window.location.replace('{}')",
                            APP_URL
                        ));
                    }
                    "hard_refresh" => {
                        let _ = menu_window.eval("window.location.reload()");
                    }
                    "return_home" => {
                        let _ = menu_window.eval(&format!(
                            "window.location.replace('{}')",
                            APP_URL
                        ));
                    }
                    _ => {}
                }
            });

            // --- Handle incoming deep links ---
            let dl_window = window.clone();
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                eprintln!("[tg-docs] deep_link: {:?}", urls);
                if let Some(url) = urls.first() {
                    let path = url.path();
                    let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                    let target = format!("{}{}{}", APP_URL, path, query);
                    let _ = dl_window.eval(&format!(
                        "window.location.replace('{}')",
                        target
                    ));
                }
            });

            // --- Check for updates in the background ---
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let updater = match app_handle.updater() {
                    Ok(u) => u,
                    Err(_) => return,
                };
                if let Ok(Some(update)) = updater.check().await {
                    let _ = update
                        .download_and_install(|_chunk, _total| {}, || {})
                        .await;
                    app_handle.restart();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
