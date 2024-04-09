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
use generated::spi_wrapper::create_spi;
use generated::spi_wrapper::YDListenerStream;
use generated::spi_wrapper::YDListenerTrait;
use std::ffi::{CStr, CString};

pub fn create_yd_api(config_filename: &str) -> Box<YDApi> {
    let cstr_config = CString::new(config_filename).unwrap();

    // Call the unsafe function to create an instance of YDApi
    let api_ptr = unsafe { makeYDApi(cstr_config.as_ptr()) };

    // Ensure that api_ptr is not null
    if api_ptr.is_null() {
        panic!("Failed to create YDApi instance, get null pointer.");
    }

    // Dereference the raw pointer to get YDApi and encapsulate it in the safe wrapper
    // Assuming YDApi's constructor or a conversion method is available to encapsulate the raw pointer
    unsafe { YDApi::from_raw(api_ptr) }
}

pub fn create_yd_api_and_spi(config_filename: &str) -> (Box<YDApi>, Box<YDListenerStream<'static>>) {
    let mut api = create_yd_api(config_filename);

    // Initialize the SPI and get the stream
    let (spi_stream, spi_ptr) = create_spi();

    // Register the SPI with the API
    api.start(spi_ptr as *const dyn YDListenerTrait);

    (api, spi_stream)
}

