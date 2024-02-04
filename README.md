# tls-xb

tls-xb is a cli tool that fetches scores from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Getting Started

tls-xb is written in Rust,
so a working [Rust installation](https://www.rust-lang.org/) will be needed.

### Installation

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb
cargo install --path .
```

Make sure `~/.cargo/bin` is in your PATH then run `tls-xb` in your terminal.

### Building

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb
cargo build --release
```

## Configuration

Create a config.toml file with the `name` and `password` key/value pairs.

For example:

``` toml
name = "your-name"
password = "your-password"
```
