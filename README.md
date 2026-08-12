<div align="center">
  <img src="public/walinux.png" alt="WaLinux logo" width="112" height="112">
  <h1>WaLinux</h1>
  <p>An unofficial, lightweight WhatsApp Web desktop client for Linux.</p>

  [![CI](https://github.com/salarzeidanlou/WaLinux/actions/workflows/ci.yml/badge.svg)](https://github.com/salarzeidanlou/WaLinux/actions/workflows/ci.yml)
  [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://v2.tauri.app/)
</div>

> [!IMPORTANT]
> WaLinux is an independent community project. It is not affiliated with, endorsed by, or sponsored by WhatsApp LLC or Meta Platforms, Inc. WhatsApp is a trademark of its respective owner.

WaLinux wraps WhatsApp Web in a small [Tauri](https://v2.tauri.app/) application and adds Linux desktop integration. It started as a learning project for Rust, web technologies, and Tauri and is now open for community contributions.

## Features

- Native desktop notifications
- System tray with show and quit actions
- Persistent WhatsApp Web session
- Window size and position restoration
- Single-instance behavior
- Download handling with automatic Images, Videos, Documents, and Others folders
- Debian, RPM, and AppImage bundle targets
- No project-owned backend or added analytics

## Project status

WaLinux is early-stage software. WhatsApp Web can change without notice, so integrations such as notifications and downloads may occasionally need updates. Please report reproducible problems through the [issue tracker](https://github.com/salarzeidanlou/WaLinux/issues).

## Requirements

- Linux with WebKitGTK 4.1
- Node.js 20.19+ or 22.12+
- npm
- The stable Rust toolchain
- An existing WhatsApp account

On Debian or Ubuntu, install Tauri's system dependencies with:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

For Arch, Fedora, openSUSE, and other distributions, follow the official [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Build from source

```bash
git clone https://github.com/salarzeidanlou/WaLinux.git
cd WaLinux
npm ci
npm run tauri dev
```

Create installable packages with:

```bash
npm run tauri build
```

Generated packages are written under `src-tauri/target/release/bundle/`. Prebuilt packages will also be published on the [Releases page](https://github.com/salarzeidanlou/WaLinux/releases) as they become available.

## Downloads

Downloaded files are saved under `~/Downloads/WaLinux/` and organized by detected file type:

```text
WaLinux/
├── Documents/
├── Images/
├── Others/
└── Videos/
```

If a file already exists, WaLinux opens the existing copy instead of overwriting it.

## Development commands

| Command | Purpose |
| --- | --- |
| `npm run dev` | Start the Vite frontend server |
| `npm run build` | Type-check and build the frontend |
| `npm run tauri dev` | Run the desktop app in development mode |
| `npm run tauri build` | Build production Linux packages |
| `npm run check:rust` | Check the Rust application |
| `npm run format:rust` | Verify Rust formatting |

## Privacy and security

WaLinux loads `https://web.whatsapp.com` and stores its WebView session locally on your computer. This project does not operate a proxy service, collect telemetry, or add analytics. Your use of WhatsApp remains subject to WhatsApp's terms and privacy policy.

Please do not publish security vulnerabilities in a public issue. See [SECURITY.md](SECURITY.md) for the reporting process.

## Contributing

Contributions are welcome, including documentation improvements, bug reports, packaging help, and Rust or Tauri changes. Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## License

WaLinux is available under the [MIT License](LICENSE).
