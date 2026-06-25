use lumina_protocol::QosMetrics;

#[derive(Clone, Debug)]
pub struct EncoderConfig {
    pub bitrate_kbps: u32,
    pub fps: u32,
}

/// The Quality of Service (QoS) Manager dynamically adjusts encoder 
/// settings based on network conditions to ensure low latency.
pub struct QosManager {
    pub current_config: EncoderConfig,
}

impl QosManager {
    pub fn new(initial_bitrate: u32, initial_fps: u32) -> Self {
        Self {
            current_config: EncoderConfig {
                bitrate_kbps: initial_bitrate,
                fps: initial_fps,
            },
        }
    }

    /// Processes incoming QoS metrics from the Client.
    /// Returns `Some(EncoderConfig)` if the settings need to be updated.
    pub fn process_metrics(&mut self, metrics: &QosMetrics) -> Option<EncoderConfig> {
        let mut changed = false;

        // 1. If RTT is high (bad network), aggressively reduce bitrate
        if metrics.rtt_ms > 150 {
            self.current_config.bitrate_kbps = (self.current_config.bitrate_kbps as f32 * 0.7) as u32;
            // Don't drop below 500 kbps to maintain some visual clarity
            self.current_config.bitrate_kbps = self.current_config.bitrate_kbps.max(500);
            changed = true;
        }

        // 2. If the client is dropping frames, it might be overwhelmed. Reduce FPS.
        if metrics.dropped_frames > 5 {
            self.current_config.fps = self.current_config.fps.saturating_sub(5).max(15);
            changed = true;
        }

        // 3. If network is perfect (low ping, zero drops), slowly increase quality
        if metrics.rtt_ms < 40 && metrics.dropped_frames == 0 {
            // Increase by 5%
            self.current_config.bitrate_kbps = (self.current_config.bitrate_kbps as f32 * 1.05) as u32;
            // Cap at 20 Mbps (20,000 kbps)
            self.current_config.bitrate_kbps = self.current_config.bitrate_kbps.min(20_000);
            
            if self.current_config.fps < 60 {
                self.current_config.fps += 1; // Slowly restore FPS
            }
            changed = true;
        }

        if changed {
            Some(self.current_config.clone())
        } else {
            None
        }
    }
}
