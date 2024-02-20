use std::ffi::CStr;

use yd_lib_rs::{create_yd_api, getYDVersion};

pub fn get_api_version() -> Option<String> {
    unsafe {
        let version_c_str = getYDVersion();
        if version_c_str.is_null() {
            None
        } else {
            Some(CStr::from_ptr(version_c_str).to_string_lossy().into_owned())
        }
    }
}

fn main() {
    let config_filename = "examples/config.txt";

    // Example of getting the API version.
    if let Some(version) = get_api_version() {
        println!("YD API Version: {}", version);
    }

    // Creating the YDApi instance.
    let api_ptr = create_yd_api(config_filename);
    if api_ptr.is_null() {
        eprintln!("Failed to create YDApi instance.");
        return;
    }
}
