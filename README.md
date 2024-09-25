# tls-xb

tls-xb is a cli tool that fetches scores and GPA from [Tsinglan Xiaobao](https://tsinglanstudent.schoolis.cn).

## Features

- Access weighted and unweighted GPA, even when hidden.
- View detailed subject scores for all subjects.
- View scores from previous semesters.
- Color coded scores depending on performance.
- Tabled output.

## Prerequisites

- tls-xb is written in [Rust](https://www.rust-lang.org/),
  so a working Rust [installation](https://rustup.rs/) will be needed.
- tls-xb uses [viuer](https://github.com/atanunq/viuer) to display the captcha,
  so a terminal supporting one of the [supported graphics protocols](https://docs.rs/crate/viuer/latest)
  is needed, terminals supporting sixel be can found [here](https://www.arewesixelyet.com/).

## Install

### From source

``` sh
cargo install tls-xb

# git version
cargo install --git https://github.com/hey2022/tls-xb.git
```

### From binaries

The [release page](https://github.com/hey2022/tls-xb/releases) contains
precompiled binaries for Linux, macOS and Windows.

## Development

``` sh
git clone https://github.com/hey2022/tls-xb.git

# Build
cd tls-xb
cargo build

# Install
cargo install --path .
```

## Usage

In your terminal, run `tls-xb login` to login, then run `tls-xb` to run the program.
