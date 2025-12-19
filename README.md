# Proxy5 - High Performance SOCKS5 Server

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tokio](https://img.shields.io/badge/Async-Tokio-green.svg)](https://tokio.rs/)

[English](#english) | [中文](#chinese)

<a name="english"></a>

## 📖 Introduction

**Proxy5** is a lightweight, high-performance, and production-ready SOCKS5 proxy server written in **Rust**.

It is designed with modern asynchronous I/O (Tokio) and follows a modular architecture. It supports standard SOCKS5 protocols (RFC 1928), Username/Password authentication (RFC 1929), and **UDP Associate** for handling DNS or gaming traffic.

Specifically optimized for Linux environments, it implements **Zero-Copy (Splice)** technology to minimize context switching and CPU usage during high-throughput data transmission.

## ✨ Features

- **🚀 High Performance**: Built on `Tokio`, handling thousands of concurrent connections with minimal footprint.
- **⚡ Zero-Copy (Linux)**: Uses the `splice` syscall on Linux to transfer data directly between kernel buffers, bypassing user space for maximum throughput.
- **🛡️ Protocol Support**:
  - **TCP Connect**: Standard TCP proxying.
  - **UDP Associate**: Full UDP support (essential for DNS resolution and gaming).
  - **Authentication**: RFC 1929 Username/Password authentication support.
- **⚙️ Flexible Configuration**: Supports both CLI arguments and `TOML` configuration files.
- **📝 Structured Logging**: Integrated with `tracing` for clear, leveled logs.
- **📦 Production Ready**: Includes Systemd service configuration for Linux deployment.

## 🛠️ Build & Install

### Prerequisites

- [Rust Toolchain](https://rustup.rs/) (1.70+)

### Build

```bash
git clone https://github.com/Xiao-VoV/rust_socks5_server.git
cd proxy5
cargo build --release
```

The binary will be located at `./target/release/proxy5`.

## 🚀 Usage

### 1. Quick Start (CLI)

Run a server on port 1080 without authentication:

```bash
./proxy5 --port 1080

```

Run with authentication:

```bash
./proxy5 --port 1080 --user admin --pass secret123

```

### 2. Configuration File (Recommended)

You can use a config file for more complex setups (e.g., multiple users).

Create `config.toml`:

```toml
ip = "0.0.0.0"
port = 1080
timeout = 300 # Connection timeout in seconds

# Define multiple users
[[users]]
username = "admin"
password = "secure_password"

[[users]]
username = "guest"
password = "123"

```

Run with config:

```bash
./proxy5 --config config.toml

```

### 3. Run as Linux Service (Systemd)

1. Copy binary: `sudo cp ./target/release/proxy5 /usr/local/bin/`
2. Copy config: `sudo mkdir /etc/proxy5 && sudo cp config.toml /etc/proxy5/`
3. Create service file `/etc/systemd/system/proxy5.service`:

```ini
[Unit]
Description=Proxy5 SOCKS5 Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/proxy5 --config /etc/proxy5/config.toml
Restart=always
RestartSec=3
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target

```

1. Start service:

```bash
sudo systemctl enable --now proxy5

```

## 🧪 Testing

### TCP Test

```bash
curl -v --socks5 127.0.0.1:1080 [http://www.google.com](http://www.google.com)
# With auth
curl -v --socks5 -U admin:secret123 127.0.0.1:1080 [http://www.google.com](http://www.google.com)

```

### UDP Test

You can use tools like `v2ray-core` or `badvpn-tun2socks` to test UDP forwarding, or use `dig`:

```bash
# Force DNS query via SOCKS5 UDP
dig @8.8.8.8 google.com +tcp=0 +proxy=127.0.0.1:1080

```

---

<a name="chinese"></a>

## 📖 简介

**Proxy5** 是一个使用 **Rust** 编写的轻量级、高性能且适用于生产环境的 SOCKS5 代理服务器。

它基于现代异步 I/O (Tokio) 构建，采用模块化架构设计。它完全支持标准的 SOCKS5 协议 (RFC 1928)、用户名/密码认证 (RFC 1929) 以及 **UDP Associate**（用于 DNS 解析或游戏流量转发）。

针对 Linux 环境进行了专门优化，利用 **零拷贝 (Splice)** 技术，在内核空间直接传输数据，极大地降低了高并发下的 CPU 占用率。

## ✨ 功能特性

- **🚀 高性能**: 基于 `Tokio` 异步运行时，轻松处理数万并发连接。
- **⚡ 零拷贝 (Zero-Copy)**: 在 Linux 下自动启用 `splice` 系统调用，数据直接在内核缓冲区流转，无需用户态拷贝，吞吐量极高。
- **🛡️ 协议全支持**:
- **TCP Connect**: 标准 TCP 代理。
- **UDP Associate**: 完整的 UDP 转发支持（DNS/游戏加速必备）。
- **身份验证**: 支持 RFC 1929 用户名/密码认证。

- **⚙️ 灵活配置**: 支持命令行参数 (CLI) 和 `TOML` 配置文件。
- **📝 结构化日志**: 集成 `tracing` 库，提供清晰的分级日志输出。
- **📦 生产就绪**: 提供 Systemd 服务文件，易于在 Linux 服务器上部署。

## 🛠️ 构建与安装

### 前置要求

- [Rust 工具链](https://rustup.rs/) (1.70+)

### 编译

```bash
git clone https://github.com/Xiao-VoV/rust_socks5_server.git
cd proxy5
cargo build --release

```

编译完成后，二进制文件位于 `./target/release/proxy5`。

## 🚀 使用指南

### 1. 快速启动 (命令行)

在 1080 端口启动（无认证）：

```bash
./proxy5 --port 1080

```

启用身份验证：

```bash
./proxy5 --port 1080 --user admin --pass secret123

```

### 2. 配置文件 (推荐)

对于多用户等复杂配置，推荐使用配置文件。

创建 `config.toml`:

```toml
ip = "0.0.0.0"
port = 1080
timeout = 300 # 连接超时时间 (秒)

# 配置多个用户
[[users]]
username = "admin"
password = "secure_password"

[[users]]
username = "guest"
password = "123"

```

指定配置文件运行:

```bash
./proxy5 --config config.toml

```

### 3. 部署为 Linux 服务 (Systemd)

1. 复制二进制文件: `sudo cp ./target/release/proxy5 /usr/local/bin/`
2. 复制配置文件: `sudo mkdir /etc/proxy5 && sudo cp config.toml /etc/proxy5/`
3. 创建服务文件 `/etc/systemd/system/proxy5.service`:

```ini
[Unit]
Description=Proxy5 SOCKS5 Server
After=network.target

[Service]
Type=simple
# 请根据实际路径修改
ExecStart=/usr/local/bin/proxy5 --config /etc/proxy5/config.toml
Restart=always
RestartSec=3
# 提高文件描述符限制以支持高并发
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target

```

1. 启动服务:

```bash
sudo systemctl enable --now proxy5

```

## 🧪 测试方法

### TCP 测试 (Curl)

```bash
curl -v --socks5 127.0.0.1:1080 [http://www.baidu.com](http://www.baidu.com)
# 带认证
curl -v --socks5 -U admin:secret123 127.0.0.1:1080 [http://www.baidu.com](http://www.baidu.com)

```

### UDP 测试

你可以使用 `v2ray` 等客户端配置 SOCKS5 Outbound 进行测试，或者使用 `dig` 命令强制走代理 UDP：

```bash
# 注意：并不是所有版本的 dig 都支持 socks 代理，或者使用 nc 测试
# 这里建议编写 Python 脚本或使用 v2ray 验证 UDP 功能

```

## 🏗️ Architecture / 架构

- **`handler.rs`**: Core pipeline control (Handshake -> Auth -> Dispatch).
- **`protocol.rs`**: Request/Response packet parsing and serialization.
- **`udp.rs`**: UDP NAT management and packet routing.
- **`auth.rs`**: RFC 1929 authentication logic.
- **`main.rs`**: Configuration loading and TCP listener loop.

## 📄 License

This project is licensed under the [MIT License](https://www.google.com/search?q=LICENSE).
