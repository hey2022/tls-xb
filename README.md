# tls-xb

tls-xb is a cli tool that fetches scores and GPA from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Features

- Access weighted and unweighted GPA, even when hidden.
- View detailed subject scores for all subjects.
- View scores from previous semesters.
- Color coded scores depending on performance.
- Tabled output.

## Prerequisites

- tls-xb uses [viuer](https://github.com/atanunq/viuer) to display the captcha,
  so a terminal supporting one of the [supported graphics protocols](https://docs.rs/crate/viuer/latest)
  is needed. Terminals supporting sixel be can found [here](https://www.arewesixelyet.com).

  Recommended terminals:
  - Windows: [Windows Terminal](https://github.com/microsoft/terminal)
  - macOS: [iTerm 2](https://iterm2.com/)
  - Linux: [Kitty](https://sw.kovidgoyal.net/kitty) or [WezTerm](https://wezfurlong.org/wezterm)

## Install

### From binaries

The [release page](https://github.com/hey2022/tls-xb/releases) contains
precompiled binaries for:

- Windows
   - [Installer](https://github.com/hey2022/tls-xb/releases/latest/download/tls-xb-x86_64-pc-windows-msvc.msi)
   - [Executable](https://github.com/hey2022/tls-xb/releases/download/v0.3.6/tls-xb-x86_64-pc-windows-msvc.zip)
   <!-- should be modified after each release -->
- macOS
   - [Apple Silicon](https://github.com/hey2022/tls-xb/releases/latest/download/tls-xb-aarch64-apple-darwin.tar.xz)
   - [Intel](https://github.com/hey2022/tls-xb/releases/latest/download/tls-xb-x86_64-apple-darwin.tar.xz)
- [Linux](https://github.com/hey2022/tls-xb/releases/latest/download/tls-xb-x86_64-unknown-linux-gnu.tar.xz)

### From source

tls-xb is written in [Rust](https://www.rust-lang.org),
so the Rust [toolchain](https://rustup.rs) will be needed to compile it.

``` sh
cargo install tls-xb

# Git version
cargo install --git https://github.com/hey2022/tls-xb.git
```

## Update

To update tls-xb simply reinstall a newer version.

## Development

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb

# Build
cargo build

# Run
cargo run

# Install
cargo install --path .
```

## Usage

1. In your terminal, run `tls-xb login` to save your login details on your computer.
1. Run `tls-xb` to run the program.

## FAQ

### Can this change my GPA?

No tls-xb only fetches data from <https://tsinglanstudent.schoolis.cn/api>
to calculate your GPA, which does not expose an api to change your GPA.
