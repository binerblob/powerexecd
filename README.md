# powerexecd

`powerexecd` exists because I couldn't get `zpoweralertd` to compile. It is not intended to be a Rust rewrite of `poweralertd`; rather, it is a small utility that solves a similar problem in a way that works for me.

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
And the program will guide you through.
