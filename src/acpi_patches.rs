// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs;
use std::path::Path;
use std::process::Command;

pub struct AcpiPatchManager;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcpiPatchStatus {
    pub dsdt_applied: bool,
    pub ssdt_sound_applied: bool,
    pub ssdt_sound_needed: bool,
    pub ssdt_d3cold_applied: bool,
    pub ssdt_d3cold_needed: bool,
    pub active_in_kernel: bool,
    pub message: String,
}

impl AcpiPatchManager {
    const DSDT_SRC: &'static str = "/sys/firmware/acpi/tables/DSDT";
    const ACPI_OVERRIDE_DIR: &'static str = "/etc/initramfs-tools/acpi_override";
    const HOOK_SCRIPT_PATH: &'static str = "/etc/initramfs-tools/hooks/acpi_override";
    const DRACUT_ACPI_CONF: &'static str = "/etc/dracut.conf.d/99-acpi-override.conf";

    /// 检测系统当前运行中 DSDT 是否已具备 PEHP=Zero
    fn is_dsdt_pehp_zero() -> bool {
        if let Ok(raw) = fs::read(Self::DSDT_SRC) {
            let pat_zero: [u8; 6] = [0x08, 0x50, 0x45, 0x48, 0x50, 0x00];
            return raw.windows(pat_zero.len()).any(|w| w == pat_zero);
        }
        false
    }

    pub fn check_status() -> AcpiPatchStatus {
        let dsdt_file = Path::new(Self::ACPI_OVERRIDE_DIR).join("DSDT.aml");
        let ssdt_sound_file = Path::new(Self::ACPI_OVERRIDE_DIR).join("SSDT_CPMACPV4.aml");
        let ssdt_d3cold_file = Path::new(Self::ACPI_OVERRIDE_DIR).join("SSDT_UPEPRPL.aml");
        let hook_file = Path::new(Self::HOOK_SCRIPT_PATH);
        let dracut_file = Path::new(Self::DRACUT_ACPI_CONF);

        let pehp_zero_active = Self::is_dsdt_pehp_zero();
        let dsdt_file_exists = dsdt_file.exists();
        let hook_installed = hook_file.exists() || dracut_file.exists();

        let dsdt_applied = dsdt_file_exists || pehp_zero_active;
        let ssdt_sound_applied = ssdt_sound_file.exists();
        let ssdt_d3cold_applied = ssdt_d3cold_file.exists();

        let ssdt_sound_needed = Self::find_cpmacpv4_ssdt().is_ok();
        let ssdt_d3cold_needed = Self::find_upeprpl_ssdt().is_ok();

        let mut dmesg_found = false;
        if let Ok(out) = Command::new("dmesg").output() {
            let log = String::from_utf8_lossy(&out.stdout);
            if log.contains("ACPI: Table Upgrade: override [DSDT")
                || log.contains("override [SSDT-   AMD-CPMACPV4]")
                || log.contains("override [SSDT-   AMD-UPEPRPL]")
            {
                dmesg_found = true;
            }
        }

        // 判定核心补丁是否已生效：内核正在运行 PEHP=0 或 启动映像挂钩就绪 或 dmesg 显示 table upgrade
        let active_in_kernel = pehp_zero_active || (dsdt_file_exists && hook_installed) || dmesg_found;

        let message = if active_in_kernel {
            "独显直连睡眠防黑屏补丁 (PEHP=0) 已生效，睡眠唤醒保护已就绪".to_string()
        } else if dsdt_file_exists && !hook_installed {
            "检测到 DSDT 补丁文件但缺少引导挂钩脚本，请点击「部署修复补丁集」".to_string()
        } else {
            "未检测到硬件睡眠补丁，建议部署以防止独显直连模式下睡眠唤醒黑屏".to_string()
        };

        AcpiPatchStatus {
            dsdt_applied,
            ssdt_sound_applied,
            ssdt_sound_needed,
            ssdt_d3cold_applied,
            ssdt_d3cold_needed,
            active_in_kernel,
            message,
        }
    }

