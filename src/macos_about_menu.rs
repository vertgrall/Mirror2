//! Redirect the macOS App menu **About** item to our New Tower branded window.

#[cfg(target_os = "macos")]
mod platform {
    use std::cell::RefCell;
    use std::sync::Once;

    use cocoa_foundation::base::{id, nil};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};

    thread_local! {
        static HANDLER: RefCell<id> = const { RefCell::new(nil) };
    }

    static REGISTER: Once = Once::new();

    extern "C" fn show_mirror2_about(_this: &Object, _sel: Sel, _sender: id) {
        let _ = (_this, _sender);
        crate::about_launch::request_about_window();
    }

    fn handler_class() -> &'static Class {
        REGISTER.call_once(|| {
            let mut decl =
                ClassDecl::new("Mirror2AboutMenuHandler", class!(NSObject)).expect("about handler");
            unsafe {
                decl.add_method(
                    sel!(showMirror2About:),
                    show_mirror2_about as extern "C" fn(&Object, Sel, id),
                );
            }
            decl.register();
        });
        Class::get("Mirror2AboutMenuHandler").expect("about handler class")
    }

    fn handler() -> id {
        HANDLER.with(|cell| {
            let mut slot = cell.borrow_mut();
            if *slot == nil {
                unsafe {
                    *slot = msg_send![handler_class(), new];
                }
            }
            *slot
        })
    }

    pub fn install() {
        unsafe {
            let app: id = msg_send![class!(NSApplication), sharedApplication];
            if app == nil {
                return;
            }
            let menubar: id = msg_send![app, mainMenu];
            if menubar == nil {
                return;
            }
            let app_item: id = msg_send![menubar, itemAtIndex: 0usize];
            if app_item == nil {
                return;
            }
            let app_menu: id = msg_send![app_item, submenu];
            if app_menu == nil {
                return;
            }
            let about_item: id = msg_send![app_menu, itemAtIndex: 0usize];
            if about_item == nil {
                return;
            }

            let target = handler();
            let _: () = msg_send![about_item, setTarget: target];
            let _: () = msg_send![about_item, setAction: sel!(showMirror2About:)];
        }
    }
}

#[cfg(target_os = "macos")]
pub use platform::install;

#[cfg(not(target_os = "macos"))]
pub fn install() {}
