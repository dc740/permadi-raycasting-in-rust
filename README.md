# permadi-raycasting-in-rust
This is a raycasting port of the code from https://permadi.com/1996/05/ray-casting-tutorial-table-of-contents/
It compiles to native or wasm and also has support for drawing sprites.

![Alt text](./screenshot.png?raw=true "Screenshot")

# Run

    cargo run --release

# Run in the browser

    ./build_web.sh


It's a port of the original javascript code, but using minifb and Rust.
I tried to keep faithful to the original code, so a beginner can compare both.
Getting the same code to run on the browser unmodified implies some dark
webassembly magic, but it works.

The sprites support is not documented in the original tutorial, but the code is not
optimized, so it should be (hopefully) easier to follow.

Press F to hide the ceiling.

I'm keeping the original license too.
