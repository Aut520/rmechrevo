# 01. 安装、构建与多发行版打包指南

## 1. 依赖要求

### 运行时依赖

- **Debian / Ubuntu**：`libc6`, `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`, `librsvg2-2`, `systemd`
- **Arch Linux**：`glibc`, `gtk3`, `webkit2gtk-4.1`, `libayatana-appindicator`, `librsvg`, `systemd`
- **内核接口**：
  - `acpi_call`（必需，暴露 `/proc/acpi/call`）
  - `uniwill-laptop`（Linux 6.19+ 主线内置，可选增强热键与部分辅助开关）

### 源码构建工具链

- Rust 1.85+（`edition = "2024"`）
- Node.js 20+ 与 `npm`
- 系统开发包：
  ```bash
  # Debian / Ubuntu
  sudo apt update && sudo apt install -y \
    build-essential pkg-config dpkg zstd \
    libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

  # Arch Linux
  sudo pacman -S --needed base-devel pkgconf zstd \
    webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg nodejs npm rust
  ```

---

## 2. 安装、升级与卸载

### Debian / Ubuntu

```bash
sudo dpkg -i dist/rmechrevo_0.1.0_amd64.deb
sudo apt-get install -f
```

维护脚本在升级前会自动停止运行中的守护服务与活跃 GUI 会话，升级后按原状态平滑恢复；首次安装自动执行 `systemctl enable --now rmechrevo-daemon.service`。

卸载：

```bash
sudo apt remove rmechrevo
# 如需清除持久化配置：
sudo rm -rf /etc/rmechrevo ~/.config/rmechrevo
```

### Arch Linux（测试版）

```bash
# 安装测试包
sudo pacman -U dist/rmechrevo-0.1.0-1-x86_64.pkg.tar.zst

# 或直接通过 PKGBUILD 本地编译安装
cd packaging/arch && makepkg -si

# 卸载
sudo pacman -R rmechrevo
```

---

## 3. Makefile 构建与打包命令

| 命令 | 说明 |
|---|---|
| `make frontend` | 自动检查 `frontend/node_modules` 并构建前端静态资源至 `frontend/dist/` |
| `make test` | 执行前端构建校验与 `cargo test --lib` 单元测试 |
| `make build` | 编译 Release 双二进制（`target/release/rmechrevo` + `target/release/rmechrevo-daemon`） |
| `make deb` | 打包输出 `dist/rmechrevo_<VERSION>_amd64.deb` 并更新 `dist/SHA256SUMS.txt` |
| `make arch` | 打包输出 Arch 测试包 `dist/rmechrevo-<VERSION>-<REV>-x86_64.pkg.tar.zst` 并更新 `dist/SHA256SUMS.txt` |
| `make packages` | 同时构建 DEB 与 Arch 测试包 |
| `make deb VERSION=0.1.0 REVISION=1` | 自定义版本号与修订号 |
| `sudo make install` / `uninstall` | 直接安装或卸载到当前系统 |
| `make clean` | 清理 `build/`、`dist/`、`frontend/dist/`（保留 `target/`） |
| `make distclean` | 深度清理，连同 `target/` 与 `gen/` 一起清除 |

### 产物目录规范

- `target/`：Cargo 编译缓存，不存放最终安装包。
- `build/deb/`、`build/arch/`：各发行版打包组装临时目录。
- `dist/`：统一发布产物输出目录（`.deb`、`.pkg.tar.zst` 与 `SHA256SUMS.txt`）。

---

## 4. GitHub Releases 手动发布工作流

仓库提供 [.github/workflows/release.yml](../.github/workflows/release.yml)（`workflow_dispatch` 手动触发）：

1. 进入 **GitHub Actions → Release → Run workflow**；
2. 填写 **Release tag**（如 `v0.1.0`）与是否标记为 **Pre-release**；
3. 自动执行单元测试、Release 编译、DEB 与 Arch 测试包构建，并创建 GitHub Release 上传 `dist/*.deb`、`dist/*.pkg.tar.zst` 和 `dist/SHA256SUMS.txt`。
