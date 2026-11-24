#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde_json::{json, Value};
use tauri::{
    menu::{IconMenuItem, Menu, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
#[tauri::command]
async fn request(command: String, payload: Value) -> Result<Value, elarydil_kvm::Error> {
    tauri::async_runtime::spawn_blocking(move || {
        let store = elarydil_kvm::store::Store::open(None)?;
        elarydil_kvm::cli::dispatch(&store, &command, payload)
    })
    .await
    .map_err(|_| elarydil_kvm::Error::new("io"))?
}
/// Asks where to save an export with the native save panel, then writes it.
/// Returns the chosen path, or `None` when the user cancels.
#[tauri::command]
async fn save_file(
    app: tauri::AppHandle,
    name: String,
    contents: String,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = app
            .dialog()
            .file()
            .set_file_name(&name)
            .add_filter("JSON", &["json"]);
        if let Some(window) = app.get_webview_window("main") {
            dialog = dialog.set_parent(&window);
        }
        let Some(path) = dialog.blocking_save_file() else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        std::fs::write(&path, contents).map_err(|e| e.to_string())?;
        Ok(Some(path.display().to_string()))
    })
    .await
    .map_err(|e| e.to_string())?
}
/// Shows a saved file in Finder / Explorer / the file manager.
#[tauri::command]
fn reveal_file(path: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(path);
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("/usr/bin/open")
        .arg("-R")
        .arg(&path)
        .spawn();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let result = std::process::Command::new("xdg-open")
        .arg(path.parent().unwrap_or(&path))
        .spawn();
    result.map(|_| ()).map_err(|e| e.to_string())
}
/// Tells the user why the agent did not start, with a button that reveals the log.
fn report_agent_start_failure(app: &tauri::AppHandle, error: &elarydil_kvm::Error) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    let log = elarydil_kvm::cli::agent_start_log_path().display().to_string();
    let mut text = error.message.clone();
    if let Some(detail) = &error.detail {
        let detail: String = detail.chars().take(700).collect();
        text.push_str(&format!("\n\n{detail}"));
    }
    text.push_str(&format!("\n\n{}\n{log}", elarydil_kvm::t("ui.agentStartFailed.log")));
    app.dialog()
        .message(text)
        .title(elarydil_kvm::t("ui.agentStartFailed.title"))
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom(
            elarydil_kvm::t("ui.agentStartFailed.showLog"),
            elarydil_kvm::t("ui.agentStartFailed.close"),
        ))
        .show(move |show_log| {
            if show_log {
                let _ = reveal_file(log);
            }
        });
}
fn show(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
fn apply_mode(app: &tauri::AppHandle, mode: &str) {
    let tray = matches!(mode, "tray-only" | "menu-bar-only" | "both");
    if let Some(t) = app.tray_by_id("status") {
        let _ = t.set_visible(tray);
    }
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(
        if mode == "menu-bar-only" || mode == "tray-only" || mode == "headless" {
            tauri::ActivationPolicy::Accessory
        } else {
            tauri::ActivationPolicy::Regular
        },
    );
    if mode == "headless" {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.hide();
        }
    }
}
/// Menu item icons are the lucide icons the interface uses, rendered to PNG by
/// `scripts/render-menu-icons.mjs` in the menu label colour of each appearance
/// (macOS does not tint custom menu images the way it tints template images).
macro_rules! menu_png {
    ($name:literal, $dark:expr) => {
        if $dark {
            &include_bytes!(concat!("../icons/menu/", $name, "-dark.png"))[..]
        } else {
            &include_bytes!(concat!("../icons/menu/", $name, ".png"))[..]
        }
    };
}
fn menu_icon(kind: &str, dark: bool) -> tauri::image::Image<'static> {
    let png: &'static [u8] = match kind {
        "open" => menu_png!("open", dark),
        "pause" => menu_png!("pause", dark),
        "play" => menu_png!("play", dark),
        "settings" => menu_png!("settings", dark),
        "quit" => menu_png!("quit", dark),
        "checked" => menu_png!("checked", dark),
        _ => menu_png!("unchecked", dark),
    };
    tauri::image::Image::from_bytes(png).unwrap_or_else(|_| status_icon())
}
fn dark_appearance(app: &tauri::AppHandle) -> bool {
    app.get_webview_window("main")
        .and_then(|w| w.theme().ok())
        .is_some_and(|t| t == tauri::Theme::Dark)
}
fn status_icon() -> tauri::image::Image<'static> {
    let kind = "status";
    let mut data = vec![0u8; 16 * 16 * 4];
    for y in 0..16i32 {
        for x in 0..16i32 {
            let dx = x - 8;
            let dy = y - 8;
            let on = match kind {
                "status" => {
                    ((y == 3 || y == 11) && (2..=13).contains(&x))
                        || ((x == 2 || x == 13) && (3..=11).contains(&y))
                        || (x == 8 && (11..=14).contains(&y))
                        || (y == 14 && (5..=11).contains(&x))
                        || (y == 6 && (5..=10).contains(&x))
                        || (x == 10 && (5..=7).contains(&y))
                        || (y == 9 && (5..=10).contains(&x))
                        || (x == 5 && (8..=10).contains(&y))
                }
                "open" => {
                    ((y == 3 || y == 11) && (2..=13).contains(&x))
                        || ((x == 2 || x == 13) && (3..=11).contains(&y))
                        || (x == 8 && (11..=14).contains(&y))
                        || (y == 14 && (5..=11).contains(&x))
                }
                "pause" => (x == 5 || x == 6 || x == 10 || x == 11) && (3..=13).contains(&y),
                "play" => (4..=12).contains(&x) && (y - 8).abs() <= (12 - x) / 2,
                "settings" => {
                    (12..=26).contains(&(dx * dx + dy * dy))
                        || ((dx.abs() <= 1 || dy.abs() <= 1) && dx * dx + dy * dy <= 40)
                }
                _ => {
                    ((20..=38).contains(&(dx * dx + dy * dy)) && y > 5)
                        || ((x == 8 || x == 9) && (1..=8).contains(&y))
                }
            };
            if on {
                let i = ((y * 16 + x) * 4) as usize;
                data[i..i + 4].copy_from_slice(&[125, 117, 132, 255]);
            }
        }
    }
    tauri::image::Image::new_owned(data, 16, 16)
}
fn tray_action(app: &tauri::AppHandle, action: &str) {
    match action {
        "quit" => app.exit(0),
        "open" => show(app),
        "settings" => {
            show(app);
            let _ = app.emit("navigate", "settings");
        }
        "toggle" | "login" => {
            let app = app.clone();
            let action = action.to_owned();
            std::thread::spawn(move || {
                let result = (|| -> Result<(), elarydil_kvm::Error> {
                    let store = elarydil_kvm::store::Store::open(None)?;
                    if action == "toggle" {
                        if let Ok(status) = elarydil_kvm::ipc::call(&store, "status", json!({})) {
                            elarydil_kvm::ipc::call(
                                &store,
                                if status["agent"] == "paused" {
                                    "agent.resume"
                                } else {
                                    "agent.pause"
                                },
                                json!({}),
                            )?;
                        } else {
                            elarydil_kvm::cli::dispatch(&store, "agent.start", json!({}))?;
                        }
                        let _ = app.emit("agent-status-changed", ());
                    } else {
                        // Login starts the app in the menu bar; the agent
                        // runs with it. A standalone agent is a Settings option.
                        let enabled = store.load()?.app.launch_gui_at_login;
                        elarydil_kvm::cli::dispatch(
                            &store,
                            if enabled {
                                "startup.disable"
                            } else {
                                "startup.enable"
                            },
                            json!({"component":"gui"}),
                        )?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    show(&app);
                    let _ = app.emit("native-menu-error", error.to_string());
                }
            });
        }
        _ => {}
    }
}
fn main() {
    // Agents this app starts (directly or through the CLI dispatcher) inherit
    // the owner PID and stop once the app quits, unless the user enabled the
    // standalone background agent in Settings.
    std::env::set_var("ELARYDIL_KVM_OWNER_PID", std::process::id().to_string());
    let mut builder = tauri::Builder::default();
    if std::env::var("ELARYDIL_KVM_NATIVE_SMOKE").as_deref() == Ok("1") {
        builder = builder.append_invoke_initialization_script(r#";
            window.__ELARYDIL_SMOKE_ERRORS__ = [];
            window.addEventListener('error', e => window.__ELARYDIL_SMOKE_ERRORS__.push(e.message || ('Resource: ' + e.target?.src)), true);
            window.addEventListener('unhandledrejection', e => window.__ELARYDIL_SMOKE_ERRORS__.push(String(e.reason)));
            window.addEventListener('securitypolicyviolation', e => window.__ELARYDIL_SMOKE_ERRORS__.push(e.violatedDirective + ': ' + e.blockedURI));
        "#);
    }
    builder
 .plugin(tauri_plugin_single_instance::init(|app,_,_|show(app)))
 .plugin(tauri_plugin_dialog::init())
 .invoke_handler(tauri::generate_handler![request, save_file, reveal_file])
 .on_page_load(|webview,payload|{
    if std::env::var("ELARYDIL_KVM_NATIVE_SMOKE").as_deref()==Ok("1")&&matches!(payload.event(),tauri::webview::PageLoadEvent::Finished)&&webview.label()=="main" {
        if let Ok(store)=elarydil_kvm::store::Store::open(None){let report=json!({"pid":std::process::id(),"pageFinished":true,"windowVisible":webview.window().is_visible().unwrap_or(false),"url":payload.url().to_string()});let _=store.atomic("native-gui-ready.json",&serde_json::to_vec(&report).unwrap());}
    }
 })
 .setup(|app|{
    let store=elarydil_kvm::store::Store::open(None)?;
    // The first run of a freshly installed agent can wait on macOS's check of
    // the new executable, so a slow start is retried in the background. Any
    // other failure, or a start that never succeeds, is shown with its reason;
    // agent.start also records it in agent_start_log_path().
    let agent_store=store.clone();
    let agent_app=app.handle().clone();
    std::thread::spawn(move||{
        let mut attempt=0;
        loop {
            attempt+=1;
            match elarydil_kvm::cli::dispatch(&agent_store,"agent.start",json!({})) {
                Ok(_)=>{let _=agent_app.emit("agent-status-changed",());return;}
                Err(e) if attempt<4&&matches!(e.code.as_str(),"agent_not_running"|"timeout")=>std::thread::sleep(std::time::Duration::from_secs(2)),
                Err(e)=>{report_agent_start_failure(&agent_app,&e);return;}
            }
        }
    });
    let dark=dark_appearance(app.handle());
    // An unreadable configuration must not stop the window from opening;
    // the failed agent start above explains it.
    let login_on=store.load().map(|c|c.app.launch_gui_at_login).unwrap_or(false);
    let open=IconMenuItem::with_id(app,"open",elarydil_kvm::t("tray.open"),true,Some(menu_icon("open",dark)),None::<&str>)?;
    let toggle=IconMenuItem::with_id(app,"toggle",elarydil_kvm::t("tray.pause"),true,Some(menu_icon("pause",dark)),None::<&str>)?;
    // A checkbox-style icon keeps "Open at Login" aligned with the other items
    // and visibly toggleable whether it is on or off.
    let login=IconMenuItem::with_id(app,"login",elarydil_kvm::t("tray.login"),true,Some(menu_icon(if login_on{"checked"}else{"unchecked"},dark)),None::<&str>)?;
    let settings=IconMenuItem::with_id(app,"settings",elarydil_kvm::t("tray.settings"),true,Some(menu_icon("settings",dark)),None::<&str>)?;
    let quit=IconMenuItem::with_id(app,"quit",elarydil_kvm::t("tray.quit"),true,Some(menu_icon("quit",dark)),None::<&str>)?;
    let separator=PredefinedMenuItem::separator(app)?;let bottom=PredefinedMenuItem::separator(app)?;
    let menu=Menu::with_items(app,&[&open,&toggle,&login,&separator,&settings,&bottom,&quit])?;
    #[cfg(target_os="macos")] let tray_image=status_icon();
    #[cfg(not(target_os="macos"))] let tray_image=tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
    let builder=TrayIconBuilder::with_id("status").tooltip(elarydil_kvm::t("ui.brand")).icon(tray_image);
    #[cfg(target_os="macos")] let builder=builder.icon_as_template(true);
    let tray=builder.menu(&menu).on_menu_event(|app,event|tray_action(app,event.id.as_ref())).build(app);
    let mode=store.load().map(|c|c.app.ui_mode).unwrap_or_else(|_|elarydil_kvm::model::Config::default().app.ui_mode);
    if tray.is_ok(){apply_mode(app.handle(),&mode);}
    let menu_store=store.clone();
    let menu_handle=app.handle().clone();
    std::thread::spawn(move || {
        let mut previous=String::new();
        let mut shown_dark=dark;
        let mut shown_login=login_on;
        loop {
            let dark=dark_appearance(&menu_handle);
            let restyle=dark!=shown_dark;
            if restyle {
                shown_dark=dark;
                let _=open.set_icon(Some(menu_icon("open",dark)));
                let _=settings.set_icon(Some(menu_icon("settings",dark)));
                let _=quit.set_icon(Some(menu_icon("quit",dark)));
            }
            let state=elarydil_kvm::ipc::call(&menu_store,"status",json!({})).map(|s|if s["agent"]=="paused"{"paused"}else{"running"}).unwrap_or("stopped");
            let key=match state{"paused"=>"tray.resume","stopped"=>"tray.start",_=>"tray.pause"};
            if previous!=state || restyle {previous=state.into();let _=toggle.set_text(elarydil_kvm::t(key));let _=toggle.set_icon(Some(menu_icon(if state=="running"{"pause"}else{"play"},dark)));}
            if let Ok(c)=menu_store.load(){
                if c.app.launch_gui_at_login!=shown_login || restyle {
                    shown_login=c.app.launch_gui_at_login;
                    let _=login.set_icon(Some(menu_icon(if shown_login{"checked"}else{"unchecked"},dark)));
                }
            }
            if std::env::var("ELARYDIL_KVM_NATIVE_SMOKE").as_deref()==Ok("1") {
                let ids=menu.items().unwrap_or_default().iter().map(|i|i.id().as_ref().to_string()).filter(|id|["open","toggle","login","settings","quit"].contains(&id.as_str())).collect::<Vec<_>>();
                let _=menu_store.atomic("native-menu-state.json",&serde_json::to_vec(&json!({"toggleLabel":toggle.text().unwrap_or_default(),"order":ids})).unwrap());
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    let handle=app.handle().clone();
    std::thread::spawn(move || {
        let mut previous=mode;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let path=store.root.join("gui-close-request.json");
            if let Some(request)=std::fs::read(&path).ok().and_then(|b|serde_json::from_slice::<Value>(&b).ok()) {
                let age=chrono_millis()-request["createdAt"].as_i64().unwrap_or(0);let _=std::fs::remove_file(&path);
                if (0..=3000).contains(&age)&&request["id"].is_string(){let _=store.atomic("gui-close-ack.json",&serde_json::to_vec(&request).unwrap());handle.exit(0);break;}
            }
            if std::env::var("ELARYDIL_KVM_NATIVE_SMOKE").as_deref()==Ok("1") {
                if let Some(window) = handle.get_webview_window("main") {
                    let report_store = store.clone();
                    let _ = window.eval_with_callback(r#"({page:document.querySelector('.content')?.dataset.page??null,text:document.body?.innerText.slice(0,5000)??'',url:location.href,readyState:document.readyState,errors:window.__ELARYDIL_SMOKE_ERRORS__??[],ipc:!!window.__TAURI_INTERNALS__?.invoke})"#, move |report| {
                        let _ = report_store.atomic("native-frontend.json", report.as_bytes());
                    });
                }
                let path=store.root.join("native-menu-request.json");
                if let Some(request)=std::fs::read(&path).ok().and_then(|b|serde_json::from_slice::<Value>(&b).ok()){let _=std::fs::remove_file(path);if let Some(action)=request["action"].as_str(){tray_action(&handle,action);}}
            }
            if let Ok(c)=store.load(){if previous!=c.app.ui_mode {previous=c.app.ui_mode;let h=handle.clone();let mode=previous.clone();let _=handle.run_on_main_thread(move||apply_mode(&h,&mode));}}
        }
    });
    Ok(())
 })
 .on_window_event(|window,event|{
    if let tauri::WindowEvent::CloseRequested{api,..}=event {if let Ok(s)=elarydil_kvm::store::Store::open(None){if let Ok(c)=s.load(){if c.app.close_window_action=="hide"&&matches!(c.app.ui_mode.as_str(),"tray-only"|"menu-bar-only"|"both"){api.prevent_close();let _=window.hide();}}}}
 })
 .run(tauri::generate_context!()).expect("desktop runtime");
}
fn chrono_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
