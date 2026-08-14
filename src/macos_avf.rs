//! macOS camera via AVFoundation, 32BGRA.
//!
//! nokhwa's Mac backend copies `CVPixelBufferGetBaseAddress` without handling
//! planar 420v frames (FaceTime's native format). That pointer is null, the
//! delegate never delivers pixels, and the LED still turns on. We ask
//! AVFoundation for packed BGRA instead and copy row-by-row.

#![allow(deprecated)]
#![allow(unexpected_cfgs)]

use std::ffi::{c_char, c_void};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::sync::Arc;
use std::time::Duration;

use cocoa_foundation::base::{id, nil};
use core_foundation::base::TCFType;
use core_foundation::number::CFNumber;
use core_media_sys::CMSampleBufferRef;
use core_video_sys::{
    kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_32BGRA, CVPixelBufferGetBaseAddress,
    CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight, CVPixelBufferGetWidth,
    CVPixelBufferLockBaseAddress, CVPixelBufferRef, CVPixelBufferUnlockBaseAddress,
};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};

#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    static AVMediaTypeVideo: id;
}

#[link(name = "CoreMedia", kind = "framework")]
extern "C" {
    fn CMSampleBufferGetImageBuffer(sbuf: CMSampleBufferRef) -> CVPixelBufferRef;
}

#[link(name = "System", kind = "dylib")]
extern "C" {
    fn dispatch_queue_create(label: *const c_char, attr: *const c_void) -> *mut c_void;
}

const AUTH_AUTHORIZED: isize = 3;

pub struct MacCamera {
    rx: Receiver<(u32, u32, Vec<u8>)>,
    pub name: String,
    _keep: SessionKeep,
}

struct SessionKeep {
    _tx: Arc<SyncSender<(u32, u32, Vec<u8>)>>,
    session: id,
    _input: id,
    _output: id,
    _delegate: id,
    _queue: *mut c_void,
}

unsafe impl Send for SessionKeep {}

impl Drop for SessionKeep {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.session, stopRunning];
        }
    }
}

impl MacCamera {
    pub fn recv_frame(&self, timeout: Duration) -> Result<(u32, u32, Vec<u8>), String> {
        match self.rx.recv_timeout(timeout) {
            Ok(frame) => Ok(frame),
            Err(RecvTimeoutError::Timeout) => Err("timeout".into()),
            Err(RecvTimeoutError::Disconnected) => Err("camera disconnected".into()),
        }
    }

    /// Drain to the newest frame when the pipe is backed up.
    pub fn try_recv_frame(&self) -> Option<(u32, u32, Vec<u8>)> {
        self.rx.try_recv().ok()
    }
}

pub fn open() -> Result<MacCamera, String> {
    wait_for_permission()?;

    unsafe { open_session() }
}

