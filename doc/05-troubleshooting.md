# 05. 常见问题与故障排查

## 1. 界面提示守护服务离线或硬件操作无响应

1. 检查后台守护服务状态：
   ```bash
   systemctl status rmechrevo-daemon.service
   ```
2. 若未运行，手动启动并设为开机自启：
   ```bash
   sudo systemctl enable --now rmechrevo-daemon.service
   ```
3. 核查通信 Socket 是否存在：
   ```bash
   ls -la /tmp/rmechrevo.sock
   ```

---

## 2. 如何查看后台日志与开启调试模式

- 实时查看守护日志：
  ```bash
  journalctl -u rmechrevo-daemon.service -n 100 -f
  ```
- 默认日志级别为 `INFO`，仅记录关键启动与异常信息。排查硬件寄存器或指令问题时，可临时开启 `DEBUG` 级别：
  ```bash
  sudo systemctl stop rmechrevo-daemon.service
  sudo RMECHREVO_LOG_LEVEL=DEBUG /usr/bin/rmechrevo-daemon
  ```
  排查完成后按 `Ctrl+C` 停止并恢复服务：
  ```bash
  sudo systemctl start rmechrevo-daemon.service
  ```

---

## 3. `acpi_call` 未加载或权限异常

在「系统与补丁 → 驱动通道」中若提示 `acpi_call` 未挂载：

```bash
sudo apt install -y acpi-call-dkms
sudo modprobe acpi_call
ls -la /proc/acpi/call
```

若开启了 UEFI Secure Boot，需要为 DKMS 模块签署 MOK 密钥或在 BIOS 中关闭 Secure Boot，否则内核会拒绝加载第三方 `acpi_call` 模块。

---

## 4. 键盘背光呼吸模式为何只有 4 种颜色？

这是 Uniwill `INOU0000`（实测 EC `1.31`）单区键盘 EC 固件的原生硬件限制：
- **常亮模式**写入 `0x0769`~`0x076B` 独立 RGB 分量，支持任意校准色彩；
- **彩虹模式**由 EC 硬件驱动循环动画，无独立色彩设置；
- **呼吸模式**由 EC 寄存器 `0x07C5` 低 3 位控制，固件仅内置 **烈红 (`1`)、亮绿 (`2`)、纯蓝 (`3`)、纯白 (`4`)** 4 档预设。界面已严格对齐硬件能力，仅在呼吸模式下展示这 4 种可选色彩。

---

## 5. Wayland / NVIDIA 环境下窗口卡顿或渲染异常

GUI 客户端已在启动入口自动启用 WebKitGTK GPU 硬件合成加速环境变量（`WEBKIT_FORCE_COMPOSITING_MODE=1`、`WEBKIT_HARDWARE_ACCELERATION_POLICY=always`）。如在特定 NVIDIA 驱动版本上遇到透明/毛玻璃闪烁或黑窗，可尝试通过以下环境变量启动验证：

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 rmechrevo
```

---

## 6. 软件退出与硬件安全还原

- **关闭窗口**：默认最小化至系统托盘，后台守护持续保持你的性能模式与键盘灯效。
- **彻底退出守护或收到 `SIGINT` / `SIGTERM`**：守护进程会先强制刷盘配置，再执行脱机硬件还原：
  - 将 `0x0741` 置为 `0x00`，将风扇温控交还主板 EC 自主管理；
  - 复位自定义功耗墙与 GPU 频率偏移；
  - 强制解锁 Win 键锁定与触控板禁用，避免退出后键盘/触控板失灵；
  - **不会熄灭当前键盘背光**，确保暗光环境下仍可正常打字。
