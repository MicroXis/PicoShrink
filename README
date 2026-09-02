# PicoShrink

**PicoShrink** is a lightweight desktop application for compressing PDF files using Ghostscript.

Built with **Rust** and **egui**, PicoShrink provides a simple graphical interface for reducing PDF file size without having to use Ghostscript from the command line.

## Features

* Simple desktop interface
* Multiple compression levels
* Custom output file selection
* Automatic Ghostscript detection
* Manual Ghostscript executable selection
* Native file dialogs
* Cross-platform architecture
* Lightweight native application

## Requirements

PicoShrink uses **Ghostscript** to perform PDF compression.

Ghostscript must currently be installed separately on your system.

### Linux

On Arch Linux:

```bash
sudo pacman -S ghostscript
```

On Debian / Ubuntu:

```bash
sudo apt install ghostscript
```

### Windows

Install Ghostscript and make sure the Ghostscript executable is available on your system.

PicoShrink can also be configured manually by selecting the Ghostscript executable if automatic detection fails.

### macOS

Using Homebrew:

```bash
brew install ghostscript
```

## Compression levels

PicoShrink currently provides three compression profiles:

| Profile      | Ghostscript preset | Description                                |
| ------------ | ------------------ | ------------------------------------------ |
| High Quality | `/prepress`        | Prioritizes image quality                  |
| Balanced     | `/ebook`           | Good balance between quality and file size |
| Strong       | `/screen`          | Prioritizes smaller file size              |

Actual compression results depend on the contents of the PDF. Already optimized PDFs may see little or no size reduction.

## Building from source

### Requirements

* Rust toolchain
* Cargo
* Ghostscript

Clone the repository:

```bash
git clone git@github.com:MicroXis/PicoShrink.git
cd PicoShrink
```

Build the application:

```bash
cargo build --release
```

The resulting executable will be available under:

```text
target/release/
```

To run the development build:

```bash
cargo run
```

## Packaging

PicoShrink uses `cargo-packager` to generate native application packages.

Install it with:

```bash
cargo install cargo-packager --locked
```

Then package the application:

```bash
cargo packager --release
```

Package formats depend on the target operating system.

Linux builds can currently generate packages such as:

* AppImage
* Debian package (`.deb`)
* `.tar.gz`

Windows and macOS packaging is planned for upcoming releases.

## Technology

PicoShrink is primarily built with:

* Rust
* egui / eframe
* Ghostscript
* cargo-packager

## Project status

PicoShrink is currently in early development.

The `0.1.x` releases focus on providing a simple and reliable graphical interface for PDF compression.

## License

PicoShrink is distributed under the MIT License.

See `LICENSE` for details.

