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

pub fn create_yd_api(config_filename: &str) -> Box<YDApi> {
    let cstr_config = CString::new(config_filename).unwrap();

    // Call the unsafe function to create an instance of YDApi
    let api_ptr = unsafe { makeYDApi(cstr_config.as_ptr()) };

    // Ensure that api_ptr is not null
    if api_ptr.is_null() {
        panic!("Failed to create YDApi instance");
    }

    // Dereference the raw pointer to get YDApi and encapsulate it in the safe wrapper
    // Assuming YDApi's constructor or a conversion method is available to encapsulate the raw pointer
    unsafe { YDApi::from_raw(api_ptr) }
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
