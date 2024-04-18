# TODO

Currently a stub copy of
https://github.com/grovesNL/glow/blob/main/examples/hello . Once I'm
happy it's working, I'll mutate it into the actual project I want to
work on!

## Building and running

### Native

To run with glutin and winit:

```shell
cargo run --features=glutin_winit
```

To run with sdl2:

```shell
cargo run --features=sdl2
```

### Web

To run with web-sys:

```shell
cargo build --target wasm32-unknown-unknown
mkdir -p generated
wasm-bindgen target/wasm32-unknown-unknown/debug/curved-space.wasm --out-dir generated --target web
cp index.html generated
```

`web.sh` has been provided to do this, for convenience. You may need
to do `cargo install wasm-bindgen-cli` first, if you haven't done wasm
work before.

CORS prevents you opening this as a file in a web browser, but you can
start a small local web browser, e.g. `python3 -m http.server 8080` in
the `generated` directory.

## Design choices

Built on [glow](https://crates.io/crates/glow) as it seems to be
simple, popular, cross-platform (including wasm) and maintained.