    /// 生成 DSDT 补丁 AML 数据 (修复独显直连睡眠无法唤醒)
    /// 原理：将 Name(PEHP, One) 改为 Name(PEHP, Zero)，避免 pciehp 驱动在唤醒时误摘掉独显
    pub fn generate_dsdt_patched_aml() -> Result<Vec<u8>, String> {
        let raw = fs::read(Self::DSDT_SRC)
            .map_err(|e| format!("读取原生 DSDT 失败: {}。请确认以 root 运行。", e))?;

        if raw.len() < 36 {
            return Err("DSDT 表长度异常".into());
        }

        // 特征码: Name (PEHP, One) -> 0x08, 'P', 'E', 'H', 'P', 0x01
        let pat: [u8; 6] = [0x08, 0x50, 0x45, 0x48, 0x50, 0x01];
        let mut matches = Vec::new();

        for i in 0..=(raw.len().saturating_sub(pat.len())) {
            if &raw[i..i + pat.len()] == &pat {
                matches.push(i);
            }
        }

        if matches.is_empty() {
            // 检查是否已经为 Zero (0x08, 'P', 'E', 'H', 'P', 0x00)
            let pat_zero: [u8; 6] = [0x08, 0x50, 0x45, 0x48, 0x50, 0x00];
            if raw.windows(pat_zero.len()).any(|w| w == pat_zero) {
                return Ok(raw);
            }
            return Err("未在 DSDT 中找到 Name(PEHP, One) 特征码，可能已打过补丁或固件版本不同".into());
        }
        if matches.len() > 1 {
            return Err(format!("在 DSDT 中找到 {} 处 PEHP 特征码，为确保安全已停止修改", matches.len()));
        }

        let mut patched = raw.clone();
        let target_idx = matches[0];

        patched[target_idx + 5] = 0x00;

        let rev = u32::from_le_bytes(patched[24..28].try_into().unwrap());
        let new_rev = rev.wrapping_add(1);
        patched[24..28].copy_from_slice(&new_rev.to_le_bytes());

        patched[9] = 0;
        let sum: u32 = patched.iter().map(|&b| b as u32).sum();
        patched[9] = (-(sum as i32) as u8) & 0xFF;

        println!(
            "[ACPI补丁] 成功生成 DSDT 补丁 (PEHP: 0x01 -> 0x00, 版本: 0x{:08X} -> 0x{:08X})",
            rev, new_rev
        );
        Ok(patched)
    }

    /// 查找系统中 OEM Table ID 为 CPMACPV4 的原生 SSDT 表
    pub fn find_cpmacpv4_ssdt() -> Result<(String, Vec<u8>), String> {
        let tables_dir = Path::new("/sys/firmware/acpi/tables");
        if !tables_dir.exists() {
            return Err("ACPI 表目录不存在".into());
        }

        for entry in fs::read_dir(tables_dir).map_err(|e| e.to_string())?.flatten() {
            let path = entry.path();
            if let Ok(data) = fs::read(&path) {
                if data.len() >= 36 {
                    let sig = &data[0..4];
                    let oem_table_id = &data[16..24];
                    if sig == b"SSDT" && oem_table_id.starts_with(b"CPMACPV4") {
                        return Ok((entry.file_name().to_string_lossy().to_string(), data));
                    }
                }
            }
        }

        Err("当前硬件未包含 CPMACPV4 表 (AMD 平台原生支持音频，无需此表)".into())
    }

    /// 查找系统中 OEM Table ID 为 UPEPRPL 的原生 SSDT 表 (独显 D3cold 修复目标)
    pub fn find_upeprpl_ssdt() -> Result<(String, Vec<u8>), String> {
        let tables_dir = Path::new("/sys/firmware/acpi/tables");
        if !tables_dir.exists() {
            return Err("ACPI 表目录不存在".into());
        }

        for entry in fs::read_dir(tables_dir).map_err(|e| e.to_string())?.flatten() {
            let path = entry.path();
            if let Ok(data) = fs::read(&path) {
                if data.len() >= 36 {
                    let sig = &data[0..4];
                    let oem_table_id = &data[16..24];
                    if sig == b"SSDT" && oem_table_id.starts_with(b"UPEPRPL") {
                        return Ok((entry.file_name().to_string_lossy().to_string(), data));
                    }
                }
            }
        }

        Err("当前硬件未包含 UPEPRPL 表 (当前固件内建电源门控，无需此表)".into())
    }

