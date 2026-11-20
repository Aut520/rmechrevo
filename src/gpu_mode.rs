// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs;
use std::path::Path;
use std::process::Command;

pub const EFIVAR_PATH: &str =
    "/sys/firmware/efi/efivars/OemMagicVariable-9f33f85c-13ca-4fd1-9c4a-96217722c593";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuOutputMode {
    Hybrid,
    Direct,
    Unknown,
}

impl GpuOutputMode {
    pub fn name(&self) -> &'static str {
        match self {
            GpuOutputMode::Hybrid => "混合输出 (Hybrid)",
            GpuOutputMode::Direct => "独显直连 (dGPU Only)",
            GpuOutputMode::Unknown => "未知模式",
        }
    }
}

pub struct GpuModeManager;

impl GpuModeManager {
    pub fn detect_current_mode() -> GpuOutputMode {
        // 若连接到 AMD (0x1002) 或 Intel (0x8086)，则是混合输出 (Hybrid)
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("eDP") {
                    let status_path = entry.path().join("status");
                    if let Ok(status) = fs::read_to_string(&status_path) {
                        if status.trim() == "connected" {
                            let real_card_dir = match fs::canonicalize(entry.path()) {
                                Ok(p) => p.parent().map(|p| p.to_path_buf()),
                                Err(_) => None,
                            };
                            if let Some(card_dir) = real_card_dir {
                                let vendor_path = card_dir.join("device/vendor");
                                if let Ok(vendor_str) = fs::read_to_string(&vendor_path) {
                                    let v = vendor_str.trim().to_lowercase();
                                    if v.contains("0x10de") {
                                        return GpuOutputMode::Direct;
                                    } else if v.contains("0x1002") || v.contains("0x8086") {
                                        return GpuOutputMode::Hybrid;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Ok(data) = fs::read(EFIVAR_PATH) {
            if data.len() >= 0x67 {
                match data[0x66] {
                    0x01 => return GpuOutputMode::Direct,
                    0x00 => return GpuOutputMode::Hybrid,
                    _ => {}
                }
            }
        }

        GpuOutputMode::Unknown
    }

    pub fn switch_mode(target: GpuOutputMode) -> Result<String, String> {
        let path = Path::new(EFIVAR_PATH);
        if !path.exists() {
            return Err("找不到 OemMagicVariable NVRAM 节点，系统可能未以 UEFI 启动或未挂载 efivarfs".into());
        }

        let mut data = fs::read(path).map_err(|e| format!("读取 efivar 失败: {}", e))?;
        if data.len() != 184 {
            return Err(format!("efivar 长度异常: {} 字节 (预期 184 字节)，中止修改以保护主板", data.len()));
        }

        let target_byte = match target {
            GpuOutputMode::Hybrid => 0x00,
            GpuOutputMode::Direct => 0x01,
            GpuOutputMode::Unknown => return Err("无效的目标模式".into()),
        };

        let old_byte = data[0x66];
        if old_byte == target_byte {
            return Ok(format!("当前已是 {}，无需改动", target.name()));
        }

        let _ = Command::new("chattr").args(["-i", EFIVAR_PATH]).status();

        data[0x66] = target_byte;

        fs::write(path, &data).map_err(|e| format!("写入 efivar 失败: {}", e))?;

        let _ = Command::new("chattr").args(["+i", EFIVAR_PATH]).status();

        Ok(format!(
            "成功修改显卡模式为 {} (0x66: 0x{:02x} -> 0x{:02x})。注意：必须重启电脑后生效！",
            target.name(),
            old_byte,
            target_byte
        ))
    }

    pub fn set_dgpu_runtime_power(power_on: bool) -> Result<String, String> {
        let acpi_call = Path::new("/proc/acpi/call");
        if !acpi_call.exists() {
            return Err("未挂载 /proc/acpi/call，无法通过 ACPI 控制独显供电".into());
        }

        if !power_on {
            let _ = fs::write(acpi_call, "\\_SB.PCI0.GPP0.PG00._OFF\0");
            let _ = fs::write(acpi_call, "\\_SB.PCI0.GPP0.PEGP.GC6I\0");
            let _ = fs::write(acpi_call, "\\_SB.PCI0.GPP0.GSTA\0");
            let msg = "已通过 ACPI 彻底切断 NVIDIA 独显回路供电 (iGPU Only 极速省电)".to_string();
            crate::log_debug!("[显卡电源] {}", msg);
            Ok(msg)
        } else {
            let _ = fs::write(acpi_call, "\\_SB.PCI0.GPP0.PG00._ON\0");
            let msg = "已恢复 NVIDIA 独显回路供电".to_string();
            crate::log_debug!("[显卡电源] {}", msg);
            Ok(msg)
        }
    }
}
