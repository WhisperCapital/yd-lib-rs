use yd_client_sys::get_api_version;

#[test]
fn test_get_api_version() {
    let version = get_api_version().expect("Failed to get API version");
    assert!(!version.is_empty(), "API version should not be empty.");
    assert_eq!(version, "1.386.40.0", "API version should be updated.");
}
