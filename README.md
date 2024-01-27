# tls-xb

tls-xb is a cli tool that fetches scores from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Building

``` sh
$ git clone https://github.com/hey2022/tls-xb
$ cd tls-xb
$ cargo build --release
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
