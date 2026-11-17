# Setup

## Supported Hosts

The project is intended to run on:

- macOS
- Linux

The generated output is intended for:

- ESP32 firmware projects.
- Arduino sketches and libraries.
- Raspberry Pi Pico W firmware projects.
- Embedded Rust projects that can include a generated Rust module.

## Rust Toolchain

Install Rust with `rustup`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, confirm Cargo is available:

```sh
cargo --version
```

The crate uses Rust edition 2024. Use a current stable Rust toolchain.

## macOS

Install command line tools if they are not already installed:

```sh
xcode-select --install
```

Build and test:

```sh
make check
make build
```

macOS includes Japanese-capable system fonts such as Hiragino. For sample verification, use a font that contains every requested glyph:

```sh
cargo run -- \
  --config examples/fontgen.toml \
  --font "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc" \
  --output-dir /tmp/font-generator-sample
```

## Linux

Install the Rust toolchain and a C compiler through the distribution package manager.

Debian or Ubuntu example:

```sh
sudo apt update
sudo apt install build-essential pkg-config
```

Install or provide a font that contains the requested glyphs. For Japanese text, a Noto Sans CJK or Noto Sans JP font is a practical default.

Build and test:

```sh
make check
make build
```

Run with an explicit font path:

```sh
cargo run -- \
  --config examples/fontgen.toml \
  --font ./fonts/NotoSansJP-Regular.ttf \
  --output-dir ./generated
```

## VS Code

Recommended extensions:

- `rust-lang.rust-analyzer`
- `tamasfe.even-better-toml`

Useful workspace commands:

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `make check`
- `make build`

No repository-specific VS Code configuration is required for the MVP.
