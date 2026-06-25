pub mod diff;
pub mod frame;

pub mod mac_capture;
pub mod win_capture;
pub mod linux_capture;
pub mod xcap_capture;

pub use frame::Frame;

/// Abstract trait for screen capture to allow swapping backends later.
pub trait CaptureDevice {
    fn capture_frame(&mut self) -> Result<Frame, String>;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

pub fn create_capture_device() -> Result<Box<dyn CaptureDevice>, String> {
    #[cfg(target_os = "macos")]
    {
        // Try native SCKit, fallback to Xcap
        if let Ok(c) = mac_capture::MacCapture::new() {
            return Ok(Box::new(c) as Box<dyn CaptureDevice>);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try native DXGI, fallback to Xcap
        if let Ok(c) = win_capture::WinCapture::new() {
            return Ok(Box::new(c) as Box<dyn CaptureDevice>);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // For Linux, LinuxCapture wraps Xcap
        if let Ok(c) = linux_capture::LinuxCapture::new() {
            return Ok(Box::new(c) as Box<dyn CaptureDevice>);
        }
    }

    // Universal Fallback
    xcap_capture::XcapCapture::new().map(|c| Box::new(c) as Box<dyn CaptureDevice>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_init() {
        let capturer = XcapCapture::new();
        // The test might fail in CI or headless environments if there's no display server,
        // but on a real desktop it should pass.
        if let Ok(mut c) = capturer {
            assert!(c.get_width() > 0);
            assert!(c.get_height() > 0);
            
            let frame = c.capture_frame().unwrap();
            assert!(frame.width > 0);
            assert!(frame.height > 0);
            assert!(!frame.data.is_empty());
            
            // RGBA image means width * height * 4 bytes
            assert_eq!(frame.data.len() as u32, frame.width * frame.height * 4);
        }
    }
}
