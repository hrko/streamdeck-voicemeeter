use std::time::Instant;

/// The Go meter used immediate attack and a 12 dB/second falling peak.
/// Keep elapsed time, rather than frame count, so missed frames do not slow decay.
pub struct PeakMeter {
    levels: [f32; 2],
    updated: Instant,
}

impl PeakMeter {
    pub fn new(now: Instant, levels: [f32; 2]) -> Self {
        Self {
            levels: levels.map(finite_level),
            updated: now,
        }
    }

    pub fn update(&mut self, now: Instant, levels: [f32; 2]) -> [f32; 2] {
        let decay = 12.0 * now.saturating_duration_since(self.updated).as_secs_f32();
        for (peak, current) in self.levels.iter_mut().zip(levels) {
            // Unlike the old implementation, never decay below the current signal.
            *peak = (*peak - decay).max(finite_level(current));
        }
        self.updated = now;
        self.levels
    }
}

fn finite_level(level: f32) -> f32 {
    if level.is_finite() {
        level.max(-120.0)
    } else {
        -120.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn stereo_attack_decay_and_current_floor() {
        let now = Instant::now();
        let mut meter = PeakMeter::new(now, [-30.0, -20.0]);
        assert_eq!(meter.update(now, [0.0, -10.0]), [0.0, -10.0]);
        assert_eq!(
            meter.update(now + Duration::from_millis(500), [-60.0, -12.0]),
            [-6.0, -12.0]
        );
        assert_eq!(
            meter.update(now + Duration::from_secs(2), [-60.0, -60.0]),
            [-24.0, -30.0]
        );
    }

    #[test]
    fn decay_is_independent_of_frame_rate_and_handles_silence() {
        let now = Instant::now();
        let mut regular = PeakMeter::new(now, [0.0; 2]);
        let mut delayed = PeakMeter::new(now, [0.0; 2]);
        let mut result = [0.0; 2];
        for frame in 1..=10 {
            result = regular.update(now + Duration::from_millis(frame * 100), [-120.0; 2]);
        }
        let skipped = delayed.update(now + Duration::from_secs(1), [-120.0; 2]);
        assert!((result[0] - skipped[0]).abs() < 0.0001);
        assert_eq!(
            delayed.update(now + Duration::from_secs(20), [f32::NAN, f32::INFINITY]),
            [-120.0; 2]
        );
    }
}
