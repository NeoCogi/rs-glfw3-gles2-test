
use std::env;

fn main() {
    let target = env::var("TARGET");
    println!("cargo:info=Building for {:?} target!", target);
    println!("cargo:rustc-link-lib=m"); // the "-l" flag
    println!("cargo:rustc-link-lib=c"); // the "-l" flag
}