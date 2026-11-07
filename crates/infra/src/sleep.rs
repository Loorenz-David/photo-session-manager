//! The macOS idle-sleep assertion held while a USB camera session is open (10E MC-31 R-I23: a Mac
//! sleep while Capture One runs loses the camera, V6). `kIOPMAssertionTypePreventUserIdleSystemSleep`
//! is a `CFSTR` macro, not a linkable symbol, so its CFString is built here (P48 L16).
use psm_core::{Error, Result, ports::system::SleepInhibitor};
use std::sync::Mutex;

#[cfg(target_os = "macos")]
mod iokit {
    use std::ffi::{CString, c_char, c_void};

    pub type CFStringRef = *const c_void;
    pub type IOPMAssertionID = u32;
    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    pub const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;
    pub const K_IO_RETURN_SUCCESS: i32 = 0;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const c_char,
            encoding: u32,
        ) -> CFStringRef;
        fn CFRelease(cf: *const c_void);
    }
    #[link(name = "IOKit", kind = "framework")]
    unsafe extern "C" {
        fn IOPMAssertionCreateWithName(
            assertion_type: CFStringRef,
            level: u32,
            name: CFStringRef,
            id: *mut IOPMAssertionID,
        ) -> i32;
        fn IOPMAssertionRelease(id: IOPMAssertionID) -> i32;
    }

    /// An owned CFString, released on drop.
    struct CfString(CFStringRef);
    impl CfString {
        fn new(text: &str) -> Option<Self> {
            let c = CString::new(text).ok()?;
            // SAFETY: `c` is a valid NUL-terminated UTF-8 string for the duration of the call.
            let s = unsafe {
                CFStringCreateWithCString(std::ptr::null(), c.as_ptr(), K_CF_STRING_ENCODING_UTF8)
            };
            (!s.is_null()).then_some(Self(s))
        }
    }
    impl Drop for CfString {
        fn drop(&mut self) {
            // SAFETY: created by CFStringCreateWithCString (owned, +1).
            unsafe { CFRelease(self.0) }
        }
    }

    pub fn create(reason: &str) -> Result<IOPMAssertionID, String> {
        let kind = CfString::new("PreventUserIdleSystemSleep").ok_or("assertion type")?;
        let name = CfString::new(reason).ok_or("assertion name")?;
        let mut id: IOPMAssertionID = 0;
        // SAFETY: both CFStrings live until after the call; `id` is a valid out pointer.
        let rc = unsafe {
            IOPMAssertionCreateWithName(kind.0, K_IOPM_ASSERTION_LEVEL_ON, name.0, &mut id)
        };
        if rc == K_IO_RETURN_SUCCESS {
            Ok(id)
        } else {
            Err(format!("IOPMAssertionCreateWithName returned {rc:#x}"))
        }
    }
    pub fn release(id: IOPMAssertionID) {
        // SAFETY: `id` came from a successful IOPMAssertionCreateWithName.
        unsafe {
            IOPMAssertionRelease(id);
        }
    }
}

/// One `PreventUserIdleSystemSleep` assertion at a time: `hold` while held and `release` while
/// not held change nothing.
#[derive(Default)]
pub struct IokitSleepInhibitor {
    held: Mutex<Option<u32>>,
}

impl SleepInhibitor for IokitSleepInhibitor {
    fn hold(&self, reason: &str) -> Result<()> {
        let mut held = self.held.lock().unwrap_or_else(|p| p.into_inner());
        if held.is_some() {
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            *held = Some(iokit::create(reason).map_err(Error::Provider)?);
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = reason;
            Err(Error::Provider("idle-sleep assertion needs macOS".into()))
        }
    }
    fn release(&self) {
        let mut held = self.held.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(_id) = held.take() {
            #[cfg(target_os = "macos")]
            iokit::release(_id);
        }
    }
}

impl Drop for IokitSleepInhibitor {
    fn drop(&mut self) {
        self.release();
    }
}
