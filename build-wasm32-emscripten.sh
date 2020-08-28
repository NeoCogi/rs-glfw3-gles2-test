#!/bin/sh
cargo rustc --release --target wasm32-unknown-emscripten -- -C link_args="-o index.html --no-heap-copy -s USE_WEBGL2=1 -s USE_GLFW=3 -s FULL_ES3=1 --preload-file suzane.obj -s ASSERTIONS=1 -s ALLOW_MEMORY_GROWTH=1"
emrun --serve_after_close index.html