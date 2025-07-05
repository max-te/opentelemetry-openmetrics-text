fn main() {
    // For tango benchmark
    println!("cargo:rustc-link-arg-benches=-rdynamic");
    println!("cargo:rerun-if-changed=build.rs");
}
