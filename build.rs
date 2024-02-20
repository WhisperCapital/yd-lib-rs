use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Determine the platform-specific library directory and file extension
    let lib_dir = if cfg!(target_os = "linux") {
        "linux64"
    } else if cfg!(target_os = "windows") {
        "win64" // Assuming you're targeting 64-bit Windows
    } else {
        panic!("Unsupported OS");
    };

    // Adjust the path to your specific setup if necessary
    let lib_path = format!("ydClient/ydAPI_c++/{}/", lib_dir);
    println!("cargo:rustc-link-search=native={}", lib_path);

    // The library name is the same across platforms
    println!("cargo:rustc-link-lib=yd");

    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++11")
        .clang_arg(format!("-IydClient/ydAPI_c++/include")) // Adjust include path as necessary
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
