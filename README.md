# TrueGear-VRC


TrueGear-VRC is a third-party command-line bridging tool that allows the VRChat client to communicate with TrueGear software via OSC (Open Sound Control) messages.

This tool implements the official WebSocket protocol V1 to send commands to compatible TrueGear software, including:

- [TrueGear Player](https://www.truegearhaptic.com/download)
- [TrueGear-CLI](https://github.com/xuan25/TrueGearCLI)

## Quick Start

1. Install Rust and Cargo from [rustup.rs](https://rustup.rs/).
2. Clone this repository:
   ```sh
   git clone https://github.com/xuan25/TrueGearVRC.git
   ```
3. Navigate to the project directory:
   ```sh
   cd TrueGearVRC
   ```
4. Build the project using Cargo:
   ```sh
   cargo build --release
   ```
5. Launch your TrueGear software (TrueGear Player or TrueGear-CLI) and ensure it is running and ready to accept WebSocket connections.
6. Run TrueGear-VRC:
   ```sh
   cargo run --release
   ```
   It will then automatically connect to the TrueGear software via WebSocket.

   You may see the following output:
   ```sh
   Listening OSC on 0.0.0.0:9005
   ```
7. Configure your VRChat client or any other OSC-capable software to send OSC messages to `localhost:9005`.

## Command-Line Options

```
Usage: truegear-vrc [OPTIONS]

Options:
  -r, --receive-port <RECEIVE_PORT>
          [default: 9001]
  -s, --send-port <SEND_PORT>
          [default: 9002]
  -f, --forward
          
  -t, --truegear-ws-url <TRUEGEAR_WS_URL>
          [default: ws://127.0.0.1:18233/v1/tact/]
      --shake-intensity <SHAKE_INTENSITY>
          [default: 50]
      --electrical-intensity <ELECTRICAL_INTENSITY>
          [default: 30]
      --electrical-interval <ELECTRICAL_INTERVAL>
          [default: 10]
      --feedback-mode <FEEDBACK_MODE>
          [default: continuous] [possible values: once, continuous]
  -v, --verbose
          Enable verbose logging
  -h, --help
          Print help
  -V, --version
          Print version
```
