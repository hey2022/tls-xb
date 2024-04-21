# tls-xb

tls-xb is a cli tool that fetches scores and GPA from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Prerequisites

- tls-xb is written in [Rust](https://www.rust-lang.org/),
  so a working Rust [installation](https://rustup.rs/) will be needed.
- tls-xb uses [viuer](https://github.com/atanunq/viuer) to display the captcha,
  so a terminal supporting one of the [supported graphics protocols](https://docs.rs/crate/viuer/latest)
  is needed, terminals supporting sixel be can found [here](https://www.arewesixelyet.com/).

## Installation

``` sh
cargo install tls-xb
```

## Building

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb
cargo build --release
./target/release/tls-xb --version
```

## Usage

In your terminal, run `tls-xb login` to login, then run `tls-xb` to run the program.
