// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLimitMode {
    Standard = 0,
    Balanced = 1,
    Healthy = 2,
}

impl BatteryLimitMode {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => BatteryLimitMode::Balanced,
            2 => BatteryLimitMode::Healthy,
            _ => BatteryLimitMode::Standard,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }

    pub fn name(&self) -> &'static str {
        match self {
            BatteryLimitMode::Standard => "常规充满 (100%)",
            BatteryLimitMode::Balanced => "日常平衡 (80%)",
            BatteryLimitMode::Healthy => "长效养护 (60%)",
        }
    }

    pub fn thresholds(&self) -> (u8, u8) {
        match self {
            BatteryLimitMode::Standard => (100, 95),
            BatteryLimitMode::Balanced => (80, 75),
            BatteryLimitMode::Healthy => (60, 50),
        }
    }

    pub fn mode_byte(&self) -> u8 {
        match self {
            BatteryLimitMode::Healthy => 0x01,
            BatteryLimitMode::Balanced => 0x02,
            BatteryLimitMode::Standard => 0x00,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub capacity: u8,
    pub status: String,
    pub is_ac_online: bool,
    pub voltage: f32,
    pub current_ma: i32,
    pub power_w: f32,
    pub health_percent: u8,
    pub cycle_count: u32,
    pub charge_now_mah: u32,
    pub charge_full_mah: u32,
    pub charge_design_mah: u32,
    pub limit_mode: i32,
    #[serde(default)]
    pub limit_mode_reg: u8,
    pub limit_up: u8,
    pub limit_down: u8,
    pub hardware_status_msg: String,
}

pub struct BatteryManager;

impl BatteryManager {
    const ACPI_CALL_PATH: &'static str = "/proc/acpi/call";
    const ADDR_LIMIT_MODE: u16 = 0x074F; // 原厂硬件模式锁存通道 (0x01: 60%, 0x02: 80%, 0x00: 100%)
    const ADDR_LIMIT_UP: u16 = 0x07B9;
    const ADDR_LIMIT_DOWN: u16 = 0x07D0;

    /// 向 EC 写入充电上限与保护门限 (双通道物理下发：0x074F 模式锁存 + 0x07B9/0x07D0 阈值)
    pub fn set_limit(mode: BatteryLimitMode) -> Result<String, String> {
        let (up, down) = mode.thresholds();
        let mode_byte = mode.mode_byte();

        Self::ec_write(Self::ADDR_LIMIT_MODE, mode_byte)?;
        Self::ec_write(Self::ADDR_LIMIT_UP, up)?;
        Self::ec_write(Self::ADDR_LIMIT_DOWN, down)?;

        let readback_mode = Self::ec_read(Self::ADDR_LIMIT_MODE).unwrap_or(mode_byte);
        let readback_up = Self::ec_read(Self::ADDR_LIMIT_UP).unwrap_or(up);
        let readback_down = Self::ec_read(Self::ADDR_LIMIT_DOWN).unwrap_or(down);

        let msg = format!(
            "已成功向 EC 双通道下发硬件限充参数：{} (模式锁存: 0x{:02X} [0x074F], 充电上限: {}% [0x07B9=0x{:02X}], 保护下限: {}% [0x07D0=0x{:02X}])",
            mode.name(),
            readback_mode,
            readback_up,
            readback_up,
            readback_down,
            readback_down
        );
        println!("[电池养护] {}", msg);
        Ok(msg)
    }

    pub fn get_limit() -> BatteryLimitMode {
        if let Ok(mode_byte) = Self::ec_read(Self::ADDR_LIMIT_MODE) {
            match mode_byte {
                0x01 => return BatteryLimitMode::Healthy,
                0x02 => return BatteryLimitMode::Balanced,
                0x00 => return BatteryLimitMode::Standard,
                _ => {}
            }
        }
        if let Ok(up) = Self::ec_read(Self::ADDR_LIMIT_UP) {
            if up <= 65 && up > 0 {
                return BatteryLimitMode::Healthy;
            } else if up <= 85 && up > 65 {
                return BatteryLimitMode::Balanced;
            } else if up > 85 {
                return BatteryLimitMode::Standard;
            }
        }
        BatteryLimitMode::Standard
    }

    pub fn get_battery_info(current_mode: BatteryLimitMode) -> BatteryInfo {
        let bat_base = Path::new("/sys/class/power_supply/BAT0");
        let ac_base = Path::new("/sys/class/power_supply/AC0");

        let mut capacity: u8 = 100;
        let mut raw_status = "Not charging".to_string();
        let mut voltage_uv: u64 = 16900000;
        let mut current_ua: i64 = 0;
        let mut charge_now_uah: u64 = 5200000;
        let mut charge_full_uah: u64 = 5200000;
        let mut charge_full_design_uah: u64 = 5200000;
        let mut cycle_count: u32 = 0;

        if bat_base.exists() {
            if let Ok(s) = fs::read_to_string(bat_base.join("capacity")) {
                capacity = s.trim().parse().unwrap_or(100);
            }
            if let Ok(s) = fs::read_to_string(bat_base.join("status")) {
                raw_status = s.trim().to_string();
            }
            if let Ok(s) = fs::read_to_string(bat_base.join("voltage_now")) {
                voltage_uv = s.trim().parse().unwrap_or(16900000);
            }
            if let Ok(s) = fs::read_to_string(bat_base.join("current_now")) {
                current_ua = s.trim().parse().unwrap_or(0);
            }
            if let Ok(s) = fs::read_to_string(bat_base.join("charge_now")) {
                charge_now_uah = s.trim().parse().unwrap_or(5200000);
            }
            if let Ok(s) = fs::read_to_string(bat_base.join("charge_full")) {
                charge_full_uah = s.trim().parse().unwrap_or(5200000);
            }
            if let Ok(s) = fs::read_to_string(bat_base.join("charge_full_design")) {
                charge_full_design_uah = s.trim().parse().unwrap_or(5200000);
            }
            if let Ok(s) = fs::read_to_string(bat_base.join("cycle_count")) {
                cycle_count = s.trim().parse().unwrap_or(0);
            }
        }

        let is_ac_online = if ac_base.exists() {
            fs::read_to_string(ac_base.join("online"))
                .map(|s| s.trim() == "1")
                .unwrap_or(true)
        } else {
            true
        };

        let voltage = (voltage_uv as f32) / 1_000_000.0;
        let current_ma = (current_ua / 1000) as i32;
        let power_w = voltage * (current_ma.abs() as f32 / 1000.0);

        let health_percent = if charge_full_design_uah > 0 {
            ((charge_full_uah as f32 / charge_full_design_uah as f32) * 100.0).clamp(0.0, 100.0) as u8
        } else {
            100
        };

        let (up_target, down_target) = current_mode.thresholds();
        let limit_mode_reg = Self::ec_read(Self::ADDR_LIMIT_MODE).unwrap_or(current_mode.mode_byte());
        let limit_up = Self::ec_read(Self::ADDR_LIMIT_UP).unwrap_or(up_target);
        let limit_down = Self::ec_read(Self::ADDR_LIMIT_DOWN).unwrap_or(down_target);

        let (status_cn, hardware_msg) = match raw_status.as_str() {
            "Charging" => (
                "正在充电".to_string(),
                format!("EC 正在充电中，达到设定上限 ({}%) 时主板将自动切断充电回路", limit_up)
            ),
            "Discharging" => (
                "电池放电中".to_string(),
                "已断开外部适配器，系统当前完全依赖动力电池供电运行".to_string()
            ),
            "Full" => (
                "已充满".to_string(),
                "电池电量已达上限，EC 处于停止充电维持模式".to_string()
            ),
            _ => {
                if is_ac_online && capacity >= limit_up && limit_up < 100 {
                    (
                        "未充电 (适配器直通)".to_string(),
                        format!(
                            "当前电量 ({}%) 高于养护设定阈值 ({}%)：EC 已物理切断电池充电通路，由电源适配器直接给主板供电保护电芯。待使用或自然放电至 {}% 以下再次插电，将锁定在该养护区间。",
                            capacity, limit_up, limit_up
                        )
                    )
                } else if is_ac_online {
                    (
                        "未充电 (电源接通)".to_string(),
                        "电源已接通，当前未在充电".to_string()
                    )
                } else {
                    (
                        "未充电".to_string(),
                        "电池处于空闲待机状态".to_string()
                    )
                }
            }
        };

        BatteryInfo {
            capacity,
            status: status_cn,
            is_ac_online,
            voltage,
            current_ma,
            power_w,
            health_percent,
            cycle_count,
            charge_now_mah: (charge_now_uah / 1000) as u32,
            charge_full_mah: (charge_full_uah / 1000) as u32,
            charge_design_mah: (charge_full_design_uah / 1000) as u32,
            limit_mode: current_mode.to_i32(),
            limit_mode_reg,
            limit_up,
            limit_down,
            hardware_status_msg: hardware_msg,
        }
    }

    fn ec_read(addr: u16) -> Result<u8, String> {
        let path = Path::new(Self::ACPI_CALL_PATH);
        if !path.exists() {
            return Err("未挂载 /proc/acpi/call".into());
        }
        let cmd = format!("\\_SB.INOU.ECRR 0x{:04X}\0", addr);
        fs::write(path, cmd.as_bytes()).map_err(|e| format!("写入 EC 读取失败: {}", e))?;
        let res = fs::read_to_string(path).map_err(|e| format!("读取 EC 结果失败: {}", e))?;
        let s = res.trim().trim_matches('\0');
        if let Some(hex_part) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            u8::from_str_radix(hex_part, 16).map_err(|e| format!("解析十六进制失败: {}", e))
        } else {
            s.parse::<u8>().map_err(|e| format!("解析十进制失败: {}", e))
        }
    }

    fn ec_write(addr: u16, val: u8) -> Result<(), String> {
        let path = Path::new(Self::ACPI_CALL_PATH);
        if !path.exists() {
            return Err("未挂载 /proc/acpi/call，无法向 EC 下发电池阈值".into());
        }

        let cmd = format!("\\_SB.INOU.ECRW 0x{:04X} 0x{:02X}\0", addr, val);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| format!("打开 /proc/acpi/call 失败 (请确认以 root 运行守护进程): {}", e))?;

        file.write_all(cmd.as_bytes())
            .map_err(|e| format!("写入 EC 0x{:04X} 失败: {}", addr, e))?;

        Ok(())
    }
}
