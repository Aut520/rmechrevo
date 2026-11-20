// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use crate::acpi_patches::AcpiPatchStatus;
use crate::battery::BatteryInfo;
use crate::drivers::DriverStatus;
use crate::hibernate::HibernateStatus;
use crate::tuning::CustomTuningConfig;
use serde::{Deserialize, Serialize};

pub const IPC_SOCKET_PATH: &str = "/tmp/rmechrevo.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum IpcCommand {
    SetMode { mode: i32 },
    SetFanBoost { enabled: bool },
    SetFnLock { enabled: bool },
    SetWinLock { locked: bool },
    SetTouchpadDisable { disabled: bool },
    SetRgbColor { r: u8, g: u8, b: u8 },
    SetRgbPower { enabled: bool },
    SetRgbBrightness { percent: u8 },
    SetRgbEffect { effect_mode: i32 },
    SetBatteryLimit { mode: i32 },
    SetCustomTuning { config: CustomTuningConfig },
    SetCustomMode { enabled: bool },
    SetSilentTurbo { enabled: bool },
    SetTurboSubMode { sub_mode: i32 },
    SetDgpuRuntimePower { power_on: bool },
    SwitchGpuMode { to_direct: bool },
    InstallDrivers,
    InstallAcpiPatches,
    RollbackAcpiPatches,
    InstallHibernateFix,
    RollbackHibernateFix,
    ExitAndRestoreHardware,
    GetStatus,
    WakeupGui,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcEvent {
    ModeChanged {
        mode: i32,
        mode_name: String,
        source: String,
        result: String,
    },
    FanBoostChanged {
        enabled: bool,
        message: String,
    },
    SilentTurboChanged {
        enabled: bool,
        message: String,
    },
    TurboSubModeChanged {
        sub_mode: i32,
        message: String,
    },
    CustomModeChanged {
        enabled: bool,
        message: String,
    },
    RgbChanged {
        enabled: bool,
        r: u8,
        g: u8,
        b: u8,
        brightness: u8,
        effect_mode: i32,
        message: String,
    },
    BatteryLimitChanged {
        mode: i32,
        mode_name: String,
        message: String,
        battery_info: BatteryInfo,
    },
    TuningResult {
        message: String,
    },
    SwitchChanged {
        switch_name: String,
        state: bool,
        message: String,
    },
    GpuModeResult {
        current_mode: String,
        message: String,
    },
    DriverInstallResult {
        success: bool,
        message: String,
        status: DriverStatus,
    },
    AcpiPatchResult {
        success: bool,
        message: String,
        status: AcpiPatchStatus,
    },
    HibernateFixResult {
        success: bool,
        message: String,
        status: HibernateStatus,
    },
    FullStatus {
        mode: i32,
        turbo_sub_mode: i32,
        custom_mode_enabled: bool,
        fan_boost: bool,
        fn_lock: bool,
        win_lock: bool,
        touchpad_disabled: bool,
        rgb_power: bool,
        rgb_r: u8,
        rgb_g: u8,
        rgb_b: u8,
        rgb_brightness: u8,
        rgb_effect_mode: i32,
        battery_limit_mode: i32,
        battery_info: BatteryInfo,
        tuning: CustomTuningConfig,
        gpu_mode: String,
        driver_status: DriverStatus,
        acpi_patch_status: AcpiPatchStatus,
        hibernate_status: HibernateStatus,
    },
    WakeupRequested,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_full_status_roundtrip() {
        let status = IpcEvent::FullStatus {
            mode: 1,
            turbo_sub_mode: 0,
            custom_mode_enabled: false,
            fan_boost: false,
            fn_lock: false,
            win_lock: false,
            touchpad_disabled: false,
            rgb_power: true,
            rgb_r: 255,
            rgb_g: 255,
            rgb_b: 255,
            rgb_brightness: 100,
            rgb_effect_mode: 0,
            battery_limit_mode: 1,
            battery_info: crate::battery::BatteryManager::get_battery_info(crate::battery::BatteryLimitMode::Balanced),
            tuning: crate::tuning::CustomTuningConfig::default(),
            gpu_mode: "独显直连".to_string(),
            driver_status: crate::drivers::DriverManager::check_status(),
            acpi_patch_status: crate::acpi_patches::AcpiPatchManager::check_status(),
            hibernate_status: crate::hibernate::HibernateManager::check_status(),
        };

        let json = serde_json::to_string(&status).expect("Serialization failed");
        println!("\nJSON: {}", json);
        let _deserialized: IpcEvent = serde_json::from_str(&json).expect("Deserialization failed");
        println!("Deserialized successfully!");
    }

    #[test]
    fn test_ipc_get_status() {
        let cmd = IpcCommand::GetStatus;
        let json = serde_json::to_string(&cmd).unwrap();
        println!("\nGetStatus JSON: {}", json);
        let deserialized: IpcCommand = serde_json::from_str(&json).unwrap();
        println!("GetStatus Deserialized: {:?}", deserialized);
    }
}
