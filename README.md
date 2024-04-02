# tls-xb

tls-xb is a cli tool that fetches scores from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Installation

### Prerequisites

- tls-xb is written in [Rust](https://www.rust-lang.org/),
  so a working Rust [installation](https://rustup.rs/) will be needed.
- tls-xb uses [viuer](https://github.com/atanunq/viuer) to display the captcha,
  so a terminal supporting one of the [supported graphics protocols](https://docs.rs/crate/viuer/latest)
  is needed, terminals supporting sixel be can found [here](https://www.arewesixelyet.com/).

Before building tls-xb you need to clone the git repository:

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb
```

### Install

``` sh
cargo install --path .
```

Make sure `~/.cargo/bin` is in your PATH

### Updating

In the git repository, pull the latest changes with `git pull`,
then follow the installation instructions.

### Building

``` sh
cargo build --release
```

This will place the binary at `target/release/tls-xb`

## Usage

In your terminal, run `tls-xb login` to generate your login keys,
then run `tls-xb` to start the program.
