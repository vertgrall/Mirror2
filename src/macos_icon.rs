//! Set the Dock icon from embedded PNG bytes (cargo run has no .app bundle).

use cocoa_foundation::base::{id, nil};
use objc::{class, msg_send, sel, sel_impl};

/// macOS Dock + Cmd-Tab icon. Window title bars ignore custom icons on modern macOS.
pub fn set_dock_icon(png: &[u8]) {
    if png.is_empty() {
        return;
    }
    unsafe {
        let data: id = msg_send![
            class!(NSData),
            dataWithBytes: png.as_ptr() as *const std::ffi::c_void
            length: png.len()
        ];
        if data == nil {
            return;
        }

        let image: id = msg_send![class!(NSImage), alloc];
        let image: id = msg_send![image, initWithData: data];
        if image == nil {
            return;
        }

        let app: id = msg_send![class!(NSApplication), sharedApplication];
        if app == nil {
            return;
        }
        let _: () = msg_send![app, setApplicationIconImage: image];
    }
}
