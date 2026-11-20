// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use crate::acpi_patches::AcpiPatchManager;
use crate::battery::{BatteryLimitMode, BatteryManager};
use crate::drivers::{DriverManager, DriverStatus};
use crate::gpu_mode::{GpuModeManager, GpuOutputMode};
use crate::hibernate::HibernateManager;
use crate::hotkeys::{HotkeyListener, HotkeyStatus};
use crate::ipc::{IpcCommand, IpcEvent, IPC_SOCKET_PATH};
use crate::profile::{PerformanceMode, ProfileManager};
use crate::rgb::{RgbEffectMode, RgbKeyboard};
use crate::switches::LaptopSwitches;
use crate::tuning::{CustomTuningConfig, TuningManager};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

static ACTIVE_CONFIG: OnceLock<Arc<Mutex<crate::config::AppConfig>>> = OnceLock::new();

pub fn flush_pending_config() {
    if let Some(cfg_arc) = ACTIVE_CONFIG.get() {
        if let Ok(cfg) = cfg_arc.lock() {
            let _ = cfg.save_if_dirty();
        }
    }
}

pub fn run_daemon_server() -> Result<(), Box<dyn std::error::Error>> {
    crate::logger::init_log_level();
    crate::log_info!("[Daemon] 启动硬件守护服务");

    let saved_cfg = crate::config::AppConfig::load();

    let profile_mgr = Arc::new(ProfileManager::new());
    let tuning_mgr = Arc::new(TuningManager::new());
    let rgb_mgr = Arc::new(RgbKeyboard::new());

    let current_mode = Arc::new(Mutex::new(PerformanceMode::from_i32(saved_cfg.performance_mode)));
    let fan_boost = Arc::new(Mutex::new(false));
    let turbo_sub_mode = Arc::new(Mutex::new(saved_cfg.turbo_sub_mode));
    let custom_mode_enabled = Arc::new(Mutex::new(saved_cfg.performance_mode == PerformanceMode::Custom.to_i32()));
    let rgb_power = Arc::new(Mutex::new(saved_cfg.rgb_power));
    let rgb_color = Arc::new(Mutex::new((saved_cfg.rgb_r, saved_cfg.rgb_g, saved_cfg.rgb_b)));
    let rgb_brightness = Arc::new(Mutex::new(saved_cfg.rgb_brightness));
    let rgb_effect_mode = Arc::new(Mutex::new(saved_cfg.rgb_effect_mode));
    let battery_mode = Arc::new(Mutex::new(BatteryLimitMode::from_i32(saved_cfg.battery_limit_mode)));
    let custom_tuning = Arc::new(Mutex::new(saved_cfg.custom_tuning.clone()));
    let driver_status = Arc::new(Mutex::new(DriverManager::check_status()));
    let app_config_storage = Arc::new(Mutex::new(saved_cfg.clone()));
    let _ = ACTIVE_CONFIG.set(Arc::clone(&app_config_storage));
    let clients: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));

    if Path::new(IPC_SOCKET_PATH).exists() {
        let _ = fs::remove_file(IPC_SOCKET_PATH);
    }

    let listener = UnixListener::bind(IPC_SOCKET_PATH)?;
    fs::set_permissions(IPC_SOCKET_PATH, fs::Permissions::from_mode(0o666))?;
    crate::log_info!("[Daemon] IPC 就绪: {}", IPC_SOCKET_PATH);

    let cfg_for_timer = Arc::clone(&app_config_storage);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            if let Ok(cfg) = cfg_for_timer.lock() {
                let _ = cfg.save_if_dirty();
            }
        }
    });

    {
        let ds = driver_status.lock().unwrap();
        crate::log_debug!(
            "[驱动检测] 就绪={}, acpi_call={}, uniwill={}, 按键={}",
            ds.ready, ds.acpi_call_loaded, ds.uniwill_loaded, ds.hotkey_ready
        );
        if !ds.ready {
            crate::log_info!("[驱动检测] {}", ds.message);
        }
    }

    // 控制台初始化握手：通知 EC 软件在线 (0x0741 写入 0x01)
    {
        use crate::ec::{EcInterface, registers};
        let ec = EcInterface::new();
        let _ = ec.write(registers::ADDR_AP_OEM, registers::FLAG_AP_OEM_ONLINE);
        let cur_66 = ec.read(registers::ADDR_SUPPORT_BYTE2).unwrap_or(0);
        let _ = ec.write(registers::ADDR_SUPPORT_BYTE2, cur_66 | registers::FLAG_RGB_SUPPORT);
    }

    rgb_mgr.restore_state(
        saved_cfg.rgb_power,
        saved_cfg.rgb_r,
        saved_cfg.rgb_g,
        saved_cfg.rgb_b,
        saved_cfg.rgb_brightness,
        saved_cfg.rgb_effect_mode,
    );
    let init_mode = PerformanceMode::from_i32(saved_cfg.performance_mode);
    if init_mode == PerformanceMode::Custom {
        let _ = tuning_mgr.set_silent_turbo(false);
        let _ = tuning_mgr.apply_tuning(&saved_cfg.custom_tuning);
        let _ = profile_mgr.apply_mode(PerformanceMode::Custom);
    } else {
        let _ = profile_mgr.apply_mode(init_mode);
        let _ = tuning_mgr.set_silent_turbo(init_mode == PerformanceMode::Rage && saved_cfg.turbo_sub_mode == 0);
    }
    let _ = LaptopSwitches::set_fn_lock(saved_cfg.fn_lock);
    let _ = LaptopSwitches::set_win_lock(saved_cfg.win_lock);
    let _ = LaptopSwitches::set_touchpad_disabled(saved_cfg.touchpad_disabled);
    let _ = BatteryManager::set_limit(BatteryLimitMode::from_i32(saved_cfg.battery_limit_mode));

    let clients_for_hotkey = Arc::clone(&clients);
    let mode_for_hotkey = Arc::clone(&current_mode);
    let fan_for_hotkey = Arc::clone(&fan_boost);
    let custom_for_hotkey = Arc::clone(&custom_mode_enabled);
    let cfg_for_hotkey = Arc::clone(&app_config_storage);
    let profile_mgr_hotkey = Arc::clone(&profile_mgr);
    let tuning_mgr_hotkey = Arc::clone(&tuning_mgr);

    let (_hotkey_listener, hotkey_status) = HotkeyListener::start(move || {
        let (was_custom, next_mode) = {
            let mut m = mode_for_hotkey.lock().unwrap();
            let prev_custom = *m == PerformanceMode::Custom;
            let cur_hw = profile_mgr_hotkey.get_current_mode();
            let next = cur_hw.next();
            *m = next;
            (prev_custom, next)
        };

        if was_custom {
            *custom_for_hotkey.lock().unwrap() = false;
            let _ = tuning_mgr_hotkey.enable_custom_mode(false);
        }

        let was_boost = {
            let mut fb = fan_for_hotkey.lock().unwrap();
            let prev = *fb;
            *fb = false;
            prev
        };

        {
            let mut cfg = cfg_for_hotkey.lock().unwrap();
            cfg.performance_mode = next_mode.to_i32();
            cfg.fan_boost = false;
            cfg.mark_dirty();
        }

        crate::log_debug!("[物理按键] 切换模式 -> {}", next_mode.name());
        let result_msg = match profile_mgr_hotkey.apply_mode(next_mode) {
            Ok(msg) => msg,
            Err(e) => e,
        };

        broadcast_event(
            &clients_for_hotkey,
            &IpcEvent::ModeChanged {
                mode: next_mode.to_i32(),
                mode_name: next_mode.name().to_string(),
                source: "机身物理按键".to_string(),
                result: result_msg,
            },
        );
        if was_boost {
            broadcast_event(
                &clients_for_hotkey,
                &IpcEvent::FanBoostChanged {
                    enabled: false,
                    message: "切换性能模式，暴风强冷已退出".into(),
                },
            );
        }
    });

    match &hotkey_status {
        HotkeyStatus::Connected(p) => crate::log_debug!("[物理按键] 已成功挂钩物理输入设备: {:?}", p),
        HotkeyStatus::PermissionDenied(p) => crate::log_error!("[物理按键] 权限不足: {:?}", p),
        HotkeyStatus::DeviceNotFound => crate::log_error!("[物理按键] 未找到 Uniwill WMI hotkeys 设备"),
    }

    crate::log_debug!("[硬件开关] uniwill sysfs 可用性: {}", LaptopSwitches::is_available());
    crate::log_debug!("[显卡模式] 当前系统输出模式: {}", GpuModeManager::detect_current_mode().name());
    crate::log_debug!("[硬件补丁] ACPI 睡眠修复状态: {}", AcpiPatchManager::check_status().message);
    crate::log_debug!("[守护进程] 等待桌面 GUI 控制台连接...\n");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                crate::log_debug!("[IPC 连接] 收到桌面 GUI 客户端接入");

                let cur_mode = *current_mode.lock().unwrap();
                let is_boost = *fan_boost.lock().unwrap();
                let sub_m = *turbo_sub_mode.lock().unwrap();
                let is_rgb_on = *rgb_power.lock().unwrap();
                let (r, g, b) = *rgb_color.lock().unwrap();
                let rgb_bri = *rgb_brightness.lock().unwrap();
                let rgb_eff = *rgb_effect_mode.lock().unwrap();
                let bat_m = *battery_mode.lock().unwrap();
                let tuning_val = custom_tuning.lock().unwrap().clone();
                let ds_val = driver_status.lock().unwrap().clone();

                let is_custom_on = *custom_mode_enabled.lock().unwrap();

                let initial_status = IpcEvent::FullStatus {
                    mode: cur_mode.to_i32(),
                    turbo_sub_mode: sub_m,
                    custom_mode_enabled: is_custom_on,
                    fan_boost: is_boost,
                    fn_lock: LaptopSwitches::get_fn_lock(),
                    win_lock: LaptopSwitches::get_win_lock(),
                    touchpad_disabled: LaptopSwitches::get_touchpad_disabled(),
                    rgb_power: is_rgb_on,
                    rgb_r: r,
                    rgb_g: g,
                    rgb_b: b,
                    rgb_brightness: rgb_bri,
                    rgb_effect_mode: rgb_eff,
                    battery_limit_mode: bat_m.to_i32(),
                    battery_info: BatteryManager::get_battery_info(bat_m),
                    tuning: tuning_val,
                    gpu_mode: GpuModeManager::detect_current_mode().name().to_string(),
                    driver_status: ds_val,
                    acpi_patch_status: AcpiPatchManager::check_status(),
                    hibernate_status: HibernateManager::check_status(),
                };
                if let Ok(json) = serde_json::to_string(&initial_status) {
                    let _ = stream.write_all(format!("{}\n", json).as_bytes());
                }

                let mut client_list = clients.lock().unwrap();
                let stream_clone = match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                client_list.push(stream_clone);
                drop(client_list);

                let clients_for_worker = Arc::clone(&clients);
                let mode_for_worker = Arc::clone(&current_mode);
                let fan_for_worker = Arc::clone(&fan_boost);
                let turbo_sub_worker = Arc::clone(&turbo_sub_mode);
                let custom_mode_worker = Arc::clone(&custom_mode_enabled);
                let rgb_p_worker = Arc::clone(&rgb_power);
                let rgb_c_worker = Arc::clone(&rgb_color);
                let rgb_b_worker = Arc::clone(&rgb_brightness);
                let rgb_e_worker = Arc::clone(&rgb_effect_mode);
                let bat_m_worker = Arc::clone(&battery_mode);
                let tuning_for_worker = Arc::clone(&custom_tuning);
                let ds_for_worker = Arc::clone(&driver_status);
                let cfg_for_worker = Arc::clone(&app_config_storage);
                let profile_mgr_worker = Arc::clone(&profile_mgr);
                let tuning_mgr_worker = Arc::clone(&tuning_mgr);
                let rgb_mgr_worker = Arc::clone(&rgb_mgr);

                thread::spawn(move || {
                    handle_client(
                        stream,
                        clients_for_worker,
                        mode_for_worker,
                        fan_for_worker,
                        turbo_sub_worker,
                        custom_mode_worker,
                        rgb_p_worker,
                        rgb_c_worker,
                        rgb_b_worker,
                        rgb_e_worker,
                        bat_m_worker,
                        tuning_for_worker,
                        ds_for_worker,
                        cfg_for_worker,
                        profile_mgr_worker,
                        tuning_mgr_worker,
                        rgb_mgr_worker,
                    );
                });
            }
            Err(e) => crate::log_error!("[IPC 错误] 接收客户端连接失败: {}", e),
        }
    }

    Ok(())
}

