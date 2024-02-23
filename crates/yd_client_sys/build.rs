use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Determine the platform-specific library directory
    let lib_dir = match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux") => "linux64",
        Ok("windows") => "win64",
        _ => panic!("Unsupported OS"),
    };

    let third_party_project_dir = env::current_dir().unwrap().join("thirdparty");
    let lib_path = third_party_project_dir.join("ydClient").join("ydAPI_c++").join(lib_dir);
    assert!(lib_path.exists(), "Library path does not exist: {:?}", lib_path);

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
    println!("cargo:rustc-link-lib=dylib=yd");

    let wrapper_header_path = third_party_project_dir.join("wrapper.hpp").to_str().expect("Path to string conversion failed").to_owned();

    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header(wrapper_header_path)
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++11")
        .clang_arg(format!("-I{}", lib_path.join("include").display())) // Adjust include path as necessary
        .generate()
        .expect("Unable to generate bindings");
    // TODO: fix "expected trait, found struct `YDListener`"

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