fn wait_for_permission() -> Result<(), String> {
    unsafe {
        let status: isize = msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: AVMediaTypeVideo
        ];
        if status == AUTH_AUTHORIZED {
            return Ok(());
        }
        if status == 1 || status == 2 {
            return Err("camera permission was declined".into());
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    dispatch::Queue::main().exec_async(move || unsafe {
        let wrapper = block::ConcreteBlock::new(move |granted: BOOL| {
            let _ = tx.send(granted == YES);
        });
        let block = wrapper.copy();
        let _: () = msg_send![
            class!(AVCaptureDevice),
            requestAccessForMediaType: AVMediaTypeVideo
            completionHandler: &*block
        ];
        std::mem::forget(block);
    });

    match rx.recv_timeout(Duration::from_secs(90)) {
        Ok(true) => Ok(()),
        Ok(false) => Err("camera permission was declined".into()),
        Err(_) => Err("camera permission timed out".into()),
    }
}

unsafe fn open_session() -> Result<MacCamera, String> {
    let device: id =
        msg_send![class!(AVCaptureDevice), defaultDeviceWithMediaType: AVMediaTypeVideo];
    if device.is_null() {
        return Err("no camera found".into());
    }

    let name = nsstring_to_string(msg_send![device, localizedName]);
    eprintln!("mirror2: AVFoundation camera {name:?} → 32BGRA");

    let mut err: id = nil;
    let input: id = msg_send![
        class!(AVCaptureDeviceInput),
        deviceInputWithDevice: device
        error: &mut err
    ];
    if input.is_null() {
        return Err("could not open camera input".into());
    }

    let (tx, rx) = sync_channel::<(u32, u32, Vec<u8>)>(1);
    let tx = Arc::new(tx);

    let delegate = make_delegate(&tx)?;
    let queue = dispatch_queue_create(c"com.newtower.mirror2.frames".as_ptr(), std::ptr::null());
    if queue.is_null() {
        return Err("could not create camera queue".into());
    }

    let output: id = msg_send![class!(AVCaptureVideoDataOutput), new];
    let _: () = msg_send![output, setAlwaysDiscardsLateVideoFrames: YES];
    let _: () = msg_send![output, setSampleBufferDelegate: delegate queue: queue];
    set_bgra_settings(output)?;

    let session: id = msg_send![class!(AVCaptureSession), new];
    let _: () = msg_send![session, beginConfiguration];
    let can_in: BOOL = msg_send![session, canAddInput: input];
    if can_in != YES {
        return Err("session rejected camera input".into());
    }
    let _: () = msg_send![session, addInput: input];
    let can_out: BOOL = msg_send![session, canAddOutput: output];
    if can_out != YES {
        return Err("session rejected camera output".into());
    }
    let _: () = msg_send![session, addOutput: output];
    let _: () = msg_send![session, commitConfiguration];
    let _: () = msg_send![session, startRunning];

    Ok(MacCamera {
        rx,
        name,
        _keep: SessionKeep {
            _tx: tx,
            session,
            _input: input,
            _output: output,
            _delegate: delegate,
            _queue: queue,
        },
    })
}

unsafe fn set_bgra_settings(output: id) -> Result<(), String> {
    let number = CFNumber::from(kCVPixelFormatType_32BGRA as i32);
    let key = kCVPixelBufferPixelFormatTypeKey as *const c_void;
    let value = number.as_concrete_TypeRef() as *const c_void;
    let dict: id = msg_send![
        class!(NSDictionary),
        dictionaryWithObject: value
        forKey: key
    ];
    if dict.is_null() {
        return Err("could not build BGRA settings".into());
    }
    let _: () = msg_send![output, setVideoSettings: dict];
    Ok(())
}

fn delegate_class() -> &'static Class {
    static CLASS: std::sync::OnceLock<&'static Class> = std::sync::OnceLock::new();
    CLASS.get_or_init(|| {
        let mut decl = ClassDecl::new("Mirror2CaptureCallback", class!(NSObject))
            .expect("register Mirror2CaptureCallback");
        decl.add_ivar::<*const c_void>("_tx");
        unsafe {
            decl.add_method(
                sel!(setTx:),
                set_tx as extern "C" fn(&mut Object, Sel, *const c_void),
            );
            decl.add_method(
                sel!(captureOutput:didOutputSampleBuffer:fromConnection:),
                on_frame as extern "C" fn(&mut Object, Sel, id, CMSampleBufferRef, id),
            );
        }
        decl.register()
    })
}

extern "C" fn set_tx(this: &mut Object, _: Sel, ptr: *const c_void) {
    unsafe {
        this.set_ivar("_tx", ptr);
    }
}

extern "C" fn on_frame(this: &mut Object, _: Sel, _: id, sample: CMSampleBufferRef, _: id) {
    unsafe {
        let pixel = CMSampleBufferGetImageBuffer(sample);
        if pixel.is_null() {
            return;
        }
        CVPixelBufferLockBaseAddress(pixel, 0);
        let base = CVPixelBufferGetBaseAddress(pixel);
        if base.is_null() {
            CVPixelBufferUnlockBaseAddress(pixel, 0);
            return;
        }
        let width = CVPixelBufferGetWidth(pixel) as u32;
        let height = CVPixelBufferGetHeight(pixel) as u32;
        let stride = CVPixelBufferGetBytesPerRow(pixel);
        if width == 0 || height == 0 {
            CVPixelBufferUnlockBaseAddress(pixel, 0);
            return;
        }

        let mut rgb = vec![0u8; width as usize * height as usize * 3];
        let src = base as *const u8;
        for y in 0..height as usize {
            let row = src.add(y * stride);
            for x in 0..width as usize {
                let b = *row.add(x * 4);
                let g = *row.add(x * 4 + 1);
                let r = *row.add(x * 4 + 2);
                let o = (y * width as usize + x) * 3;
                rgb[o] = r;
                rgb[o + 1] = g;
                rgb[o + 2] = b;
            }
        }
        CVPixelBufferUnlockBaseAddress(pixel, 0);

        let ptr: *const c_void = *this.get_ivar("_tx");
        if ptr.is_null() {
            return;
        }
        let tx = &*(ptr as *const SyncSender<(u32, u32, Vec<u8>)>);
        let _ = tx.try_send((width, height, rgb));
    }
}

unsafe fn make_delegate(tx: &Arc<SyncSender<(u32, u32, Vec<u8>)>>) -> Result<id, String> {
    let cls = delegate_class();
    let delegate: id = msg_send![cls, new];
    if delegate.is_null() {
        return Err("could not create camera delegate".into());
    }
    let ptr = Arc::as_ptr(tx) as *const c_void;
    let _: () = msg_send![delegate, setTx: ptr];
    Ok(delegate)
}

unsafe fn nsstring_to_string(value: id) -> String {
    if value.is_null() {
        return "Camera".into();
    }
    let utf8: *const c_char = msg_send![value, UTF8String];
    if utf8.is_null() {
        return "Camera".into();
    }
    std::ffi::CStr::from_ptr(utf8)
        .to_string_lossy()
        .into_owned()
}
