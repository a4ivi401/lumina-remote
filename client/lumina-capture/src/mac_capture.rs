use crate::frame::Frame;
use crate::CaptureDevice;
use std::time::Instant;

#[cfg(target_os = "macos")]
use crate::xcap_capture::XcapCapture;

#[cfg(target_os = "macos")]
pub struct MacCapture {
    inner: XcapCapture,
}

#[cfg(target_os = "macos")]
impl MacCapture {
    pub fn new() -> Result<Self, String> {
        let inner = XcapCapture::new()?;
        Ok(Self { inner })
    }
}

#[cfg(target_os = "macos")]
impl CaptureDevice for MacCapture {
    fn capture_frame(&mut self) -> Result<Frame, String> {
        self.inner.capture_frame()
    }

    fn get_width(&self) -> u32 {
        self.inner.get_width()
    }

    fn get_height(&self) -> u32 {
        self.inner.get_height()
    }
}

#[cfg(not(target_os = "macos"))]
pub struct MacCapture {}

#[cfg(not(target_os = "macos"))]
impl MacCapture {
    pub fn new() -> Result<Self, String> {
        Err("MacCapture only supported on macOS".into())
    }
}

#[cfg(not(target_os = "macos"))]
impl CaptureDevice for MacCapture {
    fn capture_frame(&mut self) -> Result<Frame, String> { unimplemented!() }
    fn get_width(&self) -> u32 { 0 }
    fn get_height(&self) -> u32 { 0 }
}
