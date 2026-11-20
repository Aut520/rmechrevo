// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use rmechrevo_core::daemon_service::run_daemon_server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rmechrevo_core::logger::init_log_level();
    let is_root = unsafe { libc::geteuid() == 0 };
    if !is_root {
        eprintln!("错误：rmechrevo-daemon 需要 root 权限运行 (sudo systemctl start rmechrevo-daemon)");
        std::process::exit(1);
    }

    unsafe {
        libc::signal(libc::SIGINT, handle_exit_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, handle_exit_signal as *const () as libc::sighandler_t);
    }

    run_daemon_server()
}

extern "C" fn handle_exit_signal(_sig: libc::c_int) {
    rmechrevo_core::daemon_service::flush_pending_config();
    rmechrevo_core::daemon_service::restore_hardware_independent_mode();
    let _ = std::fs::remove_file(rmechrevo_core::ipc::IPC_SOCKET_PATH);
    std::process::exit(0);
}
