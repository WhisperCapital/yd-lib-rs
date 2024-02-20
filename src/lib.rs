#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(unused_variables, unused_mut)]
#![allow(clippy::explicit_auto_deref)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod ffi_utils;
pub use ffi_utils::*;
use std::ffi::CString;

// Utility function to create a YDApi instance.
pub fn create_yd_api(config_filename: &str) -> *mut YDApi {
    let cstr_config = CString::new(config_filename).unwrap();
    unsafe { makeYDApi(cstr_config.as_ptr()) }
}
