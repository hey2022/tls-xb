# tls-xb

tls-xb is a cli tool that fetches scores from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Getting Started

### Prerequisites

- tls-xb is written in Rust,
  so a working [Rust installation](https://www.rust-lang.org/) will be needed.
- tls-xb uses [viuer](https://github.com/atanunq/viuer) to display the captcha,
  so a terminal supporting one of the [supported graphics protocols](https://docs.rs/crate/viuer/latest)
  is needed, terminals supporting sixel be can found [here](https://www.arewesixelyet.com/).

### Installation

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb
cargo install --path .
```

Make sure `~/.cargo/bin` is in your PATH  

### Building

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb
cargo build --release
```

### Usage

In your terminal, run `tls-xb login` to generate your login keys,
then run `tls-xb` to start the program.
