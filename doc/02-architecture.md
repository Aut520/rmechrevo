# 02. 系统架构、IPC 通信与日志配置

## 1. 特权分离双二进制架构

在 Wayland（Hyprland / GNOME / KDE）与 X11 桌面环境中，GUI 进程以 `root` 身份运行会导致显示会话权限拒绝与安全面扩大；而底层 EC 寄存器读写、ACPI 补丁部署和物理快捷键监听又必须具备 `root` 特权。本项目采用标准 C/S 架构将二者彻底拆分：

```text
+-----------------------------------+        Unix Socket (JSON Lines)       +-----------------------------------+
|  rmechrevo (普通桌面用户进程)      | <====================================> |  rmechrevo-daemon (systemd root)  |
|  - Tauri 2 + Vue 3 前端界面       |        /tmp/rmechrevo.sock            |  - EC / ACPI 串行读写 (Mutex)     |
|  - 系统托盘与无边框毛玻璃窗口      |                                        |  - 物理模式按键监听 (evdev)       |
|  - WebKitGTK GPU 硬件加速         |                                        |  - 2s 遥测采集与多客户端广播      |
|  - 本地外观偏好 (localStorage)    |                                        |  - 配置防抖持久化与脱机硬件还原   |
+-----------------------------------+                                        +-----------------------------------+
```

### 1.1 `rmechrevo-daemon`（`src/bin/daemon.rs` → `src/daemon_service.rs`）

- 由 `packaging/linux/rmechrevo-daemon.service` 以 `root` 身份常驻托管；
- 启动时完成 EC 在线握手（`0x0741 = 0x01`），从磁盘恢复上次保存的性能模式、自定义功耗参数、单区 RGB 背光状态与电池充电上限；
- 所有对 `/proc/acpi/call` 的读写均由 `src/ec/mod.rs` 中的全局 `EC_LOCK: Mutex<()>` 串行化保护，防止遥测线程、快捷键监听线程与前端指令并发读写冲突；
- 收到 `SIGINT` / `SIGTERM` 或前端 `ExitAndRestoreHardware` 指令时，强制落盘未写入的配置并执行硬件脱机还原（清 `0x0741 = 0x00`、解锁 Win 键与触控板、复位自定义功耗墙与超频偏移，但保留当前键盘背光照明）。

### 1.2 `rmechrevo`（`src/bin/gui.rs` → `src/gui_app.rs`）

- 供桌面用户免 `sudo` 直接启动；若意外通过 `sudo rmechrevo` 调用，会自动探测活跃桌面用户并降权拉起；
- 启动时自动配置 WebKitGTK 硬件合成加速环境变量，避免软件光栅化掉帧；
- 后台维护与 `/tmp/rmechrevo.sock` 的断线重连循环，连接建立后主动发送 `GetStatus` 同步全量硬件状态，并接收守护进程每 2 秒广播一次的实时遥测数据。

---

## 2. Unix Socket IPC 协议

传输层采用本地 Unix Domain Socket（`/tmp/rmechrevo.sock`，权限 `0666`），应用层为每行一条的 **JSON Lines**（定义见 `src/ipc.rs`）：

### 客户端指令（`IpcCommand`）

| 指令 | 说明 |
|---|---|
| `GetStatus` | 拉取当前全量硬件状态、驱动通道、ACPI/S4 补丁状态 |
| `SetMode { mode }` | 切换四档互斥性能模式（`0` 狂暴 / `1` 均衡 / `2` 办公 / `3` 自定义） |
| `SetTurboSubMode { sub_mode }` / `SetSilentTurbo { enabled }` | 切换狂暴子模式（`0` 静音狂暴 / `1` 满血狂暴） |
| `SetFanBoost { enabled }` | 开关一键暴风强冷 |
| `ApplyCustomTuning { config }` | 下发并保存自定义 CPU 功耗墙、GPU 偏移与双风扇转速 |
| `SetRgbPower` / `SetRgbBrightness` / `SetRgbColor` / `SetRgbEffect` | 控制单区 RGB 键盘背光开关、亮度、色彩与灯效 |
| `SetBatteryLimit { mode }` | 切换 60% / 80% / 100% 充电上限 |
| `SetWinLock` / `SetTouchpadDisable` / `SetFnLock` | 机身辅助开关控制 |
| `SetGpuMode` / `SetRuntimeDgpuPower` | 显卡输出模式（MUX）与运行时独显供电控制 |
| `CheckDrivers` / `InstallDrivers` | 检测与尝试加载 `acpi_call` / `uniwill` 驱动 |
| `CheckAcpiPatches` / `ApplyAcpiPatches` / `RollbackAcpiPatches` | S3 睡眠 ACPI 补丁检测、部署与回滚 |
| `CheckHibernateFix` / `ApplyHibernateFix` / `RollbackHibernateFix` | S4 休眠防黑屏加固检测、部署与回滚 |
| `ExitAndRestoreHardware` | 强制刷盘并还原 EC 脱机独立模式后退出守护 |

### 服务端推送事件（`IpcEvent`）

包含 `FullStatus`、`Telemetry`、`ModeChanged`、`TurboSubModeChanged`、`FanBoostChanged`、`RgbChanged`、`BatteryLimitChanged`、`TuningResult`、`SwitchChanged`、`GpuModeResult`、`DriverInstallResult`、`AcpiPatchResult`、`HibernateFixResult`、`Error`。

---

## 3. 配置持久化机制

- **配置文件路径**（`src/config.rs`）：系统守护统一持久化至 `/etc/rmechrevo/config.json`，避免开机尚未登录桌面时找不到用户配置；首次切换若检测到旧版 `~/.config/rmechrevo/config.json` 会自动迁移。
- **原子防抖写盘**：状态变更仅标记 `dirty = true`，后台线程每 5 秒检查一次并通过原子 `swap(false)` + 临时文件重命名安全落盘（写入失败自动回滚脏标记），退出前强制刷盘，避免频繁写盘或断电损坏。
- **前端外观配置**：壁纸来源、主题色、毛玻璃模糊度与暗度由前端 `frontend/src/composables/useAppearance.ts` 持久化在 WebView `localStorage`。

---

## 4. 分级日志与查看方式

`src/logger.rs` 提供 4 级日志过滤：`ERROR` / `WARN` / `INFO`（默认）/ `DEBUG`。

- 默认运行在 `INFO` 级别，保持 `systemd` 日志干净不刷屏；
- 查看守护进程日志：
  ```bash
  journalctl -u rmechrevo-daemon.service -f
  ```
- 开启详细调试日志（打印 EC 指令、IPC 事件与配置写盘细节）：
  ```bash
  sudo systemctl edit rmechrevo-daemon.service
  # 添加以下内容后保存并重启服务：
  # [Service]
  # Environment="RMECHREVO_LOG_LEVEL=DEBUG"
  sudo systemctl restart rmechrevo-daemon.service
  ```
