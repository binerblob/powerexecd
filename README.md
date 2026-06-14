# powerexecd

`powerexecd` exists because I couldn't get `zpoweralertd` to compile. Unlike `zpoweralertd` it is not intended to be a Rust rewrite of `poweralertd`

For reference, [`zpoweralertd`](https://github.com/mrusme/zpoweralertd) is itself a Zig reimplementation of [`poweralertd`](https://github.com/kennylevinsen/poweralertd/).

## How to Build
```sh
cargo build --release
```

## Usage
Just run:
```sh
powerexecd --help
```
