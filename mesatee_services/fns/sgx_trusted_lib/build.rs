fn main() {
    let path = "/teaclave/build/ffi/golib";
    let lib = "math";

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
}
