use std::time::{Duration, Instant};
use xcap::Monitor;

/// A frame captured from the screen.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// Raw RGBA pixels
    pub data: Vec<u8>,
    pub timestamp: Duration,
}

/// Abstract trait for screen capture to allow swapping backends later.
pub trait CaptureDevice {
    fn capture_frame(&mut self) -> Result<Frame, String>;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

/// Cross-platform capture implementation using the `xcap` crate.
/// Used for Phase 1 (PoC).
pub struct XcapCapture {
    monitor: Monitor,
    start_time: Instant,
}

impl XcapCapture {
    /// Initializes capture on the primary monitor.
    pub fn new() -> Result<Self, String> {
        let monitors = Monitor::all().map_err(|e| e.to_string())?;
        
        let monitor = monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| Monitor::all().unwrap_or_default().into_iter().next())
            .ok_or("No monitors found")?;

        Ok(Self {
            monitor,
            start_time: Instant::now(),
        })
    }
}

impl CaptureDevice for XcapCapture {
    fn capture_frame(&mut self) -> Result<Frame, String> {
        let image = self.monitor.capture_image().map_err(|e| e.to_string())?;
        
        Ok(Frame {
            width: image.width(),
            height: image.height(),
            data: image.into_raw(),
            timestamp: self.start_time.elapsed(),
        })
    }

    fn get_width(&self) -> u32 {
        self.monitor.width().unwrap_or(0)
    }

    fn get_height(&self) -> u32 {
        self.monitor.height().unwrap_or(0)
    }
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
