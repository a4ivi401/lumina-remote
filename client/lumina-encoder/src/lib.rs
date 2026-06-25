pub mod qos;
use lumina_capture::Frame;

/// Represents a compressed video packet ready to be sent over QUIC.
pub struct EncodedPacket {
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub timestamp_us: u64,
}

/// Abstract trait for video encoding to allow hot-swapping encoders (FFmpeg, software, etc).
pub trait VideoEncoder {
    /// Encodes a raw RGBA frame into one or more compressed packets (e.g. H.264 NAL units).
    fn encode_frame(&mut self, frame: &Frame) -> Result<Vec<EncodedPacket>, String>;
}

/// Hardware-accelerated encoder using FFmpeg (H.264 / AV1).
/// 
/// Note: Full FFmpeg integration requires `libavcodec-dev` system dependencies.
/// This is the skeleton implementation for the architectural flow.
pub struct FFmpegEncoder {
    width: u32,
    height: u32,
    // Future fields: AVCodecContext, SwsContext, AVFrame (raw), AVFrame (yuv), AVPacket
}

impl FFmpegEncoder {
    pub fn new(width: u32, height: u32, _fps: i32) -> Result<Self, String> {
        // To use `ffmpeg-next`:
        // ffmpeg_next::init().map_err(|e| e.to_string())?;
        // let encoder = ffmpeg_next::encoder::find(ffmpeg_next::codec::Id::H264)...
        
        Ok(Self {
            width,
            height,
        })
    }
}

impl VideoEncoder for FFmpegEncoder {
    fn encode_frame(&mut self, frame: &Frame) -> Result<Vec<EncodedPacket>, String> {
        // Stub for the PoC. 
        // Real implementation involves:
        // 1. Copying frame.data into an AVFrame (RGBA)
        // 2. SwsScale to convert RGBA -> YUV420P
        // 3. avcodec_send_frame -> avcodec_receive_packet
        // 4. Wrapping the result in EncodedPacket.
        
        Ok(vec![EncodedPacket {
            data: vec![], // Empty payload for now
            is_keyframe: true,
            timestamp_us: frame.timestamp.as_micros() as u64,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_encoder_interface() {
        let mut encoder = FFmpegEncoder::new(1920, 1080, 60).unwrap();
        let dummy_frame = Frame {
            width: 1920,
            height: 1080,
            data: vec![0; 1920 * 1080 * 4],
            timestamp: Duration::from_millis(16),
        };
        
        let packets = encoder.encode_frame(&dummy_frame).unwrap();
        assert_eq!(packets.len(), 1);
        assert!(packets[0].is_keyframe);
    }
}