    pub fn install_all_patches() -> Result<String, String> {
        fs::create_dir_all(Self::ACPI_OVERRIDE_DIR)
            .map_err(|e| format!("创建 {} 目录失败 (请以 root 身份运行守护进程): {}", Self::ACPI_OVERRIDE_DIR, e))?;

        let dsdt_bytes = Self::generate_dsdt_patched_aml()?;
        let dsdt_path = Path::new(Self::ACPI_OVERRIDE_DIR).join("DSDT.aml");
        fs::write(&dsdt_path, dsdt_bytes).map_err(|e| format!("写入 DSDT.aml 失败: {}", e))?;
        println!("[ACPI补丁] 成功写入核心独显直连唤醒补丁: {:?}", dsdt_path);

        let hook_content = r#"#!/bin/sh
set -e

PREREQ=""
prereqs() { echo "$PREREQ"; }
case "$1" in
    prereqs) prereqs; exit 0 ;;
esac

. /usr/share/initramfs-tools/hook-functions

ACPI_DIR="/etc/initramfs-tools/acpi_override"
if [ -d "$ACPI_DIR" ] && ls "$ACPI_DIR"/*.aml >/dev/null 2>&1; then
    TMP_DIR=$(mktemp -d)
    mkdir -p "$TMP_DIR/kernel/firmware/acpi"
    cp -L "$ACPI_DIR"/*.aml "$TMP_DIR/kernel/firmware/acpi/"

    TMP_CPIO=$(mktemp)
    (cd "$TMP_DIR" && find kernel -type f -print0 | cpio --null -H newc -o) > "$TMP_CPIO" 2>/dev/null

    prepend_earlyinitramfs "$TMP_CPIO"

    rm -rf "$TMP_DIR" "$TMP_CPIO"
fi
"#;

        if let Some(parent) = Path::new(Self::HOOK_SCRIPT_PATH).parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(Self::HOOK_SCRIPT_PATH, hook_content)
            .map_err(|e| format!("写入 initramfs hook 失败: {}", e))?;

        let _ = Command::new("chmod").args(["+x", Self::HOOK_SCRIPT_PATH]).status();

        if let Some(parent) = Path::new(Self::DRACUT_ACPI_CONF).parent() {
            let _ = fs::create_dir_all(parent);
            let dracut_conf = format!(
                "# rmechrevo ACPI override configuration\nacpi_override=\"yes\"\nacpi_table_dir=\"{}\"\n",
                Self::ACPI_OVERRIDE_DIR
            );
            let _ = fs::write(Self::DRACUT_ACPI_CONF, dracut_conf);
        }

        println!("[ACPI补丁] 正在重新生成内核 initramfs 引导映像 (update-initramfs -u)...");
        let status = Command::new("update-initramfs")
            .arg("-u")
            .status();

        match status {
            Ok(s) if s.success() => {
                let msg = "🎉 硬件睡眠唤醒补丁已成功部署到系统启动映像！PEHP=0 直连睡眠防护已就绪。".to_string();
                println!("{}", msg);
                Ok(msg)
            }
            Ok(s) => Err(format!("update-initramfs 退出码异常: {:?}", s.code())),
            Err(e) => {
                // 回退尝试 dracut
                if Command::new("dracut").arg("-f").status().map(|s| s.success()).unwrap_or(false) {
                    Ok("🎉 补丁已通过 dracut 成功更新至内核映像！".to_string())
                } else {
                    Err(format!("执行 update-initramfs 失败: {}", e))
                }
            }
        }
    }

    pub fn rollback_patches() -> Result<String, String> {
        let _ = fs::remove_file(Self::HOOK_SCRIPT_PATH);
        let _ = fs::remove_file(Self::DRACUT_ACPI_CONF);
        if Path::new(Self::ACPI_OVERRIDE_DIR).exists() {
            let _ = fs::remove_dir_all(Self::ACPI_OVERRIDE_DIR);
        }

        println!("[ACPI补丁] 正在回滚 initramfs 引导映像...");
        let status = Command::new("update-initramfs").arg("-u").status();
        match status {
            Ok(s) if s.success() => {
                Ok("已成功卸载 ACPI 补丁并重构系统启动映像，系统已恢复为原生 ACPI 表。".into())
            }
            _ => Err("执行 update-initramfs -u 失败，请确认系统特权权限".into()),
        }
    }
}
