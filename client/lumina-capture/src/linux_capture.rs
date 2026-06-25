use crate::frame::Frame;
use crate::CaptureDevice;
use crate::xcap_capture::XcapCapture;

// For now, on Linux we wrap Xcap since native Wayland/Pipewire is highly complex
// and Xcap handles X11 natively. We can expand this to true PipeWire later.
pub struct LinuxCapture {
    inner: XcapCapture,
}

impl LinuxCapture {
    pub fn new() -> Result<Self, String> {
        let inner = XcapCapture::new()?;
        Ok(Self { inner })
    }
}

impl CaptureDevice for LinuxCapture {
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
