// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

// 运行身份：普通桌面用户 (Wayland / Hyprland 原生渲染)

use crate::battery::BatteryInfo;
use crate::ipc::{IpcCommand, IpcEvent, IPC_SOCKET_PATH};
use crate::sensors::HardwareSensors;
use crate::tuning::CustomTuningConfig;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, State};

const RGB_PRESETS: &[(u8, u8, u8)] = &[
    (255, 255, 255),
    (255, 0, 0),
    (0, 255, 0),
    (0, 0, 255),
    (0, 255, 255),
    (180, 0, 255),
    (255, 204, 0),
    (255, 102, 0),
];

#[derive(Clone, Serialize, Deserialize)]
pub struct TelemetryPayload {
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub fan_cpu_rpm: Option<u32>,
    pub fan_gpu_rpm: Option<u32>,
    pub battery: Option<BatteryInfo>,
}

struct AppIpcState {
    cmd_sender: Sender<IpcCommand>,
    is_connected: Arc<AtomicBool>,
}

#[tauri::command]
fn send_command(
    action: String,
    payload: serde_json::Value,
    state: State<'_, AppIpcState>,
) -> Result<(), String> {
    let cmd = match action.as_str() {
        "SetMode" => {
            let mode = payload.get("mode").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            IpcCommand::SetMode { mode }
        }
        "SetFanBoost" => {
            let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            IpcCommand::SetFanBoost { enabled }
        }
        "SetFnLock" => {
            let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            IpcCommand::SetFnLock { enabled }
        }
        "SetWinLock" => {
            let locked = payload.get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
            IpcCommand::SetWinLock { locked }
        }
        "SetTouchpadDisable" => {
            let disabled = payload.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
            IpcCommand::SetTouchpadDisable { disabled }
        }
        "SetRgbPower" => {
            let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            IpcCommand::SetRgbPower { enabled }
        }
        "SetRgbBrightness" => {
            let percent = payload.get("percent").and_then(|v| v.as_u64()).unwrap_or(100) as u8;
            IpcCommand::SetRgbBrightness { percent }
        }
        "SetRgbColor" => {
            let (r, g, b) = if let (Some(r), Some(g), Some(b)) = (
                payload.get("r").and_then(|v| v.as_u64()),
                payload.get("g").and_then(|v| v.as_u64()),
                payload.get("b").and_then(|v| v.as_u64()),
            ) {
                (r as u8, g as u8, b as u8)
            } else {
                let idx = payload.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                RGB_PRESETS.get(idx).copied().unwrap_or((255, 255, 255))
            };
            IpcCommand::SetRgbColor { r, g, b }
        }
        "SetRgbEffect" => {
            let effect_mode = payload.get("effect_mode").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            IpcCommand::SetRgbEffect { effect_mode }
        }
        "SetBatteryLimit" => {
            let mode = payload.get("mode").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            IpcCommand::SetBatteryLimit { mode }
        }
        "SetCustomTuning" => {
            if let Some(config_val) = payload.get("config") {
                if let Ok(config) = serde_json::from_value::<CustomTuningConfig>(config_val.clone()) {
                    IpcCommand::SetCustomTuning { config }
                } else {
                    return Err(format!("解析调优配置失败: {:?}", config_val));
                }
            } else {
                return Err("缺少调优配置参数".into());
            }
        }
        "SetCustomMode" => {
            let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            IpcCommand::SetCustomMode { enabled }
        }
        "SwitchGpuMode" => {
            let to_direct = payload.get("to_direct").and_then(|v| v.as_bool()).unwrap_or(true);
            IpcCommand::SwitchGpuMode { to_direct }
        }
        "SetSilentTurbo" => {
            let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            IpcCommand::SetSilentTurbo { enabled }
        }
        "SetTurboSubMode" => {
            let sub_mode = payload.get("sub_mode").and_then(|v| v.as_i64()).map(|v| v as i32).unwrap_or(1);
            IpcCommand::SetTurboSubMode { sub_mode }
        }
        "SetDgpuRuntimePower" => {
            let power_on = payload.get("power_on").and_then(|v| v.as_bool()).unwrap_or(true);
            IpcCommand::SetDgpuRuntimePower { power_on }
        }
        "InstallDrivers" => IpcCommand::InstallDrivers,
        "InstallAcpiPatches" => IpcCommand::InstallAcpiPatches,
        "RollbackAcpiPatches" => IpcCommand::RollbackAcpiPatches,
        "InstallHibernateFix" => IpcCommand::InstallHibernateFix,
        "RollbackHibernateFix" => IpcCommand::RollbackHibernateFix,
        "ExitAndRestoreHardware" => IpcCommand::ExitAndRestoreHardware,
        "GetStatus" => IpcCommand::GetStatus,
        "WakeupGui" => IpcCommand::WakeupGui,
        other => return Err(format!("未知操作指令: {}", other)),
    };

    state.cmd_sender.send(cmd).map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn minimize_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn toggle_maximize(app_handle: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let is_max = window.is_maximized().map_err(|e| e.to_string())?;
        if is_max {
            window.unmaximize().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.maximize().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn quit_app(state: State<'_, AppIpcState>) -> Result<(), String> {
    crate::log_debug!("[Tauri 客户端] 收到退出请求，通知守护服务脱机恢复出厂自主模式...");
    let _ = state.cmd_sender.send(IpcCommand::ExitAndRestoreHardware);
    std::thread::sleep(Duration::from_millis(200));
    std::process::exit(0);
}

#[tauri::command]
fn get_initial_status() -> crate::config::AppConfig {
    crate::config::AppConfig::load()
}

#[tauri::command]
fn get_system_info() -> crate::sensors::SystemDriverStatus {
    HardwareSensors::get_system_status()
}

#[tauri::command]
fn get_hibernate_status() -> crate::hibernate::HibernateStatus {
    crate::hibernate::HibernateManager::check_status()
}

#[tauri::command]
fn get_acpi_patch_status() -> crate::acpi_patches::AcpiPatchStatus {
    crate::acpi_patches::AcpiPatchManager::check_status()
}

#[tauri::command]
fn check_ipc_connected(state: State<'_, AppIpcState>) -> bool {
    state.is_connected.load(Ordering::SeqCst)
}

fn read_basic_battery_sysfs() -> (u8, bool, f32, i32, f32) {
    let bat_base = std::path::Path::new("/sys/class/power_supply/BAT0");
    let ac_base = std::path::Path::new("/sys/class/power_supply/AC0");

    let mut capacity: u8 = 100;
    let mut voltage_uv: u64 = 16900000;
    let mut current_ua: i64 = 0;

    if bat_base.exists() {
        if let Ok(s) = std::fs::read_to_string(bat_base.join("capacity")) {
            capacity = s.trim().parse().unwrap_or(100);
        }
        if let Ok(s) = std::fs::read_to_string(bat_base.join("voltage_now")) {
            voltage_uv = s.trim().parse().unwrap_or(16900000);
        }
        if let Ok(s) = std::fs::read_to_string(bat_base.join("current_now")) {
            current_ua = s.trim().parse().unwrap_or(0);
        }
    }

    let is_ac_online = if ac_base.exists() {
        std::fs::read_to_string(ac_base.join("online"))
            .map(|s| s.trim() == "1")
            .unwrap_or(true)
    } else {
        true
    };

    let voltage = (voltage_uv as f32) / 1_000_000.0;
    let current_ma = (current_ua / 1000) as i32;
    let power_w = voltage * (current_ma.abs() as f32 / 1000.0);

    (capacity, is_ac_online, voltage, current_ma, power_w)
}

pub fn run_gui_app() -> Result<(), Box<dyn std::error::Error>> {
    // 强制开启 Linux / WebKitGTK 硬件加速 (GPU 渲染)，彻底杜绝 CPU 软件光栅化掉帧
    unsafe {
        if std::env::var("WEBKIT_FORCE_COMPOSITING_MODE").is_err() {
            std::env::set_var("WEBKIT_FORCE_COMPOSITING_MODE", "1");
        }
        if std::env::var("WEBKIT_HARDWARE_ACCELERATION_POLICY").is_err() {
            std::env::set_var("WEBKIT_HARDWARE_ACCELERATION_POLICY", "always");
        }
        if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "0");
        }
    }

    crate::logger::init_log_level();
    crate::log_info!("[GUI] 控制中心启动");

    let (cmd_tx, cmd_rx): (Sender<IpcCommand>, Receiver<IpcCommand>) = channel();
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let is_connected = Arc::new(AtomicBool::new(false));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppIpcState {
            cmd_sender: cmd_tx.clone(),
            is_connected: Arc::clone(&is_connected),
        })
        .invoke_handler(tauri::generate_handler![
            send_command,
            hide_window,
            minimize_window,
            toggle_maximize,
            quit_app,
            get_initial_status,
            get_system_info,
            get_hibernate_status,
            get_acpi_patch_status,
            check_ipc_connected
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let screen_size = monitor.size();
                    let scale_factor = monitor.scale_factor();
                    let logical_width = (screen_size.width as f64 / scale_factor) as u32;
                    let logical_height = (screen_size.height as f64 / scale_factor) as u32;

                    let target_w = (logical_width as f64 * 0.62).round() as u32;
                    let target_h = (logical_height as f64 * 0.72).round() as u32;

                    let w = target_w.clamp(1160, 1360);
                    let h = target_h.clamp(740, 860);

                    let _ = window.set_size(tauri::LogicalSize::new(w, h));
                    let _ = window.center();
                }
            }

            let show_i = MenuItem::with_id(app, "show", "💻 显示控制中心 (主界面)", true, None::<&str>)?;
            let rage_i = MenuItem::with_id(app, "rage", "🔥 狂暴模式", true, None::<&str>)?;
            let bal_i = MenuItem::with_id(app, "bal", "⚖️ 均衡模式", true, None::<&str>)?;
            let off_i = MenuItem::with_id(app, "off", "🌱 办公模式", true, None::<&str>)?;
            let fan_i = MenuItem::with_id(app, "fan", "🌪️ 暴风强冷 (开关)", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "✕ 退出控制中心 (脱机恢复硬件独立模式)", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &rage_i, &bal_i, &off_i, &fan_i, &quit_i])?;

            let tx_tray = cmd_tx.clone();
            let tx_tray_click = cmd_tx.clone();
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .ok()
                .or_else(|| app.default_window_icon().cloned());
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("RMechrevo")
                .show_menu_on_left_click(false);
            if let Some(icon) = tray_icon {
                tray_builder = tray_builder.icon(icon);
            }
            let _tray = tray_builder
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = tx_tray.send(IpcCommand::GetStatus);
                            }
                        }
                        "rage" => {
                            let _ = tx_tray.send(IpcCommand::SetMode { mode: 0 });
                        }
                        "bal" => {
                            let _ = tx_tray.send(IpcCommand::SetMode { mode: 1 });
                        }
                        "off" => {
                            let _ = tx_tray.send(IpcCommand::SetMode { mode: 2 });
                        }
                        "fan" => {
                            let _ = tx_tray.send(IpcCommand::SetFanBoost { enabled: true });
                        }
                        "quit" => {
                            crate::log_debug!("[托盘] 用户点击完全退出，通知守护进程脱机还原...");
                            let _ = tx_tray.send(IpcCommand::ExitAndRestoreHardware);
                            std::thread::sleep(Duration::from_millis(200));
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = tx_tray_click.send(IpcCommand::GetStatus);
                        }
                    }
                })
                .build(app)?;

            let cmd_rx_clone = Arc::clone(&cmd_rx);
            let cmd_tx_for_reconnect = cmd_tx.clone();
            let app_handle_for_ipc = app_handle.clone();
            let is_connected_for_thread = Arc::clone(&is_connected);

            thread::spawn(move || {
                loop {
                    match UnixStream::connect(IPC_SOCKET_PATH) {
                        Ok(stream) => {
                            crate::log_debug!("[Tauri IPC] 成功建立与硬件守护服务的长连接: {}", IPC_SOCKET_PATH);
                            is_connected_for_thread.store(true, Ordering::SeqCst);
                            let _ = cmd_tx_for_reconnect.send(IpcCommand::GetStatus);
                            let _ = app_handle_for_ipc.emit("connection-status", serde_json::json!({
                                "connected": true,
                                "message": "硬件守护直通服务已连接"
                            }));

                            let stream_write = match stream.try_clone() {
                                Ok(s) => s,
                                Err(_) => {
                                    thread::sleep(Duration::from_secs(1));
                                    continue;
                                }
                            };

                            let conn_alive = Arc::new(AtomicBool::new(true));
                            let conn_alive_write = Arc::clone(&conn_alive);
                            let rx_clone = Arc::clone(&cmd_rx_clone);
                            let mut write_stream = stream_write;

                            let _write_handle = thread::spawn(move || {
                                while conn_alive_write.load(Ordering::SeqCst) {
                                    let cmd_opt = {
                                        let rx = rx_clone.lock().unwrap();
                                        rx.recv_timeout(Duration::from_millis(100)).ok()
                                    };
                                    if let Some(cmd) = cmd_opt {
                                        if let Ok(json) = serde_json::to_string(&cmd) {
                                            if write_stream.write_all(format!("{}\n", json).as_bytes()).is_err() {
                                                break;
                                            }
                                            let _ = write_stream.flush();
                                        }
                                    }
                                }
                            });

                            let reader = BufReader::new(stream);
                            let app_h = app_handle_for_ipc.clone();
                            for line in reader.lines() {
                                match line {
                                    Ok(text) => {
                                        if let Ok(event) = serde_json::from_str::<IpcEvent>(&text) {
                                            if let IpcEvent::WakeupRequested = event {
                                                if let Some(window) = app_h.get_webview_window("main") {
                                                    let _ = window.show();
                                                    let _ = window.set_focus();
                                                }
                                            }
                                            let _ = app_h.emit("ipc-event", event);
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }

                            conn_alive.store(false, Ordering::SeqCst);
                            is_connected_for_thread.store(false, Ordering::SeqCst);
                            crate::log_debug!("[Tauri IPC] 守护连接中断，准备进入自动重连循环...");
                            let _ = app_handle_for_ipc.emit("connection-status", serde_json::json!({
                                "connected": false,
                                "message": "硬件守护服务连接中断 (正在自动重连...)"
                            }));
                        }
                        Err(_) => {
                            is_connected_for_thread.store(false, Ordering::SeqCst);
                            let _ = app_handle_for_ipc.emit("connection-status", serde_json::json!({
                                "connected": false,
                                "message": "硬件守护连接中..."
                            }));
                        }
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            });

            let app_handle_for_sensors = app_handle.clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let readings = HardwareSensors::read_all();
                    let (capacity, is_ac_online, voltage, current_ma, power_w) = read_basic_battery_sysfs();
                    let bat_info = BatteryInfo {
                        capacity,
                        status: if is_ac_online { "电源已接通".to_string() } else { "放电中".to_string() },
                        is_ac_online,
                        voltage,
                        current_ma,
                        power_w,
                        health_percent: 100,
                        cycle_count: 0,
                        charge_now_mah: 0,
                        charge_full_mah: 0,
                        charge_design_mah: 0,
                        limit_mode: -1,
                        limit_mode_reg: 0,
                        limit_up: 0,
                        limit_down: 0,
                        hardware_status_msg: String::new(),
                    };
                    let payload = TelemetryPayload {
                        cpu_temp: readings.cpu_temp,
                        gpu_temp: readings.gpu_temp,
                        fan_cpu_rpm: readings.fan_cpu_rpm,
                        fan_gpu_rpm: readings.fan_gpu_rpm,
                        battery: Some(bat_info),
                    };
                    let _ = app_handle_for_sensors.emit("telemetry", payload);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}
