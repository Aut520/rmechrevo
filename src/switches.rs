// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs;
use std::io::Write;
use std::path::Path;

pub struct LaptopSwitches;

impl LaptopSwitches {
    const BASE_PATH: &'static str = "/sys/bus/platform/devices/INOU0000:00";
    const ACPI_CALL_PATH: &'static str = "/proc/acpi/call";

    // 核心硬件 EC 寄存器 (基于原厂 Define.ECSpec 与 uniwill-laptop 内核驱动逆向)
    const ADDR_TRIGGER_BYTE: u16 = 0x0767;      // bit 0: WinLock_Trigger (0x01 脉冲触发翻转)
    const ADDR_STATUS_BYTE: u16 = 0x0768;       // bit 0: WinLock 硬件状态 (1: 锁定, 0: 正常启用)
    const ADDR_FN_LOCK: u16 = 0x074E;           // bit 4: FnLock 硬件状态 (0x10)
    const ADDR_TOUCHPAD: u16 = 0x07A6;          // bit 6: 触控板禁用标志 (0x40, 1: 禁用, 0: 启用)

    pub fn is_available() -> bool {
        Path::new(Self::ACPI_CALL_PATH).exists() || Path::new(Self::BASE_PATH).exists()
    }

    pub fn ec_read(addr: u16) -> Result<u8, String> {
        let path = Path::new(Self::ACPI_CALL_PATH);
        if !path.exists() {
            return Err("未挂载 /proc/acpi/call".into());
        }

        let cmd = format!("\\_SB.INOU.ECRR 0x{:04X}\0", addr);
        fs::write(Self::ACPI_CALL_PATH, cmd.as_bytes())
            .map_err(|e| format!("写入 EC 读取失败: {}", e))?;

        let res = fs::read_to_string(Self::ACPI_CALL_PATH)
            .map_err(|e| format!("读取 EC 结果失败: {}", e))?;
        let s = res.trim().trim_matches('\0');

        if let Some(hex_part) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            u8::from_str_radix(hex_part, 16).map_err(|e| format!("解析十六进制失败: {}", e))
        } else {
            s.parse::<u8>().map_err(|e| format!("解析十进制失败: {}", e))
        }
    }

    pub fn ec_write(addr: u16, val: u8) -> Result<(), String> {
        let path = Path::new(Self::ACPI_CALL_PATH);
        if !path.exists() {
            return Err("未挂载 /proc/acpi/call".into());
        }

        let cmd = format!("\\_SB.INOU.ECRW 0x{:04X} 0x{:02X}\0", addr, val);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(Self::ACPI_CALL_PATH)
            .map_err(|e| format!("打开 /proc/acpi/call 失败: {}", e))?;

        file.write_all(cmd.as_bytes())
            .map_err(|e| format!("写入 EC 0x{:04X} 失败: {}", addr, e))?;

        Ok(())
    }

    pub fn get_fn_lock() -> bool {
        if let Ok(val) = Self::ec_read(Self::ADDR_FN_LOCK) {
            return (val & 0x10) != 0;
        }
        fs::read_to_string(format!("{}/fn_lock", Self::BASE_PATH))
            .map(|s| s.trim() == "1")
            .unwrap_or(false)
    }

    pub fn set_fn_lock(enabled: bool) -> Result<(), String> {
        let val_str = if enabled { "1" } else { "0" };
        let _ = fs::write(format!("{}/fn_lock", Self::BASE_PATH), val_str);

        if let Ok(cur) = Self::ec_read(Self::ADDR_FN_LOCK) {
            let new_val = if enabled { cur | 0x10 } else { cur & !0x10 };
            let _ = Self::ec_write(Self::ADDR_FN_LOCK, new_val);
        }
        Ok(())
    }

    /// 获取 Win 键锁定状态 (硬件直通：0x0768 bit 0 -> 1: 锁定，0: 启用)
    pub fn get_win_lock() -> bool {
        if let Ok(val) = Self::ec_read(Self::ADDR_STATUS_BYTE) {
            let locked = (val & 0x01) != 0;
            return locked;
        }

        fs::read_to_string(format!("{}/super_key_enable", Self::BASE_PATH))
            .map(|s| s.trim() == "0")
            .unwrap_or(false)
    }

    /// 设置 Win 键锁定（防误触与解除锁死：双通道保障 sysfs 与 EC 0x0767 硬件同步）
    pub fn set_win_lock(locked: bool) -> Result<(), String> {
        let sysfs_path = format!("{}/super_key_enable", Self::BASE_PATH);
        if Path::new(&sysfs_path).exists() {
            let sysfs_val = if locked { "0\n" } else { "1\n" };
            let _ = fs::write(&sysfs_path, sysfs_val);
            crate::log_debug!("[硬件开关] 已同步写入 uniwill-laptop sysfs super_key_enable: {}", if locked { "0 (锁定)" } else { "1 (启用)" });
        }

        if let Ok(cur_val) = Self::ec_read(Self::ADDR_STATUS_BYTE) {
            let hw_locked = (cur_val & 0x01) != 0;
            if hw_locked != locked {
                crate::log_debug!("[硬件开关] EC 0x0768 状态 (0x{:02X}, locked={}) 与目标 locked={} 不一致，触发 0x0767 翻转脉冲...", cur_val, hw_locked, locked);
                // 核心硬件触发脉冲：向 0x0767 注入 bit 0
                let cur_67 = Self::ec_read(Self::ADDR_TRIGGER_BYTE).unwrap_or(0);
                let _ = Self::ec_write(Self::ADDR_TRIGGER_BYTE, (cur_67 & 0xFE) | 0x01);
                std::thread::sleep(std::time::Duration::from_millis(60));
                let _ = Self::ec_write(Self::ADDR_TRIGGER_BYTE, cur_67 & 0xFE);
                std::thread::sleep(std::time::Duration::from_millis(60));

                if let Ok(new_val) = Self::ec_read(Self::ADDR_STATUS_BYTE) {
                    crate::log_debug!("[硬件开关] 脉冲注入后 EC 0x0768 状态: 0x{:02X} (locked={})", new_val, (new_val & 0x01) != 0);
                }
            }
        }

        crate::log_debug!("[硬件开关] Win 键锁定状态设定完毕: locked={}", locked);
        Ok(())
    }

    pub fn force_unlock_win_key() -> Result<(), String> {
        let sysfs_path = format!("{}/super_key_enable", Self::BASE_PATH);
        if Path::new(&sysfs_path).exists() {
            let _ = fs::write(&sysfs_path, "1\n");
        }

        if let Ok(val) = Self::ec_read(Self::ADDR_STATUS_BYTE) {
            if (val & 0x01) != 0 {
                crate::log_debug!("[硬件开关] 探测到 EC 硬件处于 Win 键锁定状态 (0x0768=0x{:02X})，正在强制下发解锁脉冲...", val);
                let cur_67 = Self::ec_read(Self::ADDR_TRIGGER_BYTE).unwrap_or(0);
                let _ = Self::ec_write(Self::ADDR_TRIGGER_BYTE, (cur_67 & 0xFE) | 0x01);
                std::thread::sleep(std::time::Duration::from_millis(60));
                let _ = Self::ec_write(Self::ADDR_TRIGGER_BYTE, cur_67 & 0xFE);
                std::thread::sleep(std::time::Duration::from_millis(60));
            }
        }

        crate::log_debug!("[硬件开关] Win 键已强制恢复至解锁状态");
        Ok(())
    }

    /// 获取触控板禁用状态 (优先检查底层 EC 0x07A6 与 sysfs)
    pub fn get_touchpad_disabled() -> bool {
        if let Ok(val) = Self::ec_read(Self::ADDR_TOUCHPAD) {
            return (val & 0x40) != 0;
        }

        if let Ok(s) = fs::read_to_string(format!("{}/touchpad_toggle_enable", Self::BASE_PATH)) {
            return s.trim() == "0";
        }

        false
    }

    pub fn set_touchpad_disabled(disabled: bool) -> Result<(), String> {
        let val_str = if disabled { "0" } else { "1" };
        let _ = fs::write(format!("{}/touchpad_toggle_enable", Self::BASE_PATH), val_str);

        if let Ok(cur) = Self::ec_read(Self::ADDR_TOUCHPAD) {
            let target = if disabled {
                cur | 0x40
            } else {
                cur & !0x40
            };
            if cur != target {
                let _ = Self::ec_write(Self::ADDR_TOUCHPAD, target);
            }
        }

        let hypr_en = if disabled { "false" } else { "true" };
        if let Ok(entries) = fs::read_dir("/run/user") {
            for entry in entries.flatten() {
                let hypr_dir = entry.path().join("hypr");
                if hypr_dir.exists() {
                    if let Ok(hypr_entries) = fs::read_dir(&hypr_dir) {
                        for he in hypr_entries.flatten() {
                            let sig = he.file_name().to_string_lossy().to_string();
                            if !sig.starts_with('.') {
                                let mut cmd = std::process::Command::new("hyprctl");
                                cmd.env("HYPRLAND_INSTANCE_SIGNATURE", &sig);
                                cmd.env("XDG_RUNTIME_DIR", entry.path());
                                cmd.args(["keyword", "device:uniw0001:00-093a:0255-touchpad:enabled", hypr_en]);
                                let _ = cmd.output();
                                break;
                            }
                        }
                    }
                }
            }
        }

        crate::log_debug!("[触控板] 成功设置触控板状态: disabled={}", disabled);
        Ok(())
    }
}
