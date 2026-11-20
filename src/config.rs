// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use crate::battery::BatteryLimitMode;
use crate::tuning::CustomTuningConfig;
use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub performance_mode: i32,
    pub fan_boost: bool,
    pub fn_lock: bool,
    pub win_lock: bool,
    pub touchpad_disabled: bool,
    pub rgb_power: bool,
    pub rgb_color_idx: i32,
    pub rgb_r: u8,
    pub rgb_g: u8,
    pub rgb_b: u8,
    pub rgb_brightness: u8,
    pub rgb_effect_mode: i32,
    pub battery_limit_mode: i32,
    #[serde(default = "default_turbo_sub_mode")]
    pub turbo_sub_mode: i32,
    pub custom_tuning: CustomTuningConfig,

    #[serde(skip)]
    dirty: Arc<AtomicBool>,
}

impl AppConfig {
    pub fn is_silent_turbo(&self) -> bool {
        self.turbo_sub_mode == 0
    }
}

fn default_turbo_sub_mode() -> i32 {
    1
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            performance_mode: 1,
            fan_boost: false,
            fn_lock: false,
            win_lock: false,
            touchpad_disabled: false,
            rgb_power: true,
            rgb_color_idx: 0,
            rgb_r: 255,
            rgb_g: 255,
            rgb_b: 255,
            rgb_brightness: 100,
            rgb_effect_mode: 0,
            battery_limit_mode: BatteryLimitMode::Standard.to_i32(),
            turbo_sub_mode: 1,
            custom_tuning: CustomTuningConfig::default(),
            dirty: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl AppConfig {
    pub fn detect_user_home() -> Option<(PathBuf, u32, u32)> {
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            if !sudo_user.is_empty() && sudo_user != "root" {
                if let Ok(c_user) = std::ffi::CString::new(sudo_user) {
                    unsafe {
                        let pwd = libc::getpwnam(c_user.as_ptr());
                        if !pwd.is_null() && !(*pwd).pw_dir.is_null() {
                            if let Ok(s) = CStr::from_ptr((*pwd).pw_dir).to_str() {
                                return Some((PathBuf::from(s), (*pwd).pw_uid, (*pwd).pw_gid));
                            }
                        }
                    }
                }
            }
        }

        let is_root = unsafe { libc::geteuid() == 0 };
        if !is_root {
            if let Ok(home) = std::env::var("HOME") {
                if !home.is_empty() && home != "/root" {
                    let uid = unsafe { libc::geteuid() };
                    let gid = unsafe { libc::getegid() };
                    return Some((PathBuf::from(home), uid, gid));
                }
            }
        }

        if let Ok(entries) = fs::read_dir("/run/user") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if let Ok(uid) = name.parse::<u32>() {
                        if uid >= 1000 {
                            unsafe {
                                let pwd = libc::getpwuid(uid);
                                if !pwd.is_null() && !(*pwd).pw_dir.is_null() {
                                    if let Ok(s) = CStr::from_ptr((*pwd).pw_dir).to_str() {
                                        return Some((PathBuf::from(s), (*pwd).pw_uid, (*pwd).pw_gid));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_config_path() -> PathBuf {
        if unsafe { libc::geteuid() == 0 } {
            PathBuf::from("/etc/rmechrevo/config.json")
        } else if let Some((home, _, _)) = Self::detect_user_home() {
            home.join(".config/rmechrevo/config.json")
        } else {
            PathBuf::from("/etc/rmechrevo/config.json")
        }
    }

    fn legacy_user_config_path() -> Option<PathBuf> {
        if let Some((home, _, _)) = Self::detect_user_home() {
            let p = home.join(".config/rmechrevo/config.json");
            if p.exists() {
                return Some(p);
            }
        }
        if let Ok(entries) = fs::read_dir("/home") {
            for entry in entries.flatten() {
                let p = entry.path().join(".config/rmechrevo/config.json");
                if p.exists() {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn load() -> Self {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    crate::log_debug!("[配置持久化] 已读取配置: {:?}", path);
                    return AppConfig {
                        dirty: Arc::new(AtomicBool::new(false)),
                        ..cfg
                    };
                }
            }
        }

        if let Some(legacy_path) = Self::legacy_user_config_path() {
            if legacy_path != path {
                if let Ok(content) = fs::read_to_string(&legacy_path) {
                    if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                        let migrated = AppConfig {
                            dirty: Arc::new(AtomicBool::new(false)),
                            ..cfg
                        };
                        let _ = migrated.save();
                        crate::log_info!("[配置持久化] 已从 {:?} 迁移配置至 {:?}", legacy_path, path);
                        return migrated;
                    }
                }
            }
        }

        crate::log_debug!("[配置持久化] 首次运行，写入默认配置: {:?}", path);
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    pub fn save_if_dirty(&self) -> Result<(), String> {
        if !self.dirty.swap(false, Ordering::AcqRel) {
            return Ok(());
        }
        if let Err(e) = self.save_internal() {
            self.dirty.store(true, Ordering::Release);
            return Err(e);
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        self.dirty.store(false, Ordering::Release);
        self.save_internal()
    }

    fn save_internal(&self) -> Result<(), String> {
        let path = Self::get_config_path();
        let is_system_cfg = path.starts_with("/etc/");
        let user_info = if is_system_cfg { None } else { Self::detect_user_home() };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
            if let Some((_, uid, gid)) = user_info {
                unsafe {
                    if libc::geteuid() == 0 {
                        if let Ok(c_path) = std::ffi::CString::new(parent.to_string_lossy().to_string()) {
                            libc::chown(c_path.as_ptr(), uid, gid);
                        }
                    }
                }
            }
        }

        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, json_str)
            .map_err(|e| format!("写入临时配置 {:?} 失败: {}", tmp_path, e))?;
        fs::rename(&tmp_path, &path)
            .map_err(|e| format!("落盘配置文件 {:?} 失败: {}", path, e))?;

        if let Some((_, uid, gid)) = user_info {
            unsafe {
                if libc::geteuid() == 0 {
                    if let Ok(c_path) = std::ffi::CString::new(path.to_string_lossy().to_string()) {
                        libc::chown(c_path.as_ptr(), uid, gid);
                    }
                }
            }
        }

        crate::log_debug!("[配置持久化] 已保存配置: {:?}", path);
        Ok(())
    }
}
