mod automation;
mod integrations;
mod policy;
mod settings;

use integrations::steam;

use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_updater::UpdaterExt;

#[tauri::command]
fn get_snapshot(state: tauri::State<'_, automation::RuntimeState>) -> automation::RuntimeSnapshot {
    state.inner.lock().unwrap().clone()
}

#[tauri::command]
fn set_automation(
    enabled: bool,
    state: tauri::State<'_, automation::RuntimeState>,
) -> Result<automation::RuntimeSnapshot, String> {
    let _transition_guard = state.steam_transition.lock().unwrap();
    let (was_enabled, steam_path) = {
        let snapshot = state.inner.lock().unwrap();
        (snapshot.automation_enabled, snapshot.steam_path.clone())
    };
    let captured_baseline = if enabled && !was_enabled {
        steam_path
            .as_deref()
            .and_then(|path| steam::read_current_throttle(path).ok().flatten())
    } else {
        None
    };
    let mut snapshot = state.inner.lock().unwrap();
    if captured_baseline.is_some() {
        snapshot.baseline_action = captured_baseline;
    }
    snapshot.automation_enabled = enabled;
    snapshot.settings.automation_enabled = enabled;
    if !enabled && let Some(path) = steam_path.as_deref() {
        let restore = snapshot
            .baseline_action
            .clone()
            .unwrap_or(policy::BandwidthAction::Unlimited);
        steam::invoke_steam_transition(path, snapshot.applied_action.as_ref(), &restore)
            .map_err(|error| error.to_string())?;
        snapshot.applied_action = Some(restore);
    }
    settings::save(&snapshot.settings).map_err(|error| error.to_string())?;
    Ok(snapshot.clone())
}

#[tauri::command]
fn save_policy(
    app: tauri::AppHandle,
    settings: settings::AppSettings,
    state: tauri::State<'_, automation::RuntimeState>,
) -> Result<automation::RuntimeSnapshot, String> {
    if !(128_000..=125_000_000).contains(&settings.combat_limit_bytes_per_second) {
        return Err("Throttled speed limit must be between 0.128 and 125 MB/s".into());
    }
    let autostart = app.autolaunch();
    if settings.start_with_windows {
        autostart.enable().map_err(|error| error.to_string())?;
    } else if autostart.is_enabled().unwrap_or(false) {
        autostart.disable().map_err(|error| error.to_string())?;
    }
    let _transition_guard = state.steam_transition.lock().unwrap();
    settings::save(&settings).map_err(|error| error.to_string())?;
    let mut snapshot = state.inner.lock().unwrap();
    snapshot.automation_enabled = settings.automation_enabled;
    snapshot.settings = settings;
    Ok(snapshot.clone())
}

#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<String, String> {
    match app
        .updater()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())?
    {
        Some(update) => Ok(format!(
            "Steam Throttle {} is available on GitHub Releases",
            update.version
        )),
        None => Ok("Steam Throttle is up to date".into()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = automation::RuntimeState::new(settings::load());
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(app_state)
        .setup(|app| {
            automation::spawn_monitor(app.handle().clone());
            spawn_update_checks(app.handle().clone());
            let open = MenuItemBuilder::with_id("open", "Open Steam Throttle").build(app)?;
            let restore = MenuItemBuilder::with_id("restore", "Remove Steam limit").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&open, &restore, &quit])
                .build()?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("Steam Throttle")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => show_main_window(app),
                    "restore" => restore_best_effort(app),
                    "quit" => {
                        restore_best_effort(app);
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            if let Some(window) = app.get_webview_window("main") {
                let window_to_hide = window.clone();
                let app_for_close = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let close_to_tray = app_for_close
                            .state::<automation::RuntimeState>()
                            .inner
                            .lock()
                            .unwrap()
                            .settings
                            .close_to_tray;
                        if close_to_tray {
                            api.prevent_close();
                            let _ = window_to_hide.hide();
                        } else {
                            restore_best_effort(&app_for_close);
                            app_for_close.exit(0);
                        }
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            set_automation,
            save_policy,
            check_for_update
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
                restore_best_effort(app);
            }
        });
}

fn spawn_update_checks(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(8)).await;
        loop {
            if let Ok(updater) = app.updater()
                && let Ok(Some(update)) = updater.check().await
            {
                let _ = tauri::Emitter::emit(&app, "update-available", update.version);
            }
            tokio::time::sleep(std::time::Duration::from_secs(24 * 60 * 60)).await;
        }
    });
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn restore_best_effort(app: &tauri::AppHandle) {
    let state = app.state::<automation::RuntimeState>();
    let _transition_guard = state.steam_transition.lock().unwrap();
    let snapshot = state.inner.lock().unwrap();
    if snapshot.settings.restore_on_exit
        && let Some(path) = snapshot.steam_path.as_deref()
    {
        let restore = snapshot
            .baseline_action
            .as_ref()
            .unwrap_or(&policy::BandwidthAction::Unlimited);
        let _ = steam::invoke_steam_transition(path, snapshot.applied_action.as_ref(), restore);
    }
}
