
use std::env;
use std::fs::*;
use std::fmt::Write;
use std::io::Write as OtherWrite;

fn main() {
    let target = env::var("TARGET");
    println!("cargo:warning=Building for {:?} target!", target);
    // match target {
    //     Ok(s) if s == "wasm32-unknown-emscripten" => {
    //         println!("cargo:rustc-cdylib-link-arg=-s");
    //         println!("cargo:rustc-cdylib-link-arg=USE_GLFW=3");
    //     },
    //     _ => ()
    // }
    println!("cargo:rustc-link-lib=c"); // the "-l" flag
}