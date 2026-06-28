pub mod qos;
use lumina_capture::Frame;

/// Represents a compressed video packet ready to be sent over the network.
pub struct EncodedPacket {
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub timestamp_us: u64,
}

/// Abstract trait for video encoding.
pub trait VideoEncoder {
    fn encode_frame(&mut self, frame: &Frame) -> Result<Vec<EncodedPacket>, String>;
}

/// JPEG-based encoder that works on all platforms without native FFI.
/// Produces JPEG-compressed frames that are decoded on the client side.
/// This replaces the previous stub H.264 encoders with a working implementation.
pub struct SystemEncoder {
    width: u32,
    height: u32,
    quality: u8,
    frame_count: u64,
    keyframe_interval: u64,
}

impl SystemEncoder {
    pub fn new(width: u32, height: u32, _fps: i32) -> Result<Self, String> {
        println!("[Lumina Encoder] Initializing JPEG Software Encoder ({}x{})...", width, height);
        Ok(Self {
            width,
            height,
            quality: 60, // Good balance of quality vs bandwidth
            frame_count: 0,
            keyframe_interval: 1, // Every frame is a keyframe in JPEG mode
        })
    }

    /// Adjusts encoding quality (1-100). Lower = smaller files, worse quality.
    pub fn set_quality(&mut self, quality: u8) {
        self.quality = quality.clamp(10, 100);
    }
}

impl VideoEncoder for SystemEncoder {
    fn encode_frame(&mut self, frame: &Frame) -> Result<Vec<EncodedPacket>, String> {
        self.frame_count += 1;
        
        let src_width = frame.width;
        let src_height = frame.height;
        
        // Downscale if the source is larger than our target resolution
        let (out_width, out_height, pixels) = if src_width > self.width || src_height > self.height {
            let scale = f64::min(
                self.width as f64 / src_width as f64,
                self.height as f64 / src_height as f64,
            );
            let new_w = (src_width as f64 * scale) as u32;
            let new_h = (src_height as f64 * scale) as u32;
            
            // Simple nearest-neighbor downscale (fast, good enough for remote desktop)
            let mut out = Vec::with_capacity((new_w * new_h * 4) as usize);
            for y in 0..new_h {
                let src_y = (y as f64 / scale) as u32;
                for x in 0..new_w {
                    let src_x = (x as f64 / scale) as u32;
                    let idx = ((src_y * src_width + src_x) * 4) as usize;
                    if idx + 3 < frame.data.len() {
                        out.push(frame.data[idx]);     // R
                        out.push(frame.data[idx + 1]); // G
                        out.push(frame.data[idx + 2]); // B
                        out.push(frame.data[idx + 3]); // A
                    } else {
                        out.extend_from_slice(&[0, 0, 0, 255]);
                    }
                }
            }
            (new_w, new_h, out)
        } else {
            (src_width, src_height, frame.data.clone())
        };

        // Convert RGBA -> RGB for JPEG encoding
        let rgb_data: Vec<u8> = pixels.chunks_exact(4)
            .flat_map(|px| [px[0], px[1], px[2]])
            .collect();

        // Encode to JPEG using the `image` crate
        let mut jpeg_buf = Vec::new();
        {
            use std::io::Cursor;
            let mut cursor = Cursor::new(&mut jpeg_buf);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut cursor,
                self.quality,
            );
            encoder.encode(
                &rgb_data,
                out_width,
                out_height,
                image::ColorType::Rgb8.into(),
            ).map_err(|e| format!("JPEG encode error: {}", e))?;
        }

        let timestamp_us = frame.timestamp.as_micros() as u64;
        let is_keyframe = self.frame_count % self.keyframe_interval == 0;

        Ok(vec![EncodedPacket {
            data: jpeg_buf,
            is_keyframe,
            timestamp_us,
        }])
    }
}
