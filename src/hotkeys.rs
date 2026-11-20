// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyStatus {
    Connected(PathBuf),
    PermissionDenied(PathBuf),
    DeviceNotFound,
}

#[allow(dead_code)]
#[repr(C)]
struct RawInputEvent {
    tv_sec: i64,
    tv_usec: i64,
    type_: u16,
    code: u16,
    value: i32,
}

pub struct HotkeyListener {
    _running: Arc<AtomicBool>,
}

impl HotkeyListener {
    pub fn start<F>(on_hotkey_pressed: F) -> (Self, HotkeyStatus)
    where
        F: Fn() + Send + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let status = Self::detect_device();

        match &status {
            HotkeyStatus::Connected(path) => {
                let path_clone = path.clone();
                let running_clone = Arc::clone(&running);

                thread::spawn(move || {
                    Self::listen_loop(path_clone, running_clone, on_hotkey_pressed);
                });
            }
            _ => {}
        }

        (Self { _running: running }, status)
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self._running.store(false, Ordering::SeqCst);
    }

    pub fn detect_device() -> HotkeyStatus {
        let input_class = Path::new("/sys/class/input");
        if let Ok(entries) = fs::read_dir(input_class) {
            for entry in entries.flatten() {
                let name_path = entry.path().join("device/name");
                if let Ok(name) = fs::read_to_string(&name_path) {
                    if name.trim().contains("Uniwill") && name.trim().contains("hotkeys") {
                        let file_name = entry.file_name();
                        let dev_path = PathBuf::from("/dev/input").join(file_name);

                        return match File::open(&dev_path) {
                            Ok(_) => HotkeyStatus::Connected(dev_path),
                            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                                HotkeyStatus::PermissionDenied(dev_path)
                            }
                            Err(_) => HotkeyStatus::DeviceNotFound,
                        };
                    }
                }
            }
        }

        HotkeyStatus::DeviceNotFound
    }

    fn listen_loop<F>(path: PathBuf, running: Arc<AtomicBool>, callback: F)
    where
        F: Fn(),
    {
        crate::log_debug!("[热键监听] 启动物理按键监听线程: {:?}", path);
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                crate::log_error!("[热键监听] 打开设备失败: {:?}, 错误: {}", path, e);
                return;
            }
        };

        let mut buf = [0u8; 24];
        while running.load(Ordering::SeqCst) {
            match file.read_exact(&mut buf) {
                Ok(_) => {
                    let type_ = u16::from_ne_bytes([buf[16], buf[17]]);
                    let code = u16::from_ne_bytes([buf[18], buf[19]]);
                    let value = i32::from_ne_bytes([buf[20], buf[21], buf[22], buf[23]]);

                    // EV_KEY (0x01)，并且 value == 1 (按下事件)
                    if type_ == 1 && value == 1 {
                        if code == 184 {
                            crate::log_debug!("[热键监听] 捕获到物理模式切换键按下! code=184");
                            callback();
                        } else {
                            crate::log_debug!("[热键监听] 收到硬件驱动状态通知事件 (code={}), 忽略非按键动作", code);
                        }
                    }
                }
                Err(e) => {
                    crate::log_error!("[热键监听] 读取事件中断: {}", e);
                    break;
                }
            }
        }
    }
}
