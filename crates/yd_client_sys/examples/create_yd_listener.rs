// crates/yd_client_sys/examples/create_yd_listener.rs

use std::ffi::CString;
use yd_client_sys::{create_yd_api_and_spi, spi_wrapper::YDListenerOutput};
use futures::StreamExt;
use log::info;

fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}

#[tokio::main]
async fn main() {
    init_logger();
    let config_filename = "crates/yd_client_sys/examples/config.txt";

    // Create the API and SPI using the configuration file
    let (mut api, mut spi_stream) = create_yd_api_and_spi(config_filename);

    // Convert your login details to CString
    let username = CString::new("your_username").unwrap();
    let password = CString::new("your_password").unwrap();
    let app_id = CString::new("your_app_id").unwrap();
    let auth_code = CString::new("your_auth_code").unwrap();

    // Attempt to log in
    if api.login(username, password, app_id, auth_code) {
        info!("Login request sent successfully.");
    } else {
        info!("Failed to send login request.");
    }

    // Listen for SPI events
    while let Some(spi_event) = spi_stream.next().await {
        match spi_event {
            YDListenerOutput::NotifyLogin(packet) => {
                if packet.error_no == 0 {
                    info!("Login successful.");
                    // Proceed with additional operations after successful login...
                } else {
                    info!("Login failed with error: {}", packet.error_no);
                }
            },
            YDListenerOutput::NotifyEvent(packet) => {
                info!("Received event: {}", packet.api_event);
            },
            // Handle other events as needed...
            _ => {}
        }
    }
}
