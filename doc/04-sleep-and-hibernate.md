# 04. S3 睡眠补丁与 S4 休眠防黑屏原理

采用同方 / Uniwill `INOU0000` 模具的独显游戏本（主测机型：**机械革命 蛟龙 16 Pro 2025**，主板 `JIAOLONG Series-X6xR55xK-B2`，BIOS `N.1.19MRO20`，EC `1.31`）在 Linux 下使用独显直连（dGPU Only）或 NVIDIA 闭源驱动时，常见两类电源恢复故障：S3/S0ix 睡眠唤醒黑屏与 S4 休眠恢复黑屏。`rmechrevo` 在「系统与补丁」页内置了完整的检测、一键部署与一键回滚能力。

---

## 1. S3 / S0ix 睡眠防护（`src/acpi_patches.rs`）

### 1.1 问题根因

在独显直连模式下，主板原厂 ACPI DSDT 表中为 PCIe 显卡根端口设置了：

```asl
Name (PEHP, One)
```

这会向 Linux 内核声明 PCIe 热插拔可用。系统进入睡眠再唤醒时，`pciehp` 驱动可能误判独显链路重置为热拔出，导致唤醒后黑屏假死。

### 1.2 自动修补原理

控制中心会自动读取当前系统的 `/sys/firmware/acpi/tables/DSDT`：
1. 搜索 AML 字节码特征 `0x08 'P' 'E' 'H' 'P' 0x01`（`Name (PEHP, One)`），将其精准替换为 `0x00`（`Zero`）；
2. 将 DSDT 头部 OEM Revision（偏移 `24..28`）加 1，确保内核接受 ACPI Table Override；
3. 重新计算并回写 ACPI 表头校验和（偏移 `9`）；
4. 将修补后的表写入 initramfs override 目录，并同时适配 `initramfs-tools`（Ubuntu/Debian）与 `dracut`（Fedora/Arch/部分 Ubuntu 配置），自动重建启动镜像。

### 1.3 验证与回滚

- 重启后在「系统与补丁 → S3 睡眠防护」看到 `DSDT (PEHP=0): 生效中`，或执行 `dmesg | grep -i "Table Upgrade"` 确认内核已加载覆盖表；
- 如需恢复原厂状态，点击界面 **回滚** 按钮即可删除覆盖文件并重建 initramfs。

---

## 2. S4 休眠防黑屏加固（`src/hibernate.rs`）

### 2.1 常见故障原因

Linux 下带大显存 NVIDIA 独显的笔记本执行休眠（Hibernate to Disk）时，容易因以下任一环节缺失导致恢复黑屏或失败：
- 缺少足够大于物理内存的磁盘 Swap 分区/文件，或 `zram` 优先级高于磁盘 Swap；
- 内核启动参数未正确配置 `resume=UUID=...`（及 Swap 文件所需的 `resume_offset`）；
- `dracut` / `initramfs` 未包含 `resume` 模块；
- NVIDIA 驱动未开启显存落盘（`NVreg_PreserveVideoMemoryAllocations=1`），或显存临时转储路径落在内存型 `tmpfs`（`/tmp`）而非磁盘路径（`/var/tmp`），亦或 `nvidia-hibernate` / `nvidia-resume` systemd 服务未启用。

### 2.2 一键加固与回滚机制

「系统与补丁 → S4 休眠防黑屏」实时检测以下 4 项指标：
1. **resume 参数**：核查内核命令行与 `/sys/power/resume`；
2. **SWAP 容量**：对比可用磁盘 Swap 与物理内存大小，并检测 `zram` 优先级冲突；
3. **显存落盘**：核查 `/etc/modprobe.d/` 中 NVIDIA 驱动选项及 `/var/tmp` 磁盘转储目录；
4. **休眠服务与 initramfs**：核查 `nvidia-suspend` / `nvidia-hibernate` / `nvidia-resume` 服务与 `dracut` `resume` 模块。

点击 **一键加固** 会按需补全配置并更新启动镜像；点击 **回滚** 可安全撤销由控制中心写入的休眠配置。
