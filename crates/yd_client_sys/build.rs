#![allow(unused_variables, unused_mut, dead_code)]
use clang::*;
use std::{env, fs::File, io::Write, path::PathBuf};
#[macro_use]
extern crate lazy_static;
mod build_utils;

use crate::build_utils::{process_children, HandlerConfigs};
use build_utils::{create_handlers, HandlerMap};

// 路径常量
lazy_static! {
    static ref THIRD_PARTY_PROJECT_DIR: PathBuf = env::current_dir().unwrap().join("thirdparty");
    static ref OUT_PATH: PathBuf = PathBuf::from(env::var("OUT_DIR").unwrap());
}

/// 用于打印编译时的信息方便调试，将打印为 `warning: yd_client_sys@0.1.0: ` 的形式
///
/// @url https://github.com/rust-lang/cargo/issues/985#issuecomment-1071667472
macro_rules! console_debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    generate_type();
    // 开始使用 HPP 的 AST 生成 Rust unsafe fn 的 wrapper
    clang_sys::load().expect("");
    let binding = Clang::new().unwrap();
    let index = Index::new(&binding, false, false);
    let wrapper_hpp_path = THIRD_PARTY_PROJECT_DIR.join("wrapper.hpp");
    let library_header_ast = index.parser(wrapper_hpp_path).parse().unwrap();
    let entity = library_header_ast.get_entity();
    let handlers = create_handlers(); // Assume this function is defined elsewhere
    generate_api_wrapper(&entity, &handlers);
    generate_spi_wrapper(&entity, &handlers);
}

/// 用 bindgen 生成与 C++ 代码兼容的 rust 的类型，生成的东西非常基本，还需要通过 unsafe 调用
fn generate_type() {
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Determine the platform-specific library directory
    let lib_dir = match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux") => "linux64",
        Ok("windows") => "win64",
        _ => panic!("Unsupported OS"),
    };

    let lib_path = THIRD_PARTY_PROJECT_DIR
        .join("ydClient")
        .join("ydAPI_c++")
        .join(lib_dir);
    assert!(
        lib_path.exists(),
        "Library path does not exist: {:?}",
        lib_path
    );

    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
    println!("cargo:rustc-link-lib=dylib=yd");

    let wrapper_header_path = THIRD_PARTY_PROJECT_DIR
        .join("wrapper.hpp")
        .to_str()
        .expect("Path to string conversion failed")
        .to_owned();

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

    let file_path = OUT_PATH.join("bindings.rs");
    console_debug!("file_path {:?}", file_path);
    bindings
        .write_to_file(file_path)
        .expect("Couldn't write bindings!");
}

/// 生成用于主动调用的 API 的 unsafe fn wrapper，以免每次在业务代码里调用 API 都要手动写
fn generate_api_wrapper(entity: &Entity, handlers: &HandlerMap) {
    let mut file_path =
        File::create(OUT_PATH.join("api_wrapper.rs")).expect("Unable to create api_wrapper file");
    console_debug!("file_path {:?}", file_path);

    let mut configs = HandlerConfigs::default();
    configs.record_flavor = build_utils::handlers::handle_record::RecordFlavor::API;
    let lines = process_children(entity, handlers, &mut configs);
    let file_content = lines.join("");
    file_path
        .write_all(file_content.as_bytes())
        .expect("Failed to write to api_wrapper.rs");
}

/// 生成用于被库调用的回调函数（在 C++ 生态里称为 SPI）的 unsafe fn wrapper，以免每次在业务代码里调用 API 都要手动写
fn generate_spi_wrapper(entity: &Entity, handlers: &HandlerMap) {
    let mut file_path =
        File::create(OUT_PATH.join("spi_wrapper.rs")).expect("Unable to create spi_wrapper file");
    console_debug!("file_path {:?}", file_path);

    let mut configs = HandlerConfigs::default();
    configs.record_flavor = build_utils::handlers::handle_record::RecordFlavor::SPI;
    let lines = process_children(entity, handlers, &mut configs);
    let file_content = lines.join("");
    file_path
        .write_all(file_content.as_bytes())
        .expect("Failed to write to spi_wrapper.rs");

    // Debug 用，打印所有节点
    // library_header_ast.get_entity().visit_children(|e, _parent| {
    //     let name = e.get_display_name();
    //     let kind = e.get_type();
    //     if let Some(name) = name {
    //         if let Some(kind) = kind {
    //             console_debug!("name {name} ({:?})", kind);
    //         }
    //     }
    //     EntityVisitResult::Recurse
    // });
}
