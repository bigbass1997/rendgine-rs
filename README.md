[![License: BSD 2-Clause](https://img.shields.io/badge/License-BSD%202--Clause-blue)](LICENSE)
### Description
A rendering engine/framework built on SDL2 (window/context generation and input handling), and Modern OpenGL (graphics). This library is intended primarily for personal use, thus it is not on crates.io at this time and provides minimal documentation. If you wish to use it anyways, add this to your project's Cargo.toml:
```TOML
rendgine-rs = { git = "https://github.com/bigbass1997/rendgine-rs" }
```
Note you must run `cargo update` to pull any new commits.

### Building
If you wish to build from source, for your own system, Rust is integrated with the `cargo` build system. To install Rust and `cargo`, just follow [these instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html). Once installed, while in the project directory, run `cargo build --release` to build, or use `cargo run --release` to run directly. The built binary will be available in `./out/release/rendgine-rs/`

To cross-compile builds for other operating systems, you can use [rust-embedded/cross](https://github.com/rust-embedded/cross).