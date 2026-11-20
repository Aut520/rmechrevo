// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs;
use std::sync::atomic::{AtomicU8, Ordering};
use crate::ec::{EcInterface, registers::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbEffectMode {
    Static = 0,
    Rainbow = 1,
    Breathing = 2,
}

impl RgbEffectMode {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => RgbEffectMode::Rainbow,
            2 => RgbEffectMode::Breathing,
            _ => RgbEffectMode::Static,
        }
    }
}

static SAVED_R: AtomicU8 = AtomicU8::new(255);
static SAVED_G: AtomicU8 = AtomicU8::new(255);
static SAVED_B: AtomicU8 = AtomicU8::new(255);
static SAVED_BRIGHTNESS: AtomicU8 = AtomicU8::new(100);
static SAVED_EFFECT: AtomicU8 = AtomicU8::new(0);

pub struct RgbKeyboard {
    ec: EcInterface,
}

impl RgbKeyboard {
    const SYSFS_MULTI_INTENSITY: &'static str = "/sys/class/leds/uniwill:multicolor:status/multi_intensity";
    const SYSFS_BRIGHTNESS: &'static str = "/sys/class/leds/uniwill:multicolor:status/brightness";
    const SYSFS_RAINBOW: &'static str = "/sys/bus/platform/devices/INOU0000:00/rainbow_animation";

    pub fn new() -> Self {
        Self {
            ec: EcInterface::new(),
        }
    }

