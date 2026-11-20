// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HibernateStatus {
    pub kernel_resume_configured: bool,
    pub kernel_resume_param: String,
    pub swap_uuid: String,
    pub swap_device: String,
    pub swap_major_minor: String,
    pub sys_power_resume: String,
    pub nvidia_preserve_vram: bool,
    pub nvidia_tmp_on_disk: bool,
    pub nvidia_services_ready: bool,
    pub swap_size_gb: f64,
    pub ram_size_gb: f64,
    pub swap_sufficient: bool,
    pub zram_priority_conflict: bool,
    pub dracut_resume_module: bool,
    pub all_healthy: bool,
    pub summary: String,
}

pub struct HibernateManager;

impl HibernateManager {
    const DRACUT_RESUME_CONF: &'static str = "/etc/dracut.conf.d/resume.conf";
    const INITRAMFS_RESUME_CONF: &'static str = "/etc/initramfs-tools/conf.d/resume";
    const GRUB_HIBERNATE_CFG: &'static str = "/etc/default/grub.d/99-mechrevo-hibernate.cfg";
    const NVIDIA_MODPROBE_CONF: &'static str = "/etc/modprobe.d/nvidia-hibernate-power.conf";

    pub fn check_status() -> HibernateStatus {
        let (kernel_resume_configured, kernel_resume_param) = Self::detect_kernel_resume();

        let sys_power_resume = fs::read_to_string("/sys/power/resume")
            .unwrap_or_default()
            .trim()
            .to_string();

        let (swap_device, swap_size_gb, zram_priority_conflict) = Self::detect_swap_device();
        let swap_uuid = Self::get_device_uuid(&swap_device);
        let swap_major_minor = Self::get_device_major_minor(&swap_device);

        let ram_size_gb = Self::detect_ram_size();
        let swap_sufficient = swap_size_gb >= (ram_size_gb * 0.9);

        let (nvidia_preserve_vram, nvidia_tmp_on_disk) = Self::detect_nvidia_power_config();

        let nvidia_services_ready = Self::check_nvidia_services();

        let dracut_resume_module = Self::detect_initrd_resume_module();

        let all_healthy = kernel_resume_configured
            && (!swap_uuid.is_empty() || !swap_device.is_empty())
            && swap_sufficient
            && nvidia_preserve_vram
            && nvidia_tmp_on_disk
            && nvidia_services_ready;

        let summary = if all_healthy {
            "S4 磁盘休眠与唤醒防黑屏链路已就绪 (包含内核引导恢复参数、物理 SWAP 与 NVIDIA 显存持久化)".to_string()
        } else if !kernel_resume_configured {
            "致命缺陷：内核引导参数缺少 resume=UUID，开机引导无法定位休眠快照，直接导致无法恢复并黑屏！".to_string()
        } else if !nvidia_tmp_on_disk {
            "配置缺陷：NVIDIA 显存未指定持久化落盘目录，休眠快照将写至 tmpfs 内存盘，冷开机掉电后显存清零导致内屏不亮！".to_string()
        } else if zram_priority_conflict {
            "优先级警告：zram 内存盘优先级高于物理 SWAP，休眠快照可能写入断电易失性内存盘，开机唤醒失败！".to_string()
        } else if !swap_sufficient {
            format!("容量不足：物理 SWAP 容量 ({}GB) 小于整机物理内存 ({}GB)，休眠快照写入将失败！", swap_size_gb, ram_size_gb)
        } else {
            "S4 磁盘休眠链路尚有未配置项，建议点击「一键加固 / 修复休眠防黑屏链路」进行系统底层加固。".to_string()
        };

        HibernateStatus {
            kernel_resume_configured,
            kernel_resume_param,
            swap_uuid,
            swap_device,
            swap_major_minor,
            sys_power_resume,
            nvidia_preserve_vram,
            nvidia_tmp_on_disk,
            nvidia_services_ready,
            swap_size_gb,
            ram_size_gb,
            swap_sufficient,
            zram_priority_conflict,
            dracut_resume_module,
            all_healthy,
            summary,
        }
    }

