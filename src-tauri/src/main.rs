// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    webview::{PageLoadEvent, PageLoadPayload},
    Manager, Webview,
};
use tauri_plugin_deep_link::DeepLinkExt;

const APP_URL: &str = "https://docs.tomgreen.uk";

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // --- Build menu ---
            let hard_refresh = MenuItemBuilder::with_id("hard_refresh", "Hard Refresh")
                .accelerator("CmdOrCtrl+Shift+R")
                .build(app)?;
            let sign_out =
                MenuItemBuilder::with_id("sign_out", "Sign Out").build(app)?;
            let clear_data =
                MenuItemBuilder::with_id("clear_data", "Clear Local Data").build(app)?;

            let app_submenu = SubmenuBuilder::new(app, "App")
                .item(&hard_refresh)
                .separator()
                .item(&sign_out)
                .item(&clear_data)
                .separator()
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

            let return_home = MenuItemBuilder::with_id("return_home", "Return to Docs")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&app_submenu)
                .item(&edit_submenu)
                .item(&window_submenu)
                .item(&return_home)
                .build()?;

            app.set_menu(menu)?;

            // --- Handle menu events ---
            let window = app.get_webview_window("main").unwrap();
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
                if let Some(url) = event.urls().first() {
                    let path = url.path();
                    let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                    let target = format!("{}{}{}", APP_URL, path, query);
                    let _ = dl_window.eval(&format!(
                        "window.location.replace('{}')",
                        target
                    ));
                }
            });

            Ok(())
        })
        .on_page_load(|webview: &Webview, payload: &PageLoadPayload<'_>| {
            if payload.event() == PageLoadEvent::Finished {
                // Inject link rewriting JS
                // Note: window.__TAURI__.shell.open() is NOT available on remote pages
                // because the npm plugin JS isn't loaded. Use the raw IPC invoke instead.
                let _ = webview.eval(
                    r#"
                    (function() {
                        if (window.__tg_link_rewriter_installed) return;
                        window.__tg_link_rewriter_installed = true;

                        const SCHEME_MAP = {
                            'docs.tomgreen.uk': 'tg-docs',
                            'dash.tomgreen.uk': 'tg-dash'
                        };
                        const currentHost = window.location.hostname;

                        function shellOpen(url) {
                            return window.__TAURI__.core.invoke('plugin:shell|open', {
                                path: url,
                                with: ''
                            });
                        }

                        // Override window.open — WKWebView swallows these silently
                        window.open = function(url, target, features) {
                            if (!url) return null;
                            try {
                                const parsed = new URL(url, window.location.origin);
                                const scheme = SCHEME_MAP[parsed.hostname];
                                if (scheme && parsed.hostname !== currentHost) {
                                    shellOpen(
                                        scheme + '://' + parsed.pathname + parsed.search + parsed.hash
                                    );
                                } else {
                                    shellOpen(parsed.href);
                                }
                            } catch(_) {}
                            return null;
                        };

                        // Intercept <a> clicks
                        document.addEventListener('click', function(e) {
                            const link = e.target.closest('a');
                            if (!link) return;
                            try {
                                const url = new URL(link.href);
                                const scheme = SCHEME_MAP[url.hostname];
                                if (scheme && url.hostname !== currentHost) {
                                    e.preventDefault();
                                    shellOpen(
                                        scheme + '://' + url.pathname + url.search + url.hash
                                    );
                                } else if (link.target === '_blank') {
                                    e.preventDefault();
                                    shellOpen(link.href);
                                }
                            } catch(_) {}
                        }, true);
                    })();
                    "#,
                );
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
