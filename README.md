# airadb

Interactive Android wireless debugging pairing for macOS.

`airadb` wraps the ADB wireless debugging flow so you do not have to remember the pairing and connect commands. It shows a QR code, watches ADB and Bonjour for the phone, falls back to manual `IP:port` entry when Android exposes a stale endpoint, and can launch `scrcpy` after the device is ready.

## Install

Install the latest GitHub release:

```sh
curl -fsSL https://github.com/ovitrif/airdroid/releases/latest/download/install.sh | sh
```

Pin a release or install somewhere else:

```sh
curl -fsSL https://github.com/ovitrif/airdroid/releases/latest/download/install.sh | \
  AIRADB_INSTALL_TAG=v0.1.0 AIRADB_INSTALL_DIR="$HOME/.local/bin" sh
```

Or build from source:

```sh
git clone https://github.com/ovitrif/airdroid.git
cd airdroid
cargo build --release
```

## Usage

```sh
airadb
```

On your Android phone, go to Developer options -> Wireless debugging, then follow the prompts. If a device is already connected through ADB, `airadb` skips pairing and offers the `scrcpy` options immediately.