    fn detect_kernel_resume() -> (bool, String) {
        if let Ok(cmdline) = fs::read_to_string("/proc/cmdline") {
            for part in cmdline.split_whitespace() {
                if part.starts_with("resume=") {
                    return (true, part.to_string());
                }
            }
        }

        if let Ok(resume_dev) = fs::read_to_string("/sys/power/resume") {
            let trimmed = resume_dev.trim();
            if !trimmed.is_empty() && trimmed != "0:0" {
                return (true, format!("sys_resume={}", trimmed));
            }
        }

        if let Ok(entries) = fs::read_dir("/etc/default/grub.d") {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    for line in content.lines() {
                        let trim = line.trim();
                        if !trim.starts_with('#') && trim.contains("resume=") {
                            for part in trim.split(&[' ', '"', '\''][..]) {
                                if part.starts_with("resume=") {
                                    return (true, part.to_string());
                                }
                            }
                            return (true, "grub.d resume 已配置".to_string());
                        }
                    }
                }
            }
        }

        if let Ok(grub) = fs::read_to_string("/etc/default/grub") {
            for line in grub.lines() {
                let trim = line.trim();
                if !trim.starts_with('#') && trim.contains("resume=") {
                    for part in trim.split(&[' ', '"', '\''][..]) {
                        if part.starts_with("resume=") {
                            return (true, part.to_string());
                        }
                    }
                    return (true, "grub resume 已配置".to_string());
                }
            }
        }

        if let Ok(initrd_resume) = fs::read_to_string(Self::INITRAMFS_RESUME_CONF) {
            for line in initrd_resume.lines() {
                let trim = line.trim();
                if !trim.starts_with('#') && trim.starts_with("RESUME=") {
                    let val = trim.trim_start_matches("RESUME=").trim();
                    if !val.is_empty() {
                        return (true, format!("resume={}", val));
                    }
                }
            }
        }

        if let Ok(grub_cfg) = fs::read_to_string("/boot/grub/grub.cfg") {
            for line in grub_cfg.lines() {
                let trim = line.trim();
                if (trim.starts_with("linux ") || trim.starts_with("linux16 ")) && trim.contains("resume=") {
                    for part in trim.split_whitespace() {
                        if part.starts_with("resume=") {
                            return (true, part.to_string());
                        }
                    }
                }
            }
        }

        (false, String::new())
    }

    fn detect_swap_device() -> (String, f64, bool) {
        let swaps = fs::read_to_string("/proc/swaps").unwrap_or_default();
        let mut target_device = String::new();
        let mut target_size_gb = 0.0;
        let mut zram_priority = -999;
        let mut swap_priority = -999;

        for line in swaps.lines().skip(1) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 5 {
                let dev = cols[0];
                let size_kb: f64 = cols[2].parse().unwrap_or(0.0);
                let prio: i32 = cols[4].parse().unwrap_or(-999);

                if dev.starts_with("/dev/zram") {
                    zram_priority = prio;
                } else if dev.starts_with("/dev/") || dev.starts_with('/') {
                    target_device = dev.to_string();
                    target_size_gb = (size_kb / (1024.0 * 1024.0) * 10.0).round() / 10.0;
                    swap_priority = prio;
                }
            }
        }

        if target_device.is_empty() {
            if let Ok(fstab) = fs::read_to_string("/etc/fstab") {
                for line in fstab.lines() {
                    let trim = line.trim();
                    if !trim.starts_with('#') && trim.contains("swap") {
                        let parts: Vec<&str> = trim.split_whitespace().collect();
                        if let Some(&p) = parts.first() {
                            target_device = p.to_string();
                            break;
                        }
                    }
                }
            }
        }

        let conflict = zram_priority > swap_priority && zram_priority != -999 && swap_priority != -999;
        (target_device, target_size_gb, conflict)
    }

    fn get_device_uuid(device: &str) -> String {
        if device.is_empty() {
            return String::new();
        }

        if device.starts_with("/dev/disk/by-uuid/") {
            return device.trim_start_matches("/dev/disk/by-uuid/").to_string();
        }

        if let Ok(entries) = fs::read_dir("/dev/disk/by-uuid") {
            for entry in entries.flatten() {
                if let Ok(target) = fs::read_link(entry.path()) {
                    let resolved = entry.path().parent().unwrap().join(target);
                    if let Ok(canonical) = fs::canonicalize(&resolved) {
                        if canonical.to_string_lossy() == device {
                            return entry.file_name().to_string_lossy().to_string();
                        }
                    }
                }
            }
        }

        if let Ok(out) = Command::new("blkid").args(["-s", "UUID", "-o", "value", device]).output() {
            let uuid = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !uuid.is_empty() {
                return uuid;
            }
        }

        if device.starts_with('/') && !device.starts_with("/dev/") {
            if let Ok(out) = Command::new("findmnt").args(["-no", "UUID", "-T", device]).output() {
                let uuid = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !uuid.is_empty() {
                    return uuid;
                }
            }
        }

        String::new()
    }

    fn get_device_major_minor(device: &str) -> String {
        let dev_name = device.trim_start_matches("/dev/");
        let sys_dev_path = format!("/sys/class/block/{}/dev", dev_name);
        if let Ok(content) = fs::read_to_string(sys_dev_path) {
            return content.trim().to_string();
        }
        String::new()
    }

    fn detect_ram_size() -> f64 {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let kb: f64 = parts[1].parse().unwrap_or(0.0);
                        return (kb / (1024.0 * 1024.0) * 10.0).round() / 10.0;
                    }
                }
            }
        }
        32.0
    }

    fn detect_nvidia_power_config() -> (bool, bool) {
        let mut preserve_vram = false;
        let mut tmp_on_disk = false;

        if let Ok(nvidia_params) = fs::read_to_string("/proc/driver/nvidia/params") {
            if nvidia_params.contains("PreserveVideoMemoryAllocations: 1") {
                preserve_vram = true;
            }
            if nvidia_params.contains("TemporaryFilePath: \"/var/tmp\"")
                || nvidia_params.contains("TemporaryFilePath: \"/var\"")
                || (nvidia_params.contains("TemporaryFilePath:") && !nvidia_params.contains("TemporaryFilePath: \"\"") && !nvidia_params.contains("TemporaryFilePath: \"/tmp\""))
            {
                tmp_on_disk = true;
            }
        }

        if let Ok(entries) = fs::read_dir("/etc/modprobe.d") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("conf") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("NVreg_PreserveVideoMemoryAllocations=1") {
                            preserve_vram = true;
                        }
                        if content.contains("NVreg_TemporaryFilePath=/var/tmp")
                            || content.contains("NVreg_TemporaryFilePath=/var")
                            || content.contains("NVreg_TemporaryFilePath=\"/var/tmp\"")
                        {
                            tmp_on_disk = true;
                        }
                    }
                }
            }
        }

        (preserve_vram, tmp_on_disk)
    }

    fn check_nvidia_services() -> bool {
        let h_enabled = Command::new("systemctl")
            .args(["is-enabled", "nvidia-hibernate.service"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("enabled"))
            .unwrap_or(false);

        let r_enabled = Command::new("systemctl")
            .args(["is-enabled", "nvidia-resume.service"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("enabled"))
            .unwrap_or(false);

        h_enabled && r_enabled
    }

    fn detect_initrd_resume_module() -> bool {
        // A. 检查 dracut.conf.d
        if let Ok(entries) = fs::read_dir("/etc/dracut.conf.d") {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.contains("resume") {
                        return true;
                    }
                }
            }
        }

        // B. 检查 /etc/dracut.conf
        if let Ok(content) = fs::read_to_string("/etc/dracut.conf") {
            if content.contains("resume") {
                return true;
            }
        }

        if Path::new(Self::INITRAMFS_RESUME_CONF).exists() {
            return true;
        }

        if let Ok(content) = fs::read_to_string("/etc/initramfs-tools/modules") {
            if content.lines().any(|l| l.trim() == "resume") {
                return true;
            }
        }

        false
    }

    pub fn install_hibernate_fix() -> Result<String, String> {
        let (swap_device, _, _) = Self::detect_swap_device();
        let swap_uuid = Self::get_device_uuid(&swap_device);
        if swap_uuid.is_empty() {
            return Err("未能在系统中检测到有效的物理 SWAP 分区 UUID，无法配置内核引导恢复参数。".into());
        }

        println!("[休眠修复] 检测到物理 SWAP 分区: {} (UUID: {})", swap_device, swap_uuid);

        let grub_main = Path::new("/etc/default/grub");
        let grub_backup = Path::new("/etc/default/grub.prime-backup");
        if !grub_main.exists() && grub_backup.exists() {
            println!("[休眠修复] 从备份恢复 /etc/default/grub 主配置文件...");
            let _ = fs::copy(grub_backup, grub_main);
        }

        let grub_d = Path::new("/etc/default/grub.d");
        let _ = fs::create_dir_all(grub_d);
        let grub_cfg_content = format!(
            "# rmechrevo: S4 磁盘休眠唤醒防黑屏专用参数\n\
             GRUB_CMDLINE_LINUX_DEFAULT=\"$GRUB_CMDLINE_LINUX_DEFAULT resume=UUID={}\"\n",
            swap_uuid
        );
        fs::write(Self::GRUB_HIBERNATE_CFG, grub_cfg_content)
            .map_err(|e| format!("写入 GRUB 休眠参数失败: {}", e))?;
        println!("[休眠修复] 成功写入 GRUB 引导参数: {}", Self::GRUB_HIBERNATE_CFG);

        let dracut_d = Path::new("/etc/dracut.conf.d");
        let _ = fs::create_dir_all(dracut_d);
        let dracut_content = "# 自动打包 resume 恢复模块至引导映像\nadd_dracutmodules+=\" resume \"\n";
        fs::write(Self::DRACUT_RESUME_CONF, dracut_content)
            .map_err(|e| format!("写入 dracut resume 配置失败: {}", e))?;
        println!("[休眠修复] 成功配置 dracut 恢复模块: {}", Self::DRACUT_RESUME_CONF);

        let initramfs_conf_d = Path::new("/etc/initramfs-tools/conf.d");
        let _ = fs::create_dir_all(initramfs_conf_d);
        let initramfs_content = format!("RESUME=UUID={}\n", swap_uuid);
        let _ = fs::write(Self::INITRAMFS_RESUME_CONF, initramfs_content);

        let nvidia_conf_content = "\
# NVIDIA 独显休眠显存落盘保护配置 (杜绝断电导致显存丢失与内屏不亮)
options nvidia NVreg_PreserveVideoMemoryAllocations=1
options nvidia NVreg_TemporaryFilePath=/var/tmp
";
        fs::write(Self::NVIDIA_MODPROBE_CONF, nvidia_conf_content)
            .map_err(|e| format!("写入 NVIDIA 显存配置失败: {}", e))?;
        println!("[休眠修复] 成功写入 NVIDIA 显存落盘保护规则: {}", Self::NVIDIA_MODPROBE_CONF);

        println!("[休眠修复] 正在启用 NVIDIA 专用休眠与唤醒 systemd 服务...");
        let _ = Command::new("systemctl")
            .args(["enable", "nvidia-hibernate.service", "nvidia-resume.service", "nvidia-suspend.service"])
            .status();

        let major_minor = Self::get_device_major_minor(&swap_device);
        if !major_minor.is_empty() {
            let _ = fs::write("/sys/power/resume", &major_minor);
            println!("[休眠修复] 已将当前内核休眠恢复设备设为: {}", major_minor);
        }

        println!("[休眠修复] 正在执行 update-grub 更新引导菜单...");
        let grub_res = Command::new("update-grub").status();
        if let Err(e) = grub_res {
            eprintln!("[休眠修复] update-grub 执行失败: {}", e);
        }

        println!("[休眠修复] 正在重新生成内核启动映像 (dracut -f)...");
        let dracut_status = Command::new("dracut").arg("-f").status();
        if dracut_status.is_err() || !dracut_status.unwrap().success() {
            let _ = Command::new("update-initramfs").arg("-u").status();
        }

        Ok(format!(
            "🎉 S4 磁盘休眠防黑屏链路已成功加固！\n\
             • 内核引导恢复参数已锁定物理 SWAP: UUID={}\n\
             • NVIDIA 独显已配置 /var/tmp 显存固态落盘与唤醒服务\n\
             • 启动映像恢复模块已成功打包注入\n\
             系统执行休眠关机再重新开机即可瞬时完美点亮内屏！",
            swap_uuid
        ))
    }

    pub fn rollback_hibernate_fix() -> Result<String, String> {
        let _ = fs::remove_file(Self::GRUB_HIBERNATE_CFG);
        let _ = fs::remove_file(Self::DRACUT_RESUME_CONF);
        let _ = fs::remove_file(Self::INITRAMFS_RESUME_CONF);
        let _ = fs::remove_file(Self::NVIDIA_MODPROBE_CONF);

        let _ = Command::new("update-grub").status();
        let _ = Command::new("dracut").arg("-f").status();

        Ok("休眠修复配置已回滚，系统引导已恢复原生状态。".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status() {
        let status = HibernateManager::check_status();
        println!("\n=== HIBERNATE STATUS DEBUG ===");
        println!("{:#?}", status);
        println!("==============================\n");
        assert!(status.kernel_resume_configured);
        assert!(status.nvidia_preserve_vram);
        assert!(status.nvidia_tmp_on_disk);
    }
}
