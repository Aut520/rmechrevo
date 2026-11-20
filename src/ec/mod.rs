// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

pub mod registers;

static EC_LOCK: Mutex<()> = Mutex::new(());

pub struct EcInterface {
    acpi_call_path: &'static str,
}

impl EcInterface {
    const ACPI_CALL_PATH: &'static str = "/proc/acpi/call";

    pub fn new() -> Self {
        Self {
            acpi_call_path: Self::ACPI_CALL_PATH,
        }
    }

    pub fn is_available(&self) -> bool {
        Path::new(self.acpi_call_path).exists()
    }

    fn read_unlocked(&self, addr: u16) -> Result<u8, String> {
        let path = Path::new(self.acpi_call_path);
        if !path.exists() {
            return Err("未挂载 /proc/acpi/call".into());
        }

        let cmd = format!("\\_SB.INOU.ECRR 0x{:04X}\0", addr);
        fs::write(self.acpi_call_path, cmd.as_bytes())
            .map_err(|e| format!("写入 EC 读取命令失败 (0x{:04X}): {}", addr, e))?;

        let res = fs::read_to_string(self.acpi_call_path)
            .map_err(|e| format!("读取 EC 结果失败 (0x{:04X}): {}", addr, e))?;
        let s = res.trim().trim_matches('\0');

        if let Some(hex_part) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            u8::from_str_radix(hex_part, 16)
                .map_err(|e| format!("解析十六进制失败 (0x{:04X}): {}", addr, e))
        } else {
            s.parse::<u8>()
                .map_err(|e| format!("解析十进制失败 (0x{:04X}): {}", addr, e))
        }
    }

    fn write_unlocked(&self, addr: u16, val: u8) -> Result<(), String> {
        let path = Path::new(self.acpi_call_path);
        if !path.exists() {
            return Err("未挂载 /proc/acpi/call，请先确认 acpi_call 模块已加载".into());
        }

        let cmd = format!("\\_SB.INOU.ECRW 0x{:04X} 0x{:02X}\0", addr, val);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(self.acpi_call_path)
            .map_err(|e| format!("打开 /proc/acpi/call 失败: {}", e))?;

        file.write_all(cmd.as_bytes())
            .map_err(|e| format!("写入 EC 0x{:04X} 失败: {}", addr, e))?;

        Ok(())
    }

    pub fn read(&self, addr: u16) -> Result<u8, String> {
        let _guard = EC_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        self.read_unlocked(addr)
    }

    pub fn write(&self, addr: u16, val: u8) -> Result<(), String> {
        let _guard = EC_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        self.write_unlocked(addr, val)
    }

    pub fn write_batch(&self, writes: &[(u16, u8)]) -> Result<(), String> {
        let _guard = EC_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        for &(addr, val) in writes {
            self.write_unlocked(addr, val)?;
        }
        Ok(())
    }

    pub fn read_modify_write(&self, addr: u16, mask: u8, value: u8) -> Result<(), String> {
        let _guard = EC_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let current = self.read_unlocked(addr)?;
        let new_val = (current & mask) | (value & !mask);
        self.write_unlocked(addr, new_val)
    }
}

impl Default for EcInterface {
    fn default() -> Self {
        Self::new()
    }
}
