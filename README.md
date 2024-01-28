# tls-xb

tls-xb is a cli tool that fetches scores from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Building

tls-xb is written in Rust,
so a working [Rust installation](https://www.rust-lang.org/) will be needed.

``` sh
git clone https://github.com/hey2022/tls-xb.git
cd tls-xb
cargo build --release
```

## Usage

From project root:

``` sh
./target/release/tls-xb
```

## Configuration

Create config.toml in the project root containing
`name` and `password` key/value pairs.

For example:

``` toml
name = "your-name"
password = "your-password"
```