    pub fn rgb_to_breathing_index(r: u8, g: u8, b: u8) -> u8 {
        const BREATHING_COLORS: &[(u8, u8, u8, u8)] = &[
            (1, 255, 0, 0),
            (2, 0, 255, 0),
            (3, 0, 0, 255),
            (4, 255, 255, 255),
        ];

        let mut best_idx = 4u8;
        let mut min_dist_sq = u32::MAX;
        for &(idx, tr, tg, tb) in BREATHING_COLORS {
            let dr = (r as i32) - (tr as i32);
            let dg = (g as i32) - (tg as i32);
            let db = (b as i32) - (tb as i32);
            let dist_sq = (dr * dr + dg * dg + db * db) as u32;
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_idx = idx;
            }
        }
        best_idx
    }

    fn breathing_index_to_rgb(idx: u8) -> (u8, u8, u8) {
        match idx {
            1 => (255, 0, 0),
            2 => (0, 255, 0),
            3 => (0, 0, 255),
            _ => (255, 255, 255),
        }
    }

    pub fn brightness_to_hardware(percent: u8) -> (u8, u8) {
        if percent == 0 {
            (0x00, 0x00)
        } else if percent <= 25 {
            (0x30, 0x20)
        } else if percent <= 50 {
            (0x50, 0x40)
        } else if percent <= 75 {
            (0x70, 0x60)
        } else {
            (0x90, 0x80)
        }
    }

    pub fn rgb_to_hardware_level(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        const COLOR_TABLE: &[(u8, u8, u8, u8, u8, u8)] = &[
            (255, 255, 255, 50, 40, 40),
            (255, 0, 0, 50, 0, 0),
            (255, 50, 0, 50, 5, 0),
            (255, 80, 0, 50, 10, 0),
            (145, 60, 0, 29, 13, 0),
            (255, 102, 0, 40, 10, 0),
            (255, 128, 0, 40, 15, 0),
            (255, 180, 0, 40, 25, 0),
            (150, 128, 2, 35, 20, 0),
            (255, 204, 0, 50, 40, 0),
            (204, 225, 0, 40, 44, 0),
            (120, 255, 0, 24, 50, 0),
            (60, 115, 18, 12, 23, 4),
            (0, 255, 0, 0, 50, 0),
            (0, 255, 80, 0, 50, 16),
            (0, 255, 180, 0, 50, 35),
            (60, 125, 135, 13, 25, 26),
            (0, 255, 255, 0, 50, 50),
            (0, 180, 255, 0, 35, 50),
            (0, 80, 255, 0, 16, 50),
            (0, 35, 102, 0, 8, 20),
            (0, 0, 255, 0, 0, 50),
            (80, 0, 255, 16, 0, 50),
            (180, 0, 255, 35, 0, 50),
            (110, 45, 100, 21, 9, 19),
            (255, 0, 255, 50, 0, 50),
            (255, 0, 180, 50, 0, 36),
            (255, 0, 80, 50, 0, 6),
            (180, 5, 0, 36, 0, 0),
            (0, 0, 0, 0, 0, 0),
        ];

        let mut min_dist_sq = u32::MAX;
        let mut best_level = (0u8, 0u8, 0u8);

        for &(tr, tg, tb, lr, lg, lb) in COLOR_TABLE {
            let dr = (r as i32) - (tr as i32);
            let dg = (g as i32) - (tg as i32);
            let db = (b as i32) - (tb as i32);
            let dist_sq = (dr * dr + dg * dg + db * db) as u32;
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                best_level = (lr, lg, lb);
            }
        }

        if min_dist_sq <= 324 {
            return best_level;
        }

        let lr = ((r as u32) / 5).min(50) as u8;
        let lg = ((g as u32) / 5).min(50) as u8;
        let lb = ((b as u32) / 5).min(50) as u8;
        (lr, lg, lb)
    }

    fn trigger_hardware_latch(&self, rainbow_mode: bool) {
        let cur_67 = self.ec.read(ADDR_TRIGGER_BYTE).unwrap_or(0);
        let mode_bit = if rainbow_mode { FLAG_RAINBOW_MODE } else { 0x00 };
        let latch_on = (cur_67 & 0x5E) | FLAG_RGB_TRIGGER | mode_bit;
        let _ = self.ec.write(ADDR_TRIGGER_BYTE, latch_on);
        let cur_5d = self.ec.read(ADDR_TRIGGER_BYTE2).unwrap_or(0);
        let _ = self.ec.write(ADDR_TRIGGER_BYTE2, cur_5d | FLAG_RGB_TRIGGER);
        std::thread::sleep(std::time::Duration::from_millis(35));
        let latch_off = (cur_67 & 0x5E) | mode_bit;
        let _ = self.ec.write(ADDR_TRIGGER_BYTE, latch_off);
        let _ = self.ec.write(ADDR_TRIGGER_BYTE2, cur_5d & !FLAG_RGB_TRIGGER);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    fn apply_active_state(&self) -> Result<(), String> {
        let mut brightness = SAVED_BRIGHTNESS.load(Ordering::Relaxed);
        if brightness == 0 {
            brightness = 100;
            SAVED_BRIGHTNESS.store(100, Ordering::Relaxed);
        }
        let effect = RgbEffectMode::from_i32(SAVED_EFFECT.load(Ordering::Relaxed) as i32);
        let (reg_8c_bri, reg_ab_bri) = Self::brightness_to_hardware(brightness);
        let kernel_bri = ((brightness as u32 * 200) / 100) as u8;

        let _ = self.ec.write(ADDR_AP_OEM, FLAG_AP_OEM_ONLINE);
        let cur_66 = self.ec.read(ADDR_SUPPORT_BYTE2).unwrap_or(0);
        let _ = self.ec.write(ADDR_SUPPORT_BYTE2, cur_66 | FLAG_RGB_SUPPORT);

        let cur_8c = self.ec.read(ADDR_SINGLEKBL_ENABLE).unwrap_or(0);
        let _ = self.ec.write(ADDR_SINGLEKBL_ENABLE, (cur_8c & 0x0D) | reg_8c_bri);

        let cur_ab = self.ec.read(ADDR_BRIGHTNESS_MODE).unwrap_or(0);
        let _ = self.ec.write(ADDR_BRIGHTNESS_MODE, (cur_ab & 0x1F) | reg_ab_bri);

        let mut r = SAVED_R.load(Ordering::Relaxed);
        let mut g = SAVED_G.load(Ordering::Relaxed);
        let mut b = SAVED_B.load(Ordering::Relaxed);
        if r == 0 && g == 0 && b == 0 {
            r = 255;
            g = 255;
            b = 255;
        }
        let (level_r, level_g, level_b) = Self::rgb_to_hardware_level(r, g, b);

        match effect {
            RgbEffectMode::Rainbow => {
                let _ = fs::write(Self::SYSFS_RAINBOW, "1\n");
                let _ = fs::write(Self::SYSFS_BRIGHTNESS, format!("{}\n", kernel_bri));

                let cur_c5 = self.ec.read(ADDR_EFFECT_SUB).unwrap_or(0);
                let _ = self.ec.write(ADDR_EFFECT_SUB, cur_c5 & 0xF8);
                let _ = self.ec.write(ADDR_RGB_R, 0);
                let _ = self.ec.write(ADDR_RGB_G, 0);
                let _ = self.ec.write(ADDR_RGB_B, 0);
                let _ = self.ec.write(ADDR_COMPAT_SWITCH, 1);
                let _ = self.ec.write(ADDR_COMPAT_CTRL, 0);

                self.trigger_hardware_latch(true);
            }
            RgbEffectMode::Breathing => {
                let breathing_idx = Self::rgb_to_breathing_index(r, g, b);
                let (br, bg, bb) = Self::breathing_index_to_rgb(breathing_idx);

                let _ = fs::write(Self::SYSFS_RAINBOW, "0\n");
                let _ = fs::write(Self::SYSFS_MULTI_INTENSITY, format!("{} {} {}\n", br, bg, bb));
                let _ = fs::write(Self::SYSFS_BRIGHTNESS, format!("{}\n", kernel_bri));

                let cur_67 = self.ec.read(ADDR_TRIGGER_BYTE).unwrap_or(0);
                if (cur_67 & FLAG_RAINBOW_MODE) != 0 {
                    let _ = self.ec.write(ADDR_TRIGGER_BYTE, cur_67 & 0x5E);
                }

                let cur_c5 = self.ec.read(ADDR_EFFECT_SUB).unwrap_or(0);
                let base_c5 = cur_c5 & 0xF8;
                if (cur_c5 & 0x07) != 0 && (cur_c5 & 0x07) != breathing_idx {
                    let _ = self.ec.write(ADDR_EFFECT_SUB, base_c5);
                    std::thread::sleep(std::time::Duration::from_millis(15));
                }
                let _ = self.ec.write(ADDR_EFFECT_SUB, base_c5 | breathing_idx);
            }
            RgbEffectMode::Static => {
                let _ = fs::write(Self::SYSFS_RAINBOW, "0\n");
                let _ = fs::write(Self::SYSFS_MULTI_INTENSITY, format!("{} {} {}\n", r, g, b));
                let _ = fs::write(Self::SYSFS_BRIGHTNESS, format!("{}\n", kernel_bri));

                let cur_c5 = self.ec.read(ADDR_EFFECT_SUB).unwrap_or(0);
                let _ = self.ec.write(ADDR_EFFECT_SUB, cur_c5 & 0xF8);

                let _ = self.ec.write(ADDR_RGB_R, level_r);
                let _ = self.ec.write(ADDR_RGB_G, level_g);
                let _ = self.ec.write(ADDR_RGB_B, level_b);
                let _ = self.ec.write(ADDR_COMPAT_SWITCH, 1);
                let _ = self.ec.write(ADDR_COMPAT_CTRL, 0);
                let _ = self.ec.write(ADDR_COMPAT_R, level_r);
                let _ = self.ec.write(ADDR_COMPAT_G, level_g);
                let _ = self.ec.write(ADDR_COMPAT_B, level_b);

                self.trigger_hardware_latch(false);
            }
        }
        Ok(())
    }

    pub fn restore_state(&self, enabled: bool, r: u8, g: u8, b: u8, brightness: u8, effect_mode: i32) {
        SAVED_R.store(r, Ordering::Relaxed);
        SAVED_G.store(g, Ordering::Relaxed);
        SAVED_B.store(b, Ordering::Relaxed);
        SAVED_BRIGHTNESS.store(brightness.min(100), Ordering::Relaxed);
        SAVED_EFFECT.store(RgbEffectMode::from_i32(effect_mode) as u8, Ordering::Relaxed);
        if enabled && brightness > 0 {
            let _ = self.apply_active_state();
        } else {
            let _ = self.set_power(false);
        }
    }

    pub fn set_color(&self, r: u8, g: u8, b: u8) -> Result<String, String> {
        SAVED_R.store(r, Ordering::Relaxed);
        SAVED_G.store(g, Ordering::Relaxed);
        SAVED_B.store(b, Ordering::Relaxed);

        if SAVED_EFFECT.load(Ordering::Relaxed) == RgbEffectMode::Rainbow as u8 {
            SAVED_EFFECT.store(RgbEffectMode::Static as u8, Ordering::Relaxed);
        }

        self.apply_active_state()?;
        let msg = format!("已设置背光颜色 RGB=({}, {}, {})", r, g, b);
        crate::log_debug!("[RGB背光] {}", msg);
        Ok(msg)
    }

    pub fn set_power(&self, enabled: bool) -> Result<String, String> {
        if enabled {
            self.apply_active_state()?;
            let msg = "已开启键盘背光".to_string();
            crate::log_debug!("[RGB背光] {}", msg);
            Ok(msg)
        } else {
            let _ = fs::write(Self::SYSFS_BRIGHTNESS, "0\n");
            let _ = fs::write(Self::SYSFS_MULTI_INTENSITY, "0 0 0\n");
            let _ = fs::write(Self::SYSFS_RAINBOW, "0\n");

            let _ = self.ec.write(ADDR_AP_OEM, FLAG_AP_OEM_ONLINE);

            let cur_8c = self.ec.read(ADDR_SINGLEKBL_ENABLE).unwrap_or(0);
            let _ = self.ec.write(ADDR_SINGLEKBL_ENABLE, (cur_8c & 0x0D) | 0x02);

            let cur_ab = self.ec.read(ADDR_BRIGHTNESS_MODE).unwrap_or(0);
            let _ = self.ec.write(ADDR_BRIGHTNESS_MODE, cur_ab & 0x1F);

            let cur_c5 = self.ec.read(ADDR_EFFECT_SUB).unwrap_or(0);
            let _ = self.ec.write(ADDR_EFFECT_SUB, cur_c5 & 0xF8);

            let _ = self.ec.write(ADDR_RGB_R, 0);
            let _ = self.ec.write(ADDR_RGB_G, 0);
            let _ = self.ec.write(ADDR_RGB_B, 0);
            let _ = self.ec.write(ADDR_COMPAT_SWITCH, 0);
            let _ = self.ec.write(ADDR_COMPAT_CTRL, 4);
            let _ = self.ec.write(ADDR_COMPAT_R, 0);
            let _ = self.ec.write(ADDR_COMPAT_G, 0);
            let _ = self.ec.write(ADDR_COMPAT_B, 0);

            self.trigger_hardware_latch(false);

            let msg = "已关闭键盘背光".to_string();
            crate::log_debug!("[RGB背光] {}", msg);
            Ok(msg)
        }
    }

    pub fn set_effect(&self, mode: RgbEffectMode) -> Result<String, String> {
        SAVED_EFFECT.store(mode as u8, Ordering::Relaxed);
        self.apply_active_state()?;
        let msg = format!("已切换灯效: {:?}", mode);
        crate::log_debug!("[RGB背光] {}", msg);
        Ok(msg)
    }

    pub fn set_brightness(&self, percent: u8) -> Result<String, String> {
        let percent = percent.min(100);
        SAVED_BRIGHTNESS.store(percent, Ordering::Relaxed);

        if percent == 0 {
            return self.set_power(false);
        }

        self.apply_active_state()?;
        let msg = format!("已设置亮度 {}%", percent);
        crate::log_debug!("[RGB背光] {}", msg);
        Ok(msg)
    }
}

impl Default for RgbKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::RgbKeyboard;

    #[test]
    fn breathing_index_matches_single_zone_presets() {
        assert_eq!(RgbKeyboard::rgb_to_breathing_index(255, 0, 0), 1);
        assert_eq!(RgbKeyboard::rgb_to_breathing_index(0, 255, 0), 2);
        assert_eq!(RgbKeyboard::rgb_to_breathing_index(0, 0, 255), 3);
        assert_eq!(RgbKeyboard::rgb_to_breathing_index(255, 255, 255), 4);
    }
}
