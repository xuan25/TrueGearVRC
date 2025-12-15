# TrueGear-VRC

[English](README.md) | [简体中文](README_zh.md)

TrueGear-VRC 是一个第三方命令行桥接工具，允许 VRChat 客户端通过 OSC（Open Sound Control）消息与 TrueGear 软件进行通信。

该工具实现了官方的 WebSocket 协议 V1，用于向兼容的 TrueGear 软件发送命令，包括：

- [TrueGear Player](https://www.truegearhaptic.com/download)
- [TrueGear-CLI](https://github.com/xuan25/TrueGearCLI)

## 快速开始

1. 从 [rustup.rs](https://rustup.rs/) 安装 Rust 和 Cargo。
2. 克隆此仓库：
   ```sh
   git clone https://github.com/xuan25/TrueGearVRC.git
   ```
3. 进入项目目录：
   ```sh
   cd TrueGearVRC
   ```
4. 使用 Cargo 构建项目：
   ```sh
   cargo build --release
   ```
5. 启动你的 TrueGear 软件（TrueGear Player 或 TrueGear-CLI），并确保它正在运行且已准备好接受 WebSocket 连接。
6. 运行 TrueGear-VRC：
   ```sh
   cargo run --release
   ```
   随后它会自动通过 WebSocket 连接到 TrueGear 软件。

   你可能会看到如下输出：
   ```sh
   Listening OSC on 0.0.0.0:9005
   ```
7. 配置你的 VRChat 客户端或任何其他支持 OSC 的软件，将 OSC 消息发送到 `localhost:9005`。

## 命令行选项

```
用法：truegear-vrc [选项]

选项：
  -r, --receive-osc-port <RECEIVE_OSC_PORT>
          用于监听传入 OSC 消息的端口 [默认：9001]
  -f, --forward-osc-port <FORWARD_OSC_PORT>
          将接收到的 OSC 消息转发到的端口（设为 0 表示禁用转发）[默认：0]
  -t, --truegear-ws-url <TRUEGEAR_WS_URL>
          TrueGear WebSocket 端点 [默认：ws://127.0.0.1:18233/v1/tact/]
      --shake-intensity <SHAKE_INTENSITY>
          震动强度 [默认：50]
      --electrical-intensity <ELECTRICAL_INTENSITY>
          电击强度 [默认：50]
      --electrical-interval <ELECTRICAL_INTERVAL>
          电击间隔 [默认：10]
      --feedback-mode <FEEDBACK_MODE>
          反馈模式；Once 表示每次激活只发送一次效果，Continuous 表示在激活期间持续发送效果。
          [默认：continuous] [可选值：once, continuous]
  -v, --verbose
          启用详细日志输出
  -h, --help
          打印帮助信息
  -V, --version
          打印版本信息
```
