<div align="center">

# RMechrevo

**Rust 原生重构的 Linux 专属硬件控制中心 · 面向同方 / Uniwill `INOU0000` 模具（主测机型：机械革命 蛟龙 16 Pro 2025）**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Platform: Linux](https://img.shields.io/badge/Platform-Linux-1793d1.svg)](doc/01-installation.md)
[![Built with Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](Cargo.toml)
[![Release](https://img.shields.io/github/v/release/Aut520/rmechrevo?include_prereleases)](https://github.com/Aut520/rmechrevo/releases)

</div>

---

`rmechrevo`（前缀 `r` 代表 **Rust**）是一款专为 Linux 桌面打造的原生硬件控制中心与特权守护服务，面向采用 **同方 / Uniwill `INOU0000` ACPI/EC 协议** 的游戏本模具（完整开发与验证基于 **机械革命 蛟龙 16 Pro 2025**）。项目采用 **Rust + Tauri 2 + Vue 3** 双二进制特权分离架构：后台 `rmechrevo-daemon` 托管底层 EC/ACPI 通信与物理快捷键，桌面端 `rmechrevo` 以普通用户免 `sudo` 运行，完整兼容 Wayland 与 X11。

<p align="center">
  <img src="doc/images/overview.jpg" alt="RMechrevo 概览大厅" width="900" />
</p>

## 界面预览

| 概览大厅 | RGB 键盘 |
| :---: | :---: |
| ![概览大厅](doc/images/overview.jpg) | ![RGB 键盘](doc/images/keyboard.jpg) |

| 电池与显卡 | 系统与补丁 |
| :---: | :---: |
| ![电池与显卡](doc/images/power.jpg) | ![系统与补丁](doc/images/system.jpg) |

## 功能总览

| 模块 | 能力说明 |
|---|---|
| **性能模式** | 四档互斥切换（狂暴 / 均衡 / 办公 / 自定义）、狂暴子模式（满血 / 静音）、暴风强冷（Fan Boost）、机身物理模式键联动 |
| **自定义调优** | 切入自定义模式时展开 CPU PL1/PL2/PL4 功耗墙、GPU 频率偏移与 Dynamic Boost、RamFan2 双风扇 10 阶曲线 |
| **单区 RGB 键盘** | 物理电源通断（`0x078C`）、4 档亮度、常亮（原厂 29 点色准校准）、彩虹、硬件 4 色呼吸（`0x07C5` 红/绿/蓝/白） |
| **机身辅助开关** | Win 键硬件锁定、触控板禁用、Fn 键翻转 |
| **电源与显卡** | 60% / 80% / 100% EC 硬件充电上限；MUX 独显直连 / 混合输出查看与切换；混合模式运行时独显供电控制 |
| **睡眠与休眠防护** | S3/S0ix 睡眠 DSDT (`PEHP=0`) 补丁检测/部署/回滚；NVIDIA S4 休眠防黑屏一键加固与回滚 |
| **外观定制** | 随机壁纸 / 自定义 URL / 本地图片、暗色烟熏与浅色磨砂主题、强调色与磨砂参数持久化 |

## 适配机型与实测通过环境

### 已实测通过的硬件配置

以下实机软硬件配置已完整测试通过（性能模式、自定义调优、单区 RGB、电池养护、辅助开关、S3/S4 防护均正常生效）：

| 项目 | 实测配置 |
|---|---|
| **实测机型** | 机械革命 蛟龙 16 Pro 2025（DMI: `MECHREVO JIAOLONG Series`） |
| **模具 / 主板** | 同方 / Uniwill `INOU0000` 模具 · 主板 `JIAOLONG Series-X6xR55xK-B2` |
| **BIOS 版本** | `N.1.19MRO20`（`12/18/2025`，AMI） |
| **EC 固件版本** | `1.31` |
| **处理器** | AMD Ryzen 9 8940HX with Radeon Graphics |
| **显卡** | NVIDIA GeForce RTX 5070 Max-Q / Mobile (`GB206M`) + AMD Raphael iGPU |
| **操作系统与内核** | Ubuntu 26.04.1 LTS / Linux `7.0.0-44-generic` |

### 适用范围与自检方法

- **主要适配**：机械革命 蛟龙 16 Pro 2025（`X6xR55xK` / `INOU0000` 模具）及同代单区 RGB 键盘版本。
- **潜在兼容**：其他采用同方 / Uniwill `INOU0000` EC 协议族的 OEM 笔记本（如机械革命蛟龙 / 旷世 / 耀世部分型号，以及同模具的 TUXEDO、XMG、Eluktronics 等机型）。
- **快速自检**：在终端执行以下命令，若存在 `INOU0000` 平台设备，说明具备同系列 EC 通道基础（不同机型寄存器细节请核对 [doc/03-hardware-protocol.md](doc/03-hardware-protocol.md)，欢迎通过 Issue 提交新机型与 BIOS/EC 实测报告）：
  ```bash
  ls -d /sys/bus/platform/devices/INOU0000* 2>/dev/null
  cat /sys/class/dmi/id/{sys_vendor,product_name,board_name,bios_version,ec_firmware_release}
  ```

## 快速安装

### 前置内核模块

底层 EC 读写依赖 `acpi_call` 内核模块（Linux 6.19+ 主线内核自带 `uniwill-laptop` 驱动可进一步增强辅助功能）：

```bash
# Ubuntu / Debian
sudo apt install -y acpi-call-dkms && sudo modprobe acpi_call

# Arch Linux
sudo pacman -S --needed acpi_call-dkms && sudo modprobe acpi_call
```

### Debian / Ubuntu（`.deb`）

从 [GitHub Releases](https://github.com/Aut520/rmechrevo/releases) 下载最新 `.deb` 安装包：

```bash
sudo dpkg -i rmechrevo_0.1.0_amd64.deb
sudo apt-get install -f
```

### Arch Linux（测试版 `.pkg.tar.zst` / `PKGBUILD`）

> Arch 打包支持目前处于 **测试阶段**，欢迎反馈实测结果。

```bash
# 方式 A：安装 Release 中的测试包
sudo pacman -U rmechrevo-0.1.0-1-x86_64.pkg.tar.zst

# 方式 B：使用仓库内 PKGBUILD 本地构建
cd packaging/arch && makepkg -si
```

### 从源码构建

```bash
# Ubuntu / Debian 构建依赖
sudo apt update && sudo apt install -y \
  build-essential pkg-config dpkg zstd \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# 构建并打包（产物输出至 dist/）
make deb        # 生成 Debian/Ubuntu .deb
make arch       # 生成 Arch Linux 测试版 .pkg.tar.zst
sudo make install
```

## 日常使用与排错

安装后 `rmechrevo-daemon.service` 自动启用并常驻后台，普通用户直接运行 `rmechrevo` 或从应用菜单启动即可：

```bash
systemctl status rmechrevo-daemon.service          # 查看守护状态
journalctl -u rmechrevo-daemon.service -f          # 实时查看守护日志
sudo RMECHREVO_LOG_LEVEL=DEBUG /usr/bin/rmechrevo-daemon  # 开启调试日志
```

## 文档

- [01. 安装、构建与多发行版打包指南](doc/01-installation.md)
- [02. 特权分离架构、IPC 协议与日志系统](doc/02-architecture.md)
- [03. Uniwill EC / ACPI 硬件控制协议参考](doc/03-hardware-protocol.md)
- [04. S3 睡眠补丁与 S4 休眠防黑屏原理](doc/04-sleep-and-hibernate.md)
- [05. 常见问题与故障排查](doc/05-troubleshooting.md)

## 免责声明与许可

- 本项目直接通过 ACPI/EC 接口读写笔记本底层嵌入式控制器寄存器，并支持修改 initramfs ACPI 表与休眠配置。不同 BIOS/EC 版本可能存在寄存器差异，使用前请确认硬件版本并自行评估风险。
- 本项目与任何笔记本整机厂商无官方关联。
- 基于 **[GNU GPL-3.0](LICENSE)** 协议开源 · Author: [Aut520](https://github.com/Aut520) (`aut@autkc.com`)
