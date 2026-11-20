// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

pub const ADDR_CUSTOM_ENABLE: u16 = 0x0727;
pub const ADDR_SILENT_TURBO: u16 = 0x0728;
pub const ADDR_AP_OEM: u16 = 0x0741;
pub const ADDR_COMPAT_SWITCH: u16 = 0x0745;
pub const ADDR_COMPAT_CTRL: u16 = 0x0748;
pub const ADDR_COMPAT_R: u16 = 0x0749;
pub const ADDR_COMPAT_G: u16 = 0x074A;
pub const ADDR_COMPAT_B: u16 = 0x074B;
pub const ADDR_MAFAN: u16 = 0x0751;
pub const ADDR_FAN_CTL: u16 = 0x0751;
pub const ADDR_FAN_L_DUTY: u16 = 0x075B;
pub const ADDR_FAN_R_DUTY: u16 = 0x075C;
pub const ADDR_TRIGGER_BYTE2: u16 = 0x075D;
pub const ADDR_SUPPORT_BYTE2: u16 = 0x0766;
pub const ADDR_TRIGGER: u16 = 0x0767;
pub const ADDR_TRIGGER_BYTE: u16 = 0x0767;
pub const ADDR_RGB_R: u16 = 0x0769;
pub const ADDR_RGB_G: u16 = 0x076A;
pub const ADDR_RGB_B: u16 = 0x076B;

pub const ADDR_PL1_LEGACY: u16 = 0x0730;
pub const ADDR_PL2_LEGACY: u16 = 0x0731;
pub const ADDR_PL4_LEGACY: u16 = 0x0732;
pub const ADDR_PW_MAGIC1: u16 = 0x077E;
pub const ADDR_PW_MAGIC2: u16 = 0x077F;
pub const ADDR_PW_BYTE3: u16 = 0x0780;
pub const ADDR_PW_BYTE4: u16 = 0x0781;
pub const ADDR_PW_BYTE5: u16 = 0x0782;
pub const ADDR_PW_SPL: u16 = 0x0783;
pub const ADDR_PW_SPPT: u16 = 0x0784;
pub const ADDR_PW_FPPT: u16 = 0x0785;

pub const ADDR_SINGLEKBL_ENABLE: u16 = 0x078C;
pub const ADDR_BRIGHTNESS_MODE: u16 = 0x07AB;
pub const ADDR_MODE_INDEX: u16 = 0x07AB;
pub const ADDR_EFFECT_SUB: u16 = 0x07C5;

pub const ADDR_TABLE_F1_DUTY0: u16 = 0x0F20;
pub const ADDR_TABLE_F2_DUTY0: u16 = 0x0F70;

pub const MODE_RAGE_BASE: u8 = 0x10;
pub const MODE_BALANCED_BASE: u8 = 0x00;
pub const MODE_OFFICE_BASE: u8 = 0xA0;
pub const MODE_CUSTOM_BASE: u8 = 0x80;

pub const MODE_RAGE_INDEX: u8 = 0x02;
pub const MODE_BALANCED_INDEX: u8 = 0x00;
pub const MODE_OFFICE_INDEX: u8 = 0x01;
pub const MODE_CUSTOM_INDEX: u8 = 0x03;

pub const FLAG_CUSTOM_ENABLE: u8 = 0x40;
pub const FLAG_CUSTOM_DISABLE: u8 = 0x00;
pub const FLAG_AP_OEM_ONLINE: u8 = 0x01;
pub const FLAG_AP_OEM_OFFLINE: u8 = 0x00;
pub const FLAG_RGB_SUPPORT: u8 = 0x04;
pub const FLAG_RGB_TRIGGER: u8 = 0x20;
pub const FLAG_RAINBOW_MODE: u8 = 0x80;
pub const FLAG_FAN_BOOST: u8 = 0x04;
pub const FLAG_SILENT_TURBO_ON: u8 = 0x01;
pub const FLAG_SILENT_TURBO_OFF: u8 = 0x00;

pub const PW_MAGIC1_VAL: u8 = 0x55;
pub const PW_MAGIC2_VAL: u8 = 0xAA;
pub const PW_FIXED_VAL: u8 = 0xD8;
