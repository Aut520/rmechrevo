// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use serde::{Deserialize, Serialize};
use std::fs;
use crate::ec::{EcInterface, registers::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTuningConfig {
    pub cpu_pl1: u8,
    pub cpu_pl2: u8,
    pub cpu_pl4: u8,
    pub gpu_core_offset: i16,
    pub gpu_mem_offset: i16,
    pub gpu_dynamic_boost: u8,
    pub fan_left_duty: u8,
    pub fan_right_duty: u8,
}

impl Default for CustomTuningConfig {
    fn default() -> Self {
        Self {
            cpu_pl1: 80,
            cpu_pl2: 115,
            cpu_pl4: 180,
            gpu_core_offset: 0,
            gpu_mem_offset: 0,
            gpu_dynamic_boost: 25,
            fan_left_duty: 80,
            fan_right_duty: 80,
        }
    }
}

pub struct TuningManager {
    ec: EcInterface,
}

impl TuningManager {
    const SYSFS_CTGP_OFFSET: &'static str = "/sys/bus/platform/devices/INOU0000:00/ctgp_offset";

    pub fn new() -> Self {
        Self {
            ec: EcInterface::new(),
        }
    }

    /// 应用自定义超频与风扇曲线调优 (原厂 0x0727 = 0x40 + 0x077E 功耗协议)
    pub fn apply_tuning(&self, config: &CustomTuningConfig) -> Result<String, String> {
        if !self.ec.is_available() {
            return Err("未挂载 /proc/acpi/call，无法向 EC 下发超频参数".into());
        }

        let _ = self.ec.write(ADDR_CUSTOM_ENABLE, FLAG_CUSTOM_ENABLE);

        let _ = self.ec.write(ADDR_PW_MAGIC1, PW_MAGIC1_VAL);
        let _ = self.ec.write(ADDR_PW_MAGIC2, PW_MAGIC2_VAL);
        let _ = self.ec.write(ADDR_PW_BYTE3, 0x00);
        let _ = self.ec.write(ADDR_PW_BYTE4, 0x00);
        let _ = self.ec.write(ADDR_PW_BYTE5, PW_FIXED_VAL);
        self.ec.write(ADDR_PW_SPL, config.cpu_pl1.clamp(35, 140))?;
        self.ec.write(ADDR_PW_SPPT, config.cpu_pl2.clamp(35, 140))?;
        self.ec.write(ADDR_PW_FPPT, config.cpu_pl4.clamp(35, 215))?;

        let _ = self.ec.write(ADDR_PL1_LEGACY, config.cpu_pl1.clamp(35, 120));
        let _ = self.ec.write(ADDR_PL2_LEGACY, config.cpu_pl2.clamp(35, 140));
        let _ = self.ec.write(ADDR_PL4_LEGACY, config.cpu_pl4.clamp(35, 215));

        let l_val = ((config.fan_left_duty.clamp(20, 100) as u32 * 200) / 100) as u8;
        let r_val = ((config.fan_right_duty.clamp(20, 100) as u32 * 200) / 100) as u8;
        self.ec.write(ADDR_FAN_L_DUTY, l_val)?;
        self.ec.write(ADDR_FAN_R_DUTY, r_val)?;

        let _ = self.apply_ramfan_curves(config.fan_left_duty, config.fan_right_duty);

        if std::path::Path::new(Self::SYSFS_CTGP_OFFSET).exists() {
            let _ = fs::write(Self::SYSFS_CTGP_OFFSET, format!("{}\n", config.gpu_dynamic_boost.clamp(0, 25)));
        }

        let gpu_res = Self::apply_gpu_offsets(config.gpu_core_offset, config.gpu_mem_offset);

        let msg = format!(
            "自定义调优参数已生效: CPU SPL={}W/sPPT={}W/fPPT={}W, 左风扇=0x{:02X}, 右风扇=0x{:02X}, Boost=+{}W, GPU核心偏移={:+?}MHz, 显存偏移={:+?}MHz{}",
            config.cpu_pl1, config.cpu_pl2, config.cpu_pl4, l_val, r_val, config.gpu_dynamic_boost,
            config.gpu_core_offset, config.gpu_mem_offset,
            if gpu_res.is_empty() { String::new() } else { format!(" ({})", gpu_res) }
        );
        crate::log_debug!("[自定义调优] {}", msg);
        Ok(msg)
    }

    /// 下发原厂 RamFan2 10 阶梯温控风扇曲线 (0x0F20 / 0x0F70) 并激活用户曲线接管 (0x0751 置 0x80)
    pub fn apply_ramfan_curves(&self, left_duty_pct: u8, right_duty_pct: u8) -> Result<(), String> {
        let l_max = ((left_duty_pct.clamp(30, 100) as u32 * 200) / 100) as u8;
        let r_max = ((right_duty_pct.clamp(30, 100) as u32 * 200) / 100) as u8;

        let factors: [f32; 10] = [0.30, 0.35, 0.40, 0.50, 0.55, 0.60, 0.70, 0.80, 0.90, 1.00];

        for (i, &f) in factors.iter().enumerate() {
            let offset = i as u16;
            let l_step = ((l_max as f32 * f).round() as u8).clamp(0x1E, 0xC8);
            let r_step = ((r_max as f32 * f).round() as u8).clamp(0x1E, 0xC8);
            let _ = self.ec.write(ADDR_TABLE_F1_DUTY0 + offset, l_step);
            let _ = self.ec.write(ADDR_TABLE_F2_DUTY0 + offset, r_step);
        }

        // 向 0x0751 写入 0x80 (User_Fan_Mode)，使 EC 切换到自定义温控阶梯表
        let _ = self.ec.write(ADDR_FAN_CTL, MODE_CUSTOM_BASE);
        Ok(())
    }

    pub fn apply_gpu_offsets(core_offset: i16, mem_offset: i16) -> String {
        if core_offset == 0 && mem_offset == 0 {
            return String::new();
        }

        let displays = ["", ":0", ":1", ":100"];
        let mut executed = false;

        for d in displays {
            let mut cmd = std::process::Command::new("nvidia-settings");
            if !d.is_empty() {
                cmd.env("DISPLAY", d);
            }
            cmd.args([
                "-a",
                &format!("[gpu:0]/GPUGraphicsClockOffset[3]={}", core_offset),
                "-a",
                &format!("[gpu:0]/GPUMemoryTransferRateOffset[3]={}", mem_offset),
            ]);

            if let Ok(out) = cmd.output() {
                if out.status.success() {
                    executed = true;
                    break;
                }
            }
        }

        if executed {
            "GPU 频率偏移已通过驱动下发".to_string()
        } else {
            "GPU 频率偏移已缓存 (需运行在 X11/Coolbits 权限下)".to_string()
        }
    }

    pub fn reset_gpu_offsets() {
        let displays = ["", ":0", ":1", ":100"];
        for d in displays {
            let mut cmd = std::process::Command::new("nvidia-settings");
            if !d.is_empty() {
                cmd.env("DISPLAY", d);
            }
            cmd.args([
                "-a", "[gpu:0]/GPUGraphicsClockOffset[3]=0",
                "-a", "[gpu:0]/GPUMemoryTransferRateOffset[3]=0",
                "-a", "[gpu:0]/GPUGraphicsClockOffset[4]=0",
                "-a", "[gpu:0]/GPUMemoryTransferRateOffset[4]=0",
            ]);
            if let Ok(out) = cmd.output() {
                if out.status.success() {
                    crate::log_debug!("[性能调优] GPU 频率偏移已安全复位为 0");
                    break;
                }
            }
        }
    }

    /// 开启: 往 0x0727 写入 0x40，机身按键指示灯点亮高贵纯白光，解除功耗墙
    /// 关闭: 往 0x0727 写入 0x00，清空功耗墙序列，机身指示灯切回当前模式 (狂暴红/均衡蓝/办公绿)
    pub fn enable_custom_mode(&self, enabled: bool) -> Result<(), String> {
        if !self.ec.is_available() {
            return Err("未挂载 /proc/acpi/call，无法向 EC 写入控制字".into());
        }

        if enabled {
            // 确保控制台在线 (0x0741 = 0x01)
            let _ = self.ec.write(ADDR_AP_OEM, FLAG_AP_OEM_ONLINE);
            // 写入 0x40 激活自定义模式 (白光)
            self.ec.write(ADDR_CUSTOM_ENABLE, FLAG_CUSTOM_ENABLE)?;
            crate::log_debug!("[性能调优] 原厂自定义模式已启用 (EC 0x0727=0x40 · 机身白光就绪)");
        } else {
            // 写入 0x00 关闭自定义模式，恢复跟随 0x0751
            self.ec.write(ADDR_CUSTOM_ENABLE, FLAG_CUSTOM_DISABLE)?;
            // 清空 0x077E 功耗序列
            let _ = self.ec.write(ADDR_PW_MAGIC1, 0x00);
            let _ = self.ec.write(ADDR_PW_MAGIC2, 0x00);
            let _ = self.ec.write(ADDR_PW_SPL, 0x00);
            let _ = self.ec.write(ADDR_PW_SPPT, 0x00);
            let _ = self.ec.write(ADDR_PW_FPPT, 0x00);
            crate::log_debug!("[性能调优] 原厂自定义模式已关闭 (EC 0x0727=0x00 · 恢复出厂模式跟随)");
        }
        Ok(())
    }

    /// 设置静音狂暴模式 (0x0728 寄存器: 0x01 静音狂暴限速80%, 0x00 高能满血狂暴)
    pub fn set_silent_turbo(&self, enabled: bool) -> Result<(), String> {
        let val = if enabled { FLAG_SILENT_TURBO_ON } else { FLAG_SILENT_TURBO_OFF };
        // 确保控制台在线 (0x0741=0x01)
        let _ = self.ec.write(ADDR_AP_OEM, FLAG_AP_OEM_ONLINE);
        self.ec.write(ADDR_SILENT_TURBO, val)?;
        crate::log_debug!("[性能模式] 狂暴档位调节: {}", if enabled { "静音狂暴 (风扇转速限制 80% 压制啸叫)" } else { "高能满血狂暴 (转速与功耗无限制释放)" });
        Ok(())
    }
}

impl Default for TuningManager {
    fn default() -> Self {
        Self::new()
    }
}
