// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use rmechrevo_core::gui_app::run_gui_app;
use std::env;
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 强制开启 Linux / WebKitGTK 硬件加速 (GPU 渲染)，彻底杜绝 CPU 软件光栅化掉帧
    unsafe {
        if env::var("WEBKIT_FORCE_COMPOSITING_MODE").is_err() {
            env::set_var("WEBKIT_FORCE_COMPOSITING_MODE", "1");
        }
        if env::var("WEBKIT_HARDWARE_ACCELERATION_POLICY").is_err() {
            env::set_var("WEBKIT_HARDWARE_ACCELERATION_POLICY", "always");
        }
        if env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "0");
        }
    }

    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == "--daemon" || arg == "--daemon-only") {
        return rmechrevo_core::daemon_service::run_daemon_server();
    }

    let is_root = unsafe { libc::geteuid() == 0 };

    // 如果以 root 身份运行 (如 sudo rmechrevo)，降权至活跃桌面用户，避免 Wayland / X11 拒绝显示
    if is_root {
        if let Some(target_user) = detect_desktop_user() {
            println!("========================================================");
            println!("  RMechrevo 控制中心 (GUI 客户端)");
            println!("  提示：检测到当前以 root 运行，正在以桌面用户 {} 启动界面...", target_user.username);
            println!("  安装完成后直接从应用菜单启动或执行 `rmechrevo` 即可，无需 sudo。");
            println!("========================================================");

            let mut cmd = Command::new("runuser");
            cmd.args(["-p", "-u", &target_user.username, "--"]);
            cmd.env("WAYLAND_DISPLAY", &target_user.wayland_display);
            cmd.env("DISPLAY", &target_user.display);
            cmd.env("XDG_RUNTIME_DIR", &target_user.xdg_runtime_dir);
            cmd.env("DBUS_SESSION_BUS_ADDRESS", &target_user.dbus_address);
            cmd.env("HOME", &target_user.home);
            if let Ok(desk) = env::var("XDG_CURRENT_DESKTOP") {
                cmd.env("XDG_CURRENT_DESKTOP", desk);
            } else {
                cmd.env("XDG_CURRENT_DESKTOP", "Hyprland");
            }
            if let Ok(exe) = env::current_exe() {
                cmd.arg(exe);
            } else {
                cmd.arg("/usr/bin/rmechrevo");
            }
            let mut child = cmd.spawn()?;
            let _ = child.wait();
            return Ok(());
        }
    }

    run_gui_app()
}

#[derive(Debug)]
#[allow(dead_code)]
struct DesktopUserInfo {
    username: String,
    uid: u32,
    wayland_display: String,
    display: String,
    xdg_runtime_dir: String,
    dbus_address: String,
    home: String,
}

fn detect_desktop_user() -> Option<DesktopUserInfo> {
    let mut username = env::var("SUDO_USER").unwrap_or_default();
    if username == "root" {
        username.clear();
    }

    let mut target_uid = 0u32;
    if !username.is_empty() {
        if let Ok(out) = Command::new("id").args(["-u", &username]).output() {
            if let Ok(uid) = String::from_utf8_lossy(&out.stdout).trim().parse::<u32>() {
                target_uid = uid;
            }
        }
    }

    if username.is_empty() || target_uid == 0 {
        if let Ok(entries) = fs::read_dir("/run/user") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if let Ok(uid) = name.parse::<u32>() {
                        if uid >= 1000 {
                            if let Ok(out) = Command::new("id").args(["-un", &uid.to_string()]).output() {
                                let uname = String::from_utf8_lossy(&out.stdout).trim().to_string();
                                if !uname.is_empty() && uname != "root" {
                                    username = uname;
                                    target_uid = uid;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if username.is_empty() || target_uid == 0 {
        return None;
    }

    let xdg_runtime_dir = format!("/run/user/{}", target_uid);
    let dbus_address = format!("unix:path={}/bus", xdg_runtime_dir);

    let home = unsafe {
        let pwd = libc::getpwuid(target_uid);
        if !pwd.is_null() && !(*pwd).pw_dir.is_null() {
            std::ffi::CStr::from_ptr((*pwd).pw_dir).to_string_lossy().to_string()
        } else {
            format!("/home/{}", username)
        }
    };

    let mut wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();
    if wayland_display.is_empty() {
        if let Ok(entries) = fs::read_dir(&xdg_runtime_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("wayland-") && !fname.ends_with(".lock") {
                    wayland_display = fname;
                    break;
                }
            }
        }
        if wayland_display.is_empty() {
            wayland_display = "wayland-1".to_string();
        }
    }

    let display = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());

    Some(DesktopUserInfo {
        username,
        uid: target_uid,
        wayland_display,
        display,
        xdg_runtime_dir,
        dbus_address,
        home,
    })
}
