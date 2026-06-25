use crate::frame::Frame;
use crate::CaptureDevice;
use xcap::Monitor;
use std::time::Instant;

pub struct XcapCapture {
    monitor: Monitor,
    start_time: Instant,
}

impl XcapCapture {
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
