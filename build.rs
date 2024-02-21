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
    let project_dir = env::current_dir().unwrap();
    let lib_path = project_dir.join("ydClient").join("ydAPI_c++").join(lib_dir);
    // Ensure the path is valid and print it for debugging
    assert!(lib_path.exists(), "Library path does not exist: {:?}", lib_path);
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("Debug: lib_path is {}", lib_path.display());

    // The library name is the same across platforms
    // println!("cargo:rustc-link-lib=yd");
    println!("cargo:rustc-link-lib=dylib=yd");

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
