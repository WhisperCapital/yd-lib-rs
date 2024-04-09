use std::ffi::CStr;
use yd_client_sys::{bindings::getYDVersion, create_yd_api_and_spi};
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref VERSION: String = "1.386.40.0".to_string();
}

fn get_api_version1() -> Option<String> {
    unsafe {
        let version_c_str = getYDVersion(); // Adjusted to use the bindings module
        if version_c_str.is_null() {
            None
        } else {
            Some(CStr::from_ptr(version_c_str).to_string_lossy().into_owned())
        }
    }
}

#[test]
fn test_get_api_version() {
    let version = get_api_version1().expect("Failed to get API version");
    assert!(!version.is_empty(), "API version should not be empty.");
    assert_eq!(version, *VERSION, "API version should be updated.");
}

fn get_api_version2(config_filename: &str) -> Option<String> {
    // Create the API and SPI, but we'll only use the API here
    let (mut api, _) = create_yd_api_and_spi(config_filename);

    // Call the get_version method
    unsafe {
        let version_c_str = api.get_version();
        if version_c_str.is_null() {
            None
        } else {
            Some(CStr::from_ptr(version_c_str).to_string_lossy().into_owned())
        }
    }
}

#[test]
fn test_get_api_version2() {
    // Obtain the current working directory
    let mut config_path = std::env::current_dir().expect("Failed to get current directory");
    // Append the relative path to the config.txt file
    // Adjust the relative path as needed based on the location you're running the test from
    config_path.push("examples/config.txt");
    assert!(config_path.exists(), "Config file does not exist at the expected path {:?}", config_path);
    let config_path_str = config_path.to_str().expect("Failed to convert config path to string");
    let version =
        get_api_version2(config_path_str).expect("Failed to get API version using getVersion2");
    assert!(
        !version.is_empty(),
        "API version should not be empty using getVersion2."
    );
    assert_eq!(
        version, *VERSION,
        "API version should match the expected value using getVersion2."
    );
}