fn handle_client(
    stream: UnixStream,
    clients: Arc<Mutex<Vec<UnixStream>>>,
    current_mode: Arc<Mutex<PerformanceMode>>,
    fan_boost: Arc<Mutex<bool>>,
    turbo_sub_mode: Arc<Mutex<i32>>,
    custom_mode_enabled: Arc<Mutex<bool>>,
    rgb_power: Arc<Mutex<bool>>,
    rgb_color: Arc<Mutex<(u8, u8, u8)>>,
    rgb_brightness: Arc<Mutex<u8>>,
    rgb_effect_mode: Arc<Mutex<i32>>,
    battery_mode: Arc<Mutex<BatteryLimitMode>>,
    custom_tuning: Arc<Mutex<CustomTuningConfig>>,
    driver_status: Arc<Mutex<DriverStatus>>,
    app_config_storage: Arc<Mutex<crate::config::AppConfig>>,
    profile_mgr: Arc<ProfileManager>,
    tuning_mgr: Arc<TuningManager>,
    rgb_mgr: Arc<RgbKeyboard>,
) {
    let reader = BufReader::new(stream);
    for line in reader.lines().flatten() {
        if let Ok(cmd) = serde_json::from_str::<IpcCommand>(&line) {
            match cmd {
                IpcCommand::SetMode { mode } => {
                    let new_mode = PerformanceMode::from_i32(mode);
                    let was_boost = {
                        let mut m = current_mode.lock().unwrap();
                        *m = new_mode;
                        let mut fb = fan_boost.lock().unwrap();
                        let prev_fb = *fb;
                        *fb = false;
                        if new_mode != PerformanceMode::Rage {
                            *turbo_sub_mode.lock().unwrap() = 1;
                            let _ = tuning_mgr.set_silent_turbo(false);
                        }
                        prev_fb
                    };

                    let is_custom = new_mode == PerformanceMode::Custom;
                    let was_custom = {
                        let mut cme = custom_mode_enabled.lock().unwrap();
                        let prev = *cme;
                        *cme = is_custom;
                        prev
                    };

                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.performance_mode = new_mode.to_i32();
                        cfg.fan_boost = false;
                        if new_mode != PerformanceMode::Rage {
                            cfg.turbo_sub_mode = 1;
                        }
                        cfg.mark_dirty();
                    }

                    crate::log_debug!("[面板指令] 切换性能模式 -> {}", new_mode.name());
                    let result_msg = if is_custom {
                        let tuning_cfg = custom_tuning.lock().unwrap().clone();
                        let _ = tuning_mgr.apply_tuning(&tuning_cfg);
                        match profile_mgr.apply_mode(PerformanceMode::Custom) {
                            Ok(msg) => msg,
                            Err(e) => e,
                        }
                    } else {
                        if was_custom {
                            let _ = tuning_mgr.enable_custom_mode(false);
                        }
                        match profile_mgr.apply_mode(new_mode) {
                            Ok(msg) => msg,
                            Err(e) => e,
                        }
                    };

                    broadcast_event(
                        &clients,
                        &IpcEvent::ModeChanged {
                            mode: new_mode.to_i32(),
                            mode_name: new_mode.name().to_string(),
                            source: "桌面控制台面板".to_string(),
                            result: result_msg,
                        },
                    );
                    if was_boost {
                        broadcast_event(
                            &clients,
                            &IpcEvent::FanBoostChanged {
                                enabled: false,
                                message: "切换性能模式，暴风强冷已退出".into(),
                            },
                        );
                    }
                }
                IpcCommand::SetFanBoost { enabled } => {
                    {
                        let mut f = fan_boost.lock().unwrap();
                        *f = enabled;
                    }
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.fan_boost = enabled;
                        cfg.mark_dirty();
                    }
                    let cur_mode = *current_mode.lock().unwrap();
                    crate::log_debug!("\n🖥️【面板指令】切换风扇强冷 (暴风散热) -> {}", enabled);
                    let result_msg = match profile_mgr.set_fan_boost(enabled, cur_mode) {
                        Ok(msg) => msg,
                        Err(e) => e,
                    };

                    let event = IpcEvent::FanBoostChanged {
                        enabled,
                        message: result_msg,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::SetCustomMode { enabled } => {
                    let target = if enabled { PerformanceMode::Custom } else { PerformanceMode::Balanced };
                    *current_mode.lock().unwrap() = target;
                    *custom_mode_enabled.lock().unwrap() = enabled;
                    *fan_boost.lock().unwrap() = false;
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.performance_mode = target.to_i32();
                        cfg.fan_boost = false;
                        cfg.mark_dirty();
                    }
                    let result_msg = if enabled {
                        let tuning_cfg = custom_tuning.lock().unwrap().clone();
                        let _ = tuning_mgr.apply_tuning(&tuning_cfg);
                        profile_mgr.apply_mode(PerformanceMode::Custom).unwrap_or_else(|e| e)
                    } else {
                        let _ = tuning_mgr.enable_custom_mode(false);
                        profile_mgr.apply_mode(PerformanceMode::Balanced).unwrap_or_else(|e| e)
                    };
                    broadcast_event(&clients, &IpcEvent::ModeChanged {
                        mode: target.to_i32(),
                        mode_name: target.name().to_string(),
                        source: "桌面控制台面板".to_string(),
                        result: result_msg,
                    });
                }
                IpcCommand::SetSilentTurbo { enabled } => {
                    let sub_mode = if enabled { 0 } else { 1 };
                    {
                        let mut tsm = turbo_sub_mode.lock().unwrap();
                        *tsm = sub_mode;
                    }
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.turbo_sub_mode = sub_mode;
                        cfg.mark_dirty();
                    }
                    crate::log_debug!("\n🖥️【面板指令】设置静音狂暴模式 -> {}", enabled);
                    let result_msg = match tuning_mgr.set_silent_turbo(enabled) {
                        Ok(_) => format!("静音狂暴已{}", if enabled { "开启 (风扇转速限制 80% 压制啸叫)" } else { "关闭 (全速狂暴)" }),
                        Err(e) => e,
                    };

                    broadcast_event(
                        &clients,
                        &IpcEvent::TurboSubModeChanged {
                            sub_mode,
                            message: result_msg,
                        },
                    );
                }
                IpcCommand::SetTurboSubMode { sub_mode } => {
                    let is_silent = sub_mode == 0;
                    {
                        let mut tsm = turbo_sub_mode.lock().unwrap();
                        *tsm = if is_silent { 0 } else { 1 };
                    }
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.turbo_sub_mode = if is_silent { 0 } else { 1 };
                        cfg.mark_dirty();
                    }
                    crate::log_debug!("\n🖥️【面板指令】设置狂暴子模式 -> {}", if is_silent { "静音狂暴 (限噪80W)" } else { "满血狂暴 (标准全速)" });

                    let result_msg = if is_silent {
                        let _ = tuning_mgr.set_silent_turbo(true);
                        "已激活 [静音狂暴]：风扇转速限制 80% 压制啸叫，80W 功耗平稳输出".to_string()
                    } else {
                        let _ = tuning_mgr.set_silent_turbo(false);
                        "已恢复 [满血狂暴 (标准)]：解除转速限制，阶梯全油门散热".to_string()
                    };

                    broadcast_event(
                        &clients,
                        &IpcEvent::TurboSubModeChanged {
                            sub_mode: if is_silent { 0 } else { 1 },
                            message: result_msg,
                        },
                    );
                }
                IpcCommand::SetRgbColor { r, g, b } => {
                    crate::log_debug!("[面板指令] RGB 颜色 -> R:{} G:{} B:{}", r, g, b);
                    let result_msg = rgb_mgr.set_color(r, g, b).unwrap_or_else(|e| e);
                    let next_eff = {
                        *rgb_power.lock().unwrap() = true;
                        *rgb_color.lock().unwrap() = (r, g, b);
                        let mut em = rgb_effect_mode.lock().unwrap();
                        if *em == 1 {
                            *em = 0;
                        }
                        *em
                    };
                    let bri = {
                        let mut b_lock = rgb_brightness.lock().unwrap();
                        if *b_lock == 0 {
                            *b_lock = 100;
                        }
                        *b_lock
                    };
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.rgb_power = true;
                        cfg.rgb_r = r;
                        cfg.rgb_g = g;
                        cfg.rgb_b = b;
                        cfg.rgb_brightness = bri;
                        cfg.rgb_effect_mode = next_eff;
                        cfg.mark_dirty();
                    }
                    broadcast_event(&clients, &IpcEvent::RgbChanged {
                        enabled: true,
                        r,
                        g,
                        b,
                        brightness: bri,
                        effect_mode: next_eff,
                        message: result_msg,
                    });
                }
                IpcCommand::SetRgbPower { enabled } => {
                    crate::log_debug!("[面板指令] RGB 开关 -> {}", enabled);
                    let result_msg = rgb_mgr.set_power(enabled).unwrap_or_else(|e| e);
                    *rgb_power.lock().unwrap() = enabled;
                    let bri = {
                        let mut b_lock = rgb_brightness.lock().unwrap();
                        if enabled && *b_lock == 0 {
                            *b_lock = 100;
                        }
                        *b_lock
                    };
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.rgb_power = enabled;
                        cfg.rgb_brightness = bri;
                        cfg.mark_dirty();
                    }
                    let (r, g, b) = *rgb_color.lock().unwrap();
                    let eff = *rgb_effect_mode.lock().unwrap();
                    broadcast_event(&clients, &IpcEvent::RgbChanged {
                        enabled,
                        r,
                        g,
                        b,
                        brightness: bri,
                        effect_mode: eff,
                        message: result_msg,
                    });
                }
                IpcCommand::SetRgbBrightness { percent } => {
                    crate::log_debug!("[面板指令] RGB 亮度 -> {}%", percent);
                    let result_msg = rgb_mgr.set_brightness(percent).unwrap_or_else(|e| e);
                    let is_on = percent > 0;
                    *rgb_brightness.lock().unwrap() = percent;
                    *rgb_power.lock().unwrap() = is_on;
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.rgb_brightness = percent;
                        cfg.rgb_power = is_on;
                        cfg.mark_dirty();
                    }
                    let (r, g, b) = *rgb_color.lock().unwrap();
                    let eff = *rgb_effect_mode.lock().unwrap();
                    broadcast_event(&clients, &IpcEvent::RgbChanged {
                        enabled: is_on,
                        r,
                        g,
                        b,
                        brightness: percent,
                        effect_mode: eff,
                        message: result_msg,
                    });
                }
                IpcCommand::SetRgbEffect { effect_mode } => {
                    crate::log_debug!("[面板指令] RGB 灯效 -> {}", effect_mode);
                    let mode = RgbEffectMode::from_i32(effect_mode);
                    let result_msg = rgb_mgr.set_effect(mode).unwrap_or_else(|e| e);
                    *rgb_effect_mode.lock().unwrap() = effect_mode;
                    *rgb_power.lock().unwrap() = true;
                    let bri = {
                        let mut b_lock = rgb_brightness.lock().unwrap();
                        if *b_lock == 0 {
                            *b_lock = 100;
                        }
                        *b_lock
                    };
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.rgb_effect_mode = effect_mode;
                        cfg.rgb_power = true;
                        cfg.rgb_brightness = bri;
                        cfg.mark_dirty();
                    }
                    let (r, g, b) = *rgb_color.lock().unwrap();
                    broadcast_event(&clients, &IpcEvent::RgbChanged {
                        enabled: true,
                        r,
                        g,
                        b,
                        brightness: bri,
                        effect_mode,
                        message: result_msg,
                    });
                }
                IpcCommand::SetBatteryLimit { mode } => {
                    let bat_m = BatteryLimitMode::from_i32(mode);
                    crate::log_debug!("\n🖥️【面板指令】设置电池养护模式 -> {}", bat_m.name());
                    let result_msg = match BatteryManager::set_limit(bat_m) {
                        Ok(msg) => msg,
                        Err(e) => e,
                    };
                    {
                        let mut bm = battery_mode.lock().unwrap();
                        *bm = bat_m;
                    }
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.battery_limit_mode = bat_m.to_i32();
                        cfg.mark_dirty();
                    }
                    let bat_info = BatteryManager::get_battery_info(bat_m);
                    let event = IpcEvent::BatteryLimitChanged {
                        mode: bat_m.to_i32(),
                        mode_name: bat_m.name().to_string(),
                        message: result_msg,
                        battery_info: bat_info,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::SetCustomTuning { config } => {
                    crate::log_debug!(
                        "[面板指令] 应用自定义参数: PL1={}W, PL2={}W, PL4={}W, Boost={}W",
                        config.cpu_pl1, config.cpu_pl2, config.cpu_pl4, config.gpu_dynamic_boost
                    );
                    let result = tuning_mgr.apply_tuning(&config);
                    let msg = match result {
                        Ok(m) => m,
                        Err(e) => e,
                    };
                    {
                        *custom_tuning.lock().unwrap() = config.clone();
                        *current_mode.lock().unwrap() = PerformanceMode::Custom;
                        *custom_mode_enabled.lock().unwrap() = true;
                        *fan_boost.lock().unwrap() = false;
                    }
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.performance_mode = PerformanceMode::Custom.to_i32();
                        cfg.fan_boost = false;
                        cfg.custom_tuning = config.clone();
                        cfg.mark_dirty();
                    }
                    let _ = profile_mgr.apply_mode(PerformanceMode::Custom);

                    broadcast_event(&clients, &IpcEvent::ModeChanged {
                        mode: PerformanceMode::Custom.to_i32(),
                        mode_name: PerformanceMode::Custom.name().to_string(),
                        source: "自定义模式".to_string(),
                        result: "已应用自定义性能参数".to_string(),
                    });
                    broadcast_event(&clients, &IpcEvent::TuningResult { message: msg });
                }
                IpcCommand::SetFnLock { enabled } => {
                    crate::log_debug!("\n🖥️【面板指令】切换 Fn 锁 -> {}", enabled);
                    let result = LaptopSwitches::set_fn_lock(enabled);
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.fn_lock = enabled;
                        cfg.mark_dirty();
                    }
                    let event = IpcEvent::SwitchChanged {
                        switch_name: "fn_lock".to_string(),
                        state: enabled,
                        message: match result {
                            Ok(_) => format!("Fn 功能键锁定已{}", if enabled { "开启" } else { "关闭" }),
                            Err(e) => format!("设置失败: {}", e),
                        },
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::SetWinLock { locked } => {
                    crate::log_debug!("\n🖥️【面板指令】切换 Win 键锁定 -> {}", locked);
                    let result = LaptopSwitches::set_win_lock(locked);
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.win_lock = locked;
                        cfg.mark_dirty();
                    }
                    let event = IpcEvent::SwitchChanged {
                        switch_name: "win_lock".to_string(),
                        state: locked,
                        message: match result {
                            Ok(_) => format!("Windows 键已{}", if locked { "锁定 (游戏防误触)" } else { "解锁" }),
                            Err(e) => format!("设置失败: {}", e),
                        },
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::SetTouchpadDisable { disabled } => {
                    crate::log_debug!("\n🖥️【面板指令】切换触控板状态 -> 禁用: {}", disabled);
                    let result = LaptopSwitches::set_touchpad_disabled(disabled);
                    {
                        let mut cfg = app_config_storage.lock().unwrap();
                        cfg.touchpad_disabled = disabled;
                        cfg.mark_dirty();
                    }
                    let event = IpcEvent::SwitchChanged {
                        switch_name: "touchpad_disabled".to_string(),
                        state: disabled,
                        message: match result {
                            Ok(_) => format!("触摸板已{}", if disabled { "禁用" } else { "启用" }),
                            Err(e) => format!("设置失败: {}", e),
                        },
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::SwitchGpuMode { to_direct } => {
                    let target = if to_direct {
                        GpuOutputMode::Direct
                    } else {
                        GpuOutputMode::Hybrid
                    };
                    crate::log_debug!("\n🖥️【显卡切换】收到请求切换为 -> {}", target.name());
                    let result = GpuModeManager::switch_mode(target);
                    let (msg, cur_str) = match result {
                        Ok(m) => (m, target.name().to_string()),
                        Err(e) => (e, GpuModeManager::detect_current_mode().name().to_string()),
                    };

                    let event = IpcEvent::GpuModeResult {
                        current_mode: cur_str,
                        message: msg,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::SetDgpuRuntimePower { power_on } => {
                    crate::log_debug!("\n🖥️【显卡电源】切换独显运行时 ACPI 供电 -> 供电开启: {}", power_on);
                    let (msg, cur_str) = match GpuModeManager::set_dgpu_runtime_power(power_on) {
                        Ok(m) => (m, if power_on { "混合输出 (独显就绪)".to_string() } else { "纯核显省电 (独显断电)".to_string() }),
                        Err(e) => (e, GpuModeManager::detect_current_mode().name().to_string()),
                    };

                    let event = IpcEvent::GpuModeResult {
                        current_mode: cur_str,
                        message: msg,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::InstallDrivers => {
                    crate::log_debug!("\n🖥️【面板指令】收到驱动一键安装部署请求");
                    let (success, msg) = match DriverManager::install_and_setup() {
                        Ok(m) => (true, m),
                        Err(e) => (false, e),
                    };
                    let new_ds = DriverManager::check_status();
                    {
                        let mut ds = driver_status.lock().unwrap();
                        *ds = new_ds.clone();
                    }
                    let event = IpcEvent::DriverInstallResult {
                        success,
                        message: msg,
                        status: new_ds,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::InstallAcpiPatches => {
                    crate::log_debug!("\n🖥️【ACPI补丁】收到一键安装硬件睡眠与无声修复补丁请求...");
                    let (success, msg) = match AcpiPatchManager::install_all_patches() {
                        Ok(m) => (true, m),
                        Err(e) => (false, e),
                    };
                    let st = AcpiPatchManager::check_status();
                    let event = IpcEvent::AcpiPatchResult {
                        success,
                        message: msg,
                        status: st,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::RollbackAcpiPatches => {
                    crate::log_debug!("\n🖥️【ACPI补丁】收到一键回滚补丁请求...");
                    let (success, msg) = match AcpiPatchManager::rollback_patches() {
                        Ok(m) => (true, m),
                        Err(e) => (false, e),
                    };
                    let st = AcpiPatchManager::check_status();
                    let event = IpcEvent::AcpiPatchResult {
                        success,
                        message: msg,
                        status: st,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::InstallHibernateFix => {
                    crate::log_debug!("\n🖥️【休眠防黑屏】收到一键加固 S4 磁盘休眠唤醒防黑屏全链路请求...");
                    let (success, msg) = match HibernateManager::install_hibernate_fix() {
                        Ok(m) => (true, m),
                        Err(e) => (false, e),
                    };
                    let st = HibernateManager::check_status();
                    let event = IpcEvent::HibernateFixResult {
                        success,
                        message: msg,
                        status: st,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::RollbackHibernateFix => {
                    crate::log_debug!("\n🖥️【休眠防黑屏】收到回滚休眠配置请求...");
                    let (success, msg) = match HibernateManager::rollback_hibernate_fix() {
                        Ok(m) => (true, m),
                        Err(e) => (false, e),
                    };
                    let st = HibernateManager::check_status();
                    let event = IpcEvent::HibernateFixResult {
                        success,
                        message: msg,
                        status: st,
                    };
                    broadcast_event(&clients, &event);
                }
                IpcCommand::ExitAndRestoreHardware => {
                    flush_pending_config();
                    restore_hardware_independent_mode();
                    crate::log_info!("[Daemon] 硬件脱机还原完毕，守护服务退出");
                    let _ = std::fs::remove_file(crate::ipc::IPC_SOCKET_PATH);
                    std::process::exit(0);
                }
                IpcCommand::GetStatus => {
                    let cur = *current_mode.lock().unwrap();
                    let is_boost = *fan_boost.lock().unwrap();
                    let sub_m = *turbo_sub_mode.lock().unwrap();
                    let is_rgb_on = *rgb_power.lock().unwrap();
                    let (r, g, b) = *rgb_color.lock().unwrap();
                    let rgb_bri = *rgb_brightness.lock().unwrap();
                    let rgb_eff = *rgb_effect_mode.lock().unwrap();
                    let bat_m = *battery_mode.lock().unwrap();
                    let tuning_val = custom_tuning.lock().unwrap().clone();
                    let ds_val = driver_status.lock().unwrap().clone();

                    let is_custom_on = *custom_mode_enabled.lock().unwrap();

                    let status = IpcEvent::FullStatus {
                        mode: cur.to_i32(),
                        turbo_sub_mode: sub_m,
                        custom_mode_enabled: is_custom_on,
                        fan_boost: is_boost,
                        fn_lock: LaptopSwitches::get_fn_lock(),
                        win_lock: LaptopSwitches::get_win_lock(),
                        touchpad_disabled: LaptopSwitches::get_touchpad_disabled(),
                        rgb_power: is_rgb_on,
                        rgb_r: r,
                        rgb_g: g,
                        rgb_b: b,
                        rgb_brightness: rgb_bri,
                        rgb_effect_mode: rgb_eff,
                        battery_limit_mode: bat_m.to_i32(),
                        battery_info: BatteryManager::get_battery_info(bat_m),
                        tuning: tuning_val,
                        gpu_mode: GpuModeManager::detect_current_mode().name().to_string(),
                        driver_status: ds_val,
                        acpi_patch_status: AcpiPatchManager::check_status(),
                        hibernate_status: HibernateManager::check_status(),
                    };
                    broadcast_event(&clients, &status);
                }
                IpcCommand::WakeupGui => {
                    crate::log_debug!("\n🖥️【唤醒指令】收到唤醒前端 UI 界面请求，广播唤醒事件");
                    broadcast_event(&clients, &IpcEvent::WakeupRequested);
                }
            }
        }
    }
}

fn broadcast_event(clients: &Arc<Mutex<Vec<UnixStream>>>, event: &IpcEvent) {
    let Ok(json_str) = serde_json::to_string(event) else { return };
    let msg = format!("{}\n", json_str);

    let mut list = clients.lock().unwrap();
    list.retain_mut(|client| {
        let _ = client.set_write_timeout(Some(std::time::Duration::from_millis(50)));
        client.write_all(msg.as_bytes()).is_ok()
    });
}

pub fn restore_hardware_independent_mode() {
    crate::log_info!("[Daemon] 执行硬件脱机还原...");

    let tuning_mgr = TuningManager::new();
    let profile_mgr = ProfileManager::new();

    let _ = tuning_mgr.set_silent_turbo(false);
    let _ = tuning_mgr.enable_custom_mode(false);
    let _ = profile_mgr.set_fan_boost(false, PerformanceMode::Balanced);
    let _ = profile_mgr.apply_mode(PerformanceMode::Balanced);
    let _ = LaptopSwitches::force_unlock_win_key();
    let _ = LaptopSwitches::set_touchpad_disabled(false);
    let _ = LaptopSwitches::set_fn_lock(false);
    TuningManager::reset_gpu_offsets();
    let _ = GpuModeManager::set_dgpu_runtime_power(true);

    use crate::ec::{EcInterface, registers};
    let ec = EcInterface::new();
    let _ = ec.write(registers::ADDR_AP_OEM, registers::FLAG_AP_OEM_OFFLINE);
    crate::log_debug!("[Daemon] 硬件独立模式还原完毕 (0x0741=0x00)");
}
