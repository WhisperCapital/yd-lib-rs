use yd_client_sys::spi_wrapper::{YDListenerTrait, YDListenerOutput};
use yd_client_sys::api_wrapper::{YDListener};
use std::ffi::CString;

struct MyListener;

impl<'a> YDListenerTrait<'a> for MyListener {
    fn notify_login(&mut self, error_no: std::os::raw::c_int, max_order_ref: std::os::raw::c_int, is_monitor: bool) {
        if error_no == 0 {
            println!("Login successful!");
            // Implement additional logic on login success, like notifying other systems.
        } else {
            eprintln!("Login failed with error code: {}", error_no);
        }
    }

    // Implement other callbacks as needed...
}
