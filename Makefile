# Author: Aut520 <aut@autkc.com>
# Repository: https://github.com/Aut520/rmechrevo
# License: GPL-3.0

.PHONY: all check-deps frontend test build deb arch packages run-gui run-daemon install uninstall clean distclean help

SHELL := /bin/bash

VERSION ?= $(shell grep '^version = ' Cargo.toml | head -n 1 | cut -d '"' -f 2)
ifeq ($(strip $(VERSION)),)
  VERSION := 0.1.0
endif
REVISION ?=

all: build

check-deps:
	@pkg-config --exists webkit2gtk-4.1 gtk+-3.0 || { \
		echo "缺少编译依赖：webkit2gtk-4.1 或 gtk+-3.0"; \
		echo "Debian/Ubuntu: sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev"; \
		echo "Arch Linux:    sudo pacman -S --needed webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg pkgconf"; \
		exit 1; \
	}

frontend:
	@if [ ! -d frontend/node_modules ]; then \
		cd frontend && npm ci; \
	fi
	@cd frontend && npm run build

test: frontend
	@cargo test --lib

build: check-deps frontend
	@cargo build --release --bin rmechrevo --bin rmechrevo-daemon

deb: build
	@command -v dpkg-deb >/dev/null 2>&1 || { echo "缺少打包工具：dpkg-deb"; exit 1; }
	@./packaging/debian/build-deb.sh "$(VERSION)" "$(REVISION)"

arch: build
	@./packaging/arch/build-arch.sh "$(VERSION)" "$(if $(REVISION),$(REVISION),1)"

packages: deb arch

run-gui:
	@./target/release/rmechrevo

run-daemon:
	@sudo ./target/release/rmechrevo-daemon

install: build
	@install -m 755 target/release/rmechrevo /usr/bin/rmechrevo
	@install -m 755 target/release/rmechrevo-daemon /usr/bin/rmechrevo-daemon
	@install -m 644 packaging/linux/rmechrevo-daemon.service /usr/lib/systemd/system/rmechrevo-daemon.service
	@install -m 644 packaging/linux/rmechrevo.desktop /usr/share/applications/rmechrevo.desktop
	@install -Dm 644 icons/32x32.png /usr/share/icons/hicolor/32x32/apps/rmechrevo.png
	@install -Dm 644 icons/64x64.png /usr/share/icons/hicolor/64x64/apps/rmechrevo.png
	@install -Dm 644 icons/128x128.png /usr/share/icons/hicolor/128x128/apps/rmechrevo.png
	@install -Dm 644 icons/icon.svg /usr/share/icons/hicolor/scalable/apps/rmechrevo.svg
	@systemctl daemon-reload
	@systemctl enable --now rmechrevo-daemon.service

uninstall:
	@systemctl disable --now rmechrevo-daemon.service 2>/dev/null || true
	@rm -f /usr/bin/rmechrevo /usr/bin/rmechrevo-daemon
	@rm -f /usr/lib/systemd/system/rmechrevo-daemon.service
	@rm -f /usr/share/applications/rmechrevo.desktop
	@rm -f /usr/share/icons/hicolor/32x32/apps/rmechrevo.png
	@rm -f /usr/share/icons/hicolor/64x64/apps/rmechrevo.png
	@rm -f /usr/share/icons/hicolor/128x128/apps/rmechrevo.png
	@rm -f /usr/share/icons/hicolor/scalable/apps/rmechrevo.svg
	@systemctl daemon-reload

clean:
	@rm -rf build/ dist/ frontend/dist/

distclean: clean
	@rm -rf target/ gen/

help:
	@echo "rmechrevo 构建指令："
	@echo "  make build                      编译前端与双二进制产物"
	@echo "  make test                       运行前端构建校验与 Rust 单元测试"
	@echo "  make deb [VERSION=x.y.z]        打包 Debian/Ubuntu .deb 到 dist/"
	@echo "  make arch [VERSION=x.y.z]       打包 Arch Linux 测试版 .pkg.tar.zst 到 dist/"
	@echo "  make packages                   同时生成 DEB 与 Arch 测试包"
	@echo "  make install / uninstall        直接安装或卸载到本机系统"
	@echo "  make clean / distclean          清理打包产物或全部编译缓存"
