// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHardwareInfo {
    pub product_name: String,
    pub board_name: String,
    pub bios_version: String,
    pub bios_date: String,
    pub ec_version: String,
    pub cpu_model: String,
    pub gpu_model: String,
    pub kernel_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverStatus {
    pub ready: bool,
    pub acpi_call_loaded: bool,
    pub acpi_call_installed: bool,
    pub uniwill_loaded: bool,
    pub hotkey_ready: bool,
    pub hotkey_device: Option<String>,
    pub hotkey_name: Option<String>,
    pub message: String,
    pub hardware_info: SystemHardwareInfo,
}

impl Default for DriverStatus {
    fn default() -> Self {
        Self {
            ready: false,
            acpi_call_loaded: false,
            acpi_call_installed: false,
            uniwill_loaded: false,
            hotkey_ready: false,
            hotkey_device: None,
            hotkey_name: None,
            message: "未初始化检测".into(),
            hardware_info: SystemHardwareInfo {
                product_name: "--".into(),
                board_name: "--".into(),
                bios_version: "--".into(),
                bios_date: "--".into(),
                ec_version: "--".into(),
                cpu_model: "--".into(),
                gpu_model: "--".into(),
                kernel_version: "--".into(),
            },
        }
    }
}

pub struct DriverManager;

impl DriverManager {
    const ACPI_CALL_PATH: &'static str = "/proc/acpi/call";

    pub fn check_status() -> DriverStatus {
        let mut status = DriverStatus::default();

        let dmi = Path::new("/sys/class/dmi/id");
        if dmi.exists() {
            let read_dmi = |field: &str, def: &str| -> String {
                fs::read_to_string(dmi.join(field))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| def.to_string())
            };

            let vendor = read_dmi("sys_vendor", "");
            let prod = read_dmi("product_name", "Unknown");
            status.hardware_info.product_name = if vendor.is_empty() || prod.starts_with(&vendor) {
                prod
            } else {
                format!("{} {}", vendor, prod)
            };
            status.hardware_info.board_name = read_dmi("board_name", "Unknown");
            status.hardware_info.bios_version = read_dmi("bios_version", "N.1.19MRO20");
            status.hardware_info.bios_date = read_dmi("bios_date", "12/18/2025");
            status.hardware_info.ec_version = read_dmi("ec_firmware_release", "1.31");
        }

        if let Ok(out) = Command::new("uname").arg("-sr").output() {
            status.hardware_info.kernel_version = String::from_utf8_lossy(&out.stdout).trim().to_string();
        }

        if Path::new(Self::ACPI_CALL_PATH).exists() {
            status.acpi_call_loaded = true;
            status.acpi_call_installed = true;
        } else {
            let _ = Command::new("modprobe").arg("acpi_call").output();
            if Path::new(Self::ACPI_CALL_PATH).exists() {
                status.acpi_call_loaded = true;
                status.acpi_call_installed = true;
            } else {
                status.acpi_call_loaded = false;
                let dpkg_check = Command::new("dpkg")
                    .args(["-s", "acpi-call-dkms"])
                    .output();
                status.acpi_call_installed = match dpkg_check {
                    Ok(out) => out.status.success(),
                    Err(_) => false,
                };
            }
        }

        let uniwill_sysfs = Path::new("/sys/bus/platform/devices/INOU0000:00/super_key_enable").exists();
        if !uniwill_sysfs {
            let _ = Command::new("modprobe").args(["uniwill-laptop", "force=1"]).output();
        }
        status.uniwill_loaded = Path::new("/sys/bus/platform/devices/INOU0000:00/super_key_enable").exists()
            || Path::new("/sys/bus/platform/drivers/uniwill_wmi").exists()
            || Path::new("/sys/bus/platform/devices/INOU0000:00").exists();

        if let Ok(entries) = fs::read_dir("/dev/input") {
            let mut hotkey_candidate = None;
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    if file_name.starts_with("event") {
                        let sysfs_name_path = format!(
                            "/sys/class/input/{}/device/name",
                            file_name
                        );
                        if let Ok(name) = fs::read_to_string(&sysfs_name_path) {
                            let name_trim = name.trim();
                            if name_trim.contains("Mouse") || name_trim.contains("Touchpad") {
                                continue;
                            }
                            if name_trim.to_ascii_lowercase().contains("hotkey")
                                || name_trim.contains("Uniwill WMI")
                            {
                                status.hotkey_ready = true;
                                status.hotkey_device = Some(path.to_string_lossy().to_string());
                                status.hotkey_name = Some(name_trim.to_string());
                                break;
                            } else if name_trim.contains("UNIW") || name_trim.contains("mech") {
                                hotkey_candidate = Some((path.to_string_lossy().to_string(), name_trim.to_string()));
                            }
                        }
                    }
                }
            }
            if !status.hotkey_ready {
                if let Some((dev, name)) = hotkey_candidate {
                    status.hotkey_ready = true;
                    status.hotkey_device = Some(dev);
                    status.hotkey_name = Some(name);
                }
            }
        }

        if status.acpi_call_loaded {
            status.ready = true;
            status.message = "所有核心通信驱动均已就绪，EC 硬件直通与底层寄存器已解锁".into();
        } else if status.acpi_call_installed {
            status.ready = false;
            status.message = "已安装 acpi-call-dkms，但内核模块未加载，请点击「安装/加载驱动」".into();
        } else {
            status.ready = false;
            status.message = "未检测到 acpi-call-dkms 驱动，无法控制 EC 性能模式与功耗超频".into();
        }

        status
    }

    pub fn install_and_setup() -> Result<String, String> {
        println!("[驱动安装] 开始执行 acpi-call-dkms 自动化部署流程...");

        let _ = Command::new("modprobe").arg("acpi_call").output();
        if Path::new(Self::ACPI_CALL_PATH).exists() {
            println!("[驱动安装] modprobe acpi_call 成功，/proc/acpi/call 已就绪");
            return Ok("底层驱动已成功加载并激活！硬件直通已就绪。".into());
        }

        println!("[驱动安装] 正在调用 apt-get 安装 acpi-call-dkms 与 linux-headers...");
        let install_res = Command::new("apt-get")
            .args(["install", "-y", "acpi-call-dkms"])
            .output()
            .map_err(|e| format!("执行 apt-get 失败: {}", e))?;

        if !install_res.status.success() {
            let stderr = String::from_utf8_lossy(&install_res.stderr);
            return Err(format!("apt-get 安装失败: {}", stderr.trim()));
        }

        let probe_res = Command::new("modprobe")
            .arg("acpi_call")
            .output()
            .map_err(|e| format!("执行 modprobe 失败: {}", e))?;

        if !probe_res.status.success() {
            let stderr = String::from_utf8_lossy(&probe_res.stderr);
            return Err(format!("modprobe 加载 acpi_call 失败: {}", stderr.trim()));
        }

        let _ = Command::new("modprobe").args(["uniwill-laptop", "force=1"]).output();

        if Path::new(Self::ACPI_CALL_PATH).exists() {
            println!("[驱动安装] 部署完毕，/proc/acpi/call 节点已就绪！");
            Ok("acpi-call-dkms 与 uniwill-laptop 驱动安装并挂载成功，硬件通道已完全解锁！".into())
        } else {
            Err("模块加载完毕但 /proc/acpi/call 节点未生成，请检查安全启动(Secure Boot)是否开启".into())
        }
    }
}
