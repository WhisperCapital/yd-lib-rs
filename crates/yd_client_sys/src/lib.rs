#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(unused_variables, unused_mut)]
#![allow(clippy::explicit_auto_deref)]

mod generated;
pub use generated::api_wrapper;
pub use generated::bindings;
use generated::bindings::{getYDVersion, makeYDApi, YDApi};
pub use generated::spi_wrapper;

mod ffi_utils;
pub use ffi_utils::*;
use std::ffi::{CStr, CString};

// Utility function to create a YDApi instance.
pub fn create_yd_api(config_filename: &str) -> *mut YDApi {
    let cstr_config = CString::new(config_filename).unwrap();
    unsafe { makeYDApi(cstr_config.as_ptr()) } // Adjusted to use the bindings module
}

pub fn get_api_version() -> Option<String> {
    unsafe {
        let version_c_str = getYDVersion(); // Adjusted to use the bindings module
        if version_c_str.is_null() {
            None
        } else {
            Some(CStr::from_ptr(version_c_str).to_string_lossy().into_owned())
        }
    }
}
