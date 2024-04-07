// Import necessary crates and modules. Adjust these imports based on your actual project structure and dependencies.
use std::ffi::CString;
use std::ptr::null;

use yd_client_sys::bindings::{YDApi, YDListener, YDMarketData};
use yd_client_sys::create_yd_api;

// Define the Rust listener that implements `YDListener`. This struct will handle callbacks from the YDApi.
struct YDExample1Listener {
    api: YDApi,
    username: CString,
    password: CString,
    app_id: CString,
    auth_code: CString,
    instrument_id: CString,
    max_position: i32,
    max_order_ref: i32,
    has_order: bool,
}

impl YDExample1Listener {
    fn new(
        api: YDApi,
        username: &str,
        password: &str,
        app_id: &str,
        auth_code: &str,
        instrument_id: &str,
        max_position: i32,
    ) -> Self {
        Self {
            api,
            username: CString::new(username).unwrap(),
            password: CString::new(password).unwrap(),
            app_id: CString::new(app_id).unwrap(),
            auth_code: CString::new(auth_code).unwrap(),
            instrument_id: CString::new(instrument_id).unwrap(),
            max_position,
            max_order_ref: 0,
            has_order: false,
        }
    }
}

impl YDExample1Listener {
    fn notify_ready_for_login(&mut self, has_login_failed: bool) {
        if has_login_failed {
            println!("Previous login attempt failed, retrying...");
        }
        if !self.api.login(self.username, self.password, self.app_id, self.auth_code) {
            println!("Cannot login");
            std::process::exit(1);
        }
    }

    fn notify_login(&mut self, error_no: i32, max_order_ref: i32, is_monitor: bool) {
        if error_no == 0 {
            self.max_order_ref = max_order_ref;
            println!("Login successfully.");
        } else {
            println!("Login failed, errorNo={}", error_no);
            std::process::exit(1);
        }
    }

    fn notify_finish_init(&mut self) {
        let instrument_ptr = self.api.get_instrument_by_id(self.instrument_id.clone());

        if !instrument_ptr.is_null() {
            let instrument = unsafe { &*instrument_ptr };
            println!(
                "Instrument {} initialized.",
                self.instrument_id.to_string_lossy()
            );
            // Further actions with `instrument`...
        } else {
            println!(
                "Cannot find instrument {}",
                self.instrument_id.to_string_lossy()
            );
            std::process::exit(1);
        }
    }

    fn notify_market_data(&mut self, market_data: &YDMarketData) {
        // Market data processing logic
        // This might involve checking for certain conditions to place or cancel orders
        println!("Received market data update.");
    }

    // Implement other callback methods as necessary, following the logic from the provided C++ example.
    // This might include methods like `notify_order`, `notify_trade`, and any other callbacks relevant to your application.
}

fn main() {
    let config_filename = "examples/config.txt";
    let username = "example_user";
    let password = "example_password";
    let instrument_id = "example_instrument";
    let max_position = 3;

    let api_ptr = create_yd_api(&config_filename);
    let mut listener = YDExample1Listener::new(api_ptr, username, password, "", "", instrument_id, max_position);

    // Ensure `api_ptr` is not null
    if api_ptr.is_null() {
        eprintln!("Failed to create YDApi.");
        std::process::exit(1);
    }

    // Convert `listener` to a raw pointer
    let listener_ptr: *mut YDListener = &mut listener as *mut _ as *mut YDListener;

    // Use an unsafe block to dereference `api_ptr` and call `start`
    unsafe {
        if !((*(*api_ptr).vtable_).YDApi_start)(api_ptr, listener_ptr) {
            eprintln!("Failed to start YDApi.");
            std::process::exit(1);
        }
    }
}
