// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct SensorReadings {
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub fan_cpu_rpm: Option<u32>,
    pub fan_gpu_rpm: Option<u32>,
    pub fan_source: FanSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FanSource {
    UniwillHwmon(PathBuf),
    EstimatedByTemp,
}

impl Default for FanSource {
    fn default() -> Self {
        FanSource::EstimatedByTemp
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDriverStatus {
    pub is_uniwill_loaded: bool,
    pub has_hwmon_fans: bool,
    pub has_acpi_call: bool,
    pub bios_version: String,
    pub product_name: String,
    pub cpu_name: String,
    pub gpu_name: String,
    pub gpu_mode: String,
}

pub struct HardwareSensors;

impl HardwareSensors {
    pub fn read_all() -> SensorReadings {
        let cpu_temp = Self::read_cpu_temp().unwrap_or(50.0);
        let gpu_temp = Self::read_gpu_temp().unwrap_or(45.0);

        let (fan_cpu_rpm, fan_gpu_rpm, fan_source) = match Self::read_hwmon_fans() {
            Some((c_rpm, g_rpm, path)) => (Some(c_rpm), Some(g_rpm), FanSource::UniwillHwmon(path)),
            None => {
                let max_temp = cpu_temp.max(gpu_temp);
                let est_rpm = if max_temp < 50.0 {
                    1800
                } else if max_temp < 65.0 {
                    2400 + ((max_temp - 50.0) * 60.0) as u32
                } else if max_temp < 80.0 {
                    3300 + ((max_temp - 65.0) * 80.0) as u32
                } else {
                    4500 + ((max_temp - 80.0) * 50.0).min(900.0) as u32
                };
                (Some(est_rpm), Some(est_rpm), FanSource::EstimatedByTemp)
            }
        };

        SensorReadings {
            cpu_temp,
            gpu_temp,
            fan_cpu_rpm,
            fan_gpu_rpm,
            fan_source,
        }
    }

    pub fn get_system_status() -> SystemDriverStatus {
        let is_uniwill_loaded = Path::new("/sys/bus/platform/drivers/uniwill").exists();
        let has_hwmon_fans = Self::read_hwmon_fans().is_some();
        let has_acpi_call = Path::new("/proc/acpi/call").exists();

        let bios_version = fs::read_to_string("/sys/class/dmi/id/bios_version")
            .unwrap_or_else(|_| "未知".to_string())
            .trim()
            .to_string();

        let product_name = fs::read_to_string("/sys/class/dmi/id/product_name")
            .unwrap_or_else(|_| "Uniwill Laptop".to_string())
            .trim()
            .to_string();

        let cpu_name = fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|info| {
                info.lines()
                    .find(|l| l.starts_with("model name"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| "AMD Ryzen 9 8940HX".to_string());

        let gpu_name = Self::read_gpu_name().unwrap_or_else(|| "NVIDIA GeForce RTX 5070 Mobile".to_string());
        let gpu_mode = crate::gpu_mode::GpuModeManager::detect_current_mode().name().to_string();

        SystemDriverStatus {
            is_uniwill_loaded,
            has_hwmon_fans,
            has_acpi_call,
            bios_version,
            product_name,
            cpu_name,
            gpu_name,
            gpu_mode,
        }
    }

    fn read_cpu_temp() -> Option<f32> {
        let hwmon_dir = Path::new("/sys/class/hwmon");
        if let Ok(entries) = fs::read_dir(hwmon_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name_file = path.join("name");
                if let Ok(name) = fs::read_to_string(&name_file) {
                    let name = name.trim();
                    if name == "k10temp" {
                        if let Ok(val_str) = fs::read_to_string(path.join("temp1_input")) {
                            if let Ok(milli_c) = val_str.trim().parse::<f32>() {
                                return Some(milli_c / 1000.0);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(entries) = fs::read_dir(hwmon_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name_file = path.join("name");
                if let Ok(name) = fs::read_to_string(&name_file) {
                    if name.trim() == "acpitz" {
                        if let Ok(val_str) = fs::read_to_string(path.join("temp1_input")) {
                            if let Ok(milli_c) = val_str.trim().parse::<f32>() {
                                return Some(milli_c / 1000.0);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn read_gpu_temp() -> Option<f32> {
        let output = Command::new("nvidia-smi")
            .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
            .output()
            .ok()?;

        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Ok(temp) = s.trim().lines().next().unwrap_or("").parse::<f32>() {
                return Some(temp);
            }
        }

        let hwmon_dir = Path::new("/sys/class/hwmon");
        if let Ok(entries) = fs::read_dir(hwmon_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(name) = fs::read_to_string(path.join("name")) {
                    if name.trim() == "amdgpu" {
                        if let Ok(val_str) = fs::read_to_string(path.join("temp1_input")) {
                            if let Ok(milli_c) = val_str.trim().parse::<f32>() {
                                return Some(milli_c / 1000.0);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn read_gpu_name() -> Option<String> {
        let output = Command::new("nvidia-smi")
            .args(["--query-gpu=name", "--format=csv,noheader"])
            .output()
            .ok()?;

        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            let name = s.trim().lines().next().unwrap_or("").to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }

        None
    }

    fn read_hwmon_fans() -> Option<(u32, u32, PathBuf)> {
        let hwmon_dir = Path::new("/sys/class/hwmon");
        if let Ok(entries) = fs::read_dir(hwmon_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let fan1_file = path.join("fan1_input");
                let fan2_file = path.join("fan2_input");

                if fan1_file.exists() {
                    let fan1 = fs::read_to_string(&fan1_file)
                        .ok()
                        .and_then(|s| s.trim().parse::<u32>().ok())
                        .unwrap_or(0);

                    let fan2 = if fan2_file.exists() {
                        fs::read_to_string(&fan2_file)
                            .ok()
                            .and_then(|s| s.trim().parse::<u32>().ok())
                            .unwrap_or(fan1)
                    } else {
                        fan1
                    };

                    return Some((fan1, fan2, path));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensors() {
        let r = HardwareSensors::read_all();
        assert!(r.cpu_temp.is_finite());
        assert!(r.gpu_temp.is_finite());
    }
}
