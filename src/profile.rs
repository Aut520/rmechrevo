// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use crate::ec::{EcInterface, registers::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    Rage = 0,
    Balanced = 1,
    Office = 2,
    Custom = 3,
}

impl PerformanceMode {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => PerformanceMode::Rage,
            1 => PerformanceMode::Balanced,
            2 => PerformanceMode::Office,
            _ => PerformanceMode::Custom,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }

    pub fn name(&self) -> &'static str {
        match self {
            PerformanceMode::Rage => "狂暴模式 (Rage)",
            PerformanceMode::Balanced => "均衡模式 (Balanced)",
            PerformanceMode::Office => "办公模式 (Office)",
            PerformanceMode::Custom => "自定义模式 (Custom)",
        }
    }

    /// 真实 EC 写入基地址 (0x0751)
    pub fn base_ec_val(&self) -> u8 {
        match self {
            PerformanceMode::Rage => 0x10,
            PerformanceMode::Balanced => 0x00,
            PerformanceMode::Office => 0xa0,
            PerformanceMode::Custom => 0x80,
        }
    }

    /// EC 模式索引寄存器 (0x07AB 低 5 位 / bit 0~4)
    pub fn ec_mode_index(&self) -> u8 {
        match self {
            PerformanceMode::Rage => 0x02,
            PerformanceMode::Balanced => 0x00,
            PerformanceMode::Office => 0x01,
            PerformanceMode::Custom => 0x03,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PerformanceMode::Rage => PerformanceMode::Balanced,
            PerformanceMode::Balanced => PerformanceMode::Office,
            PerformanceMode::Office => PerformanceMode::Rage,
            PerformanceMode::Custom => PerformanceMode::Rage,
        }
    }
}

pub struct ProfileManager {
    ec: EcInterface,
}

impl ProfileManager {
    pub fn new() -> Self {
        Self {
            ec: EcInterface::new(),
        }
    }

    /// 切换性能模式 (带 0x0727 自定义标志及 0x07AB 背光亮度位防护)
    pub fn apply_mode(&self, mode: PerformanceMode) -> Result<String, String> {
        let val_mafan = mode.base_ec_val();
        let val_idx = mode.ec_mode_index() & 0x1F;

        if mode == PerformanceMode::Custom {
            let _ = self.ec.write(ADDR_CUSTOM_ENABLE, FLAG_CUSTOM_ENABLE);
        } else {
            let _ = self.ec.write(ADDR_CUSTOM_ENABLE, FLAG_CUSTOM_DISABLE);
        }

        self.ec.write(ADDR_MAFAN, val_mafan)?;

        let cur_ab = self.ec.read(ADDR_MODE_INDEX).unwrap_or(0x80);
        let new_ab = (cur_ab & 0xE0) | val_idx;
        let _ = self.ec.write(ADDR_MODE_INDEX, new_ab);

        let msg = format!(
            "成功切换为 {} (EC 0x0751写入0x{:02X}, 0x07AB写入0x{:02X})",
            mode.name(),
            val_mafan,
            new_ab
        );
        crate::log_debug!("[性能模式] {}", msg);
        Ok(msg)
    }

    /// 开启：向 0x0767 (FanBoost_Trigger) 写入 0x04 触发双风扇 100% 满血极速狂转
    /// 关闭：向 0x0751 重写当前性能模式值，覆盖清除强冷，恢复平缓静音转速
    pub fn set_fan_boost(&self, enabled: bool, current_mode: PerformanceMode) -> Result<String, String> {
        if enabled {
            self.ec.write(ADDR_TRIGGER, FLAG_FAN_BOOST)?;

            let msg = "暴风强冷已启动！(EC 0x0767 触发器写入 0x04，双风扇 100% 极速狂转)".to_string();
            crate::log_debug!("[风扇强冷] {}", msg);
            Ok(msg)
        } else {
            let _ = self.apply_mode(current_mode);

            let msg = format!(
                "暴风强冷已关闭，恢复 {} 默认转速 (EC 0x0751 写入 0x{:02X})",
                current_mode.name(),
                current_mode.base_ec_val()
            );
            crate::log_debug!("[风扇强冷] {}", msg);
            Ok(msg)
        }
    }

    /// 获取当前真实的硬件性能模式 (读取 0x0751 主控制字与 0x07AB 辅助校验)
    pub fn get_current_mode(&self) -> PerformanceMode {
        if let Ok(val) = self.ec.read(ADDR_MAFAN) {
            match val {
                0x10 => return PerformanceMode::Rage,
                0x00 => return PerformanceMode::Balanced,
                0xA0 => return PerformanceMode::Office,
                0x80 => return PerformanceMode::Custom,
                _ => {}
            }
        }
        if let Ok(val) = self.ec.read(ADDR_MODE_INDEX) {
            match val & 0x1F {
                0x02 => PerformanceMode::Rage,
                0x00 => PerformanceMode::Balanced,
                0x01 => PerformanceMode::Office,
                0x03 => PerformanceMode::Custom,
                _ => PerformanceMode::Balanced,
            }
        } else {
            PerformanceMode::Balanced
        }
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

