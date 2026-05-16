use crate::{
    settings::{FpsLimit, SpeedSource},
    MetricsSnapshot,
};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct AnimationPlan {
    pub interval: Duration,
    pub load: f32,
}

impl AnimationPlan {
    pub fn from_snapshot(
        snapshot: &MetricsSnapshot,
        speed_source: SpeedSource,
        fps_limit: FpsLimit,
    ) -> Self {
        let load = match speed_source {
            SpeedSource::Cpu => snapshot.cpu_total,
            SpeedSource::Memory => snapshot.memory_load,
        }
        .clamp(0.0, 100.0);

        let speed = ((load / 5.0) * fps_limit.rate()).max(1.0);
        let interval_ms = (500.0 / speed).round().clamp(25.0, 500.0) as u64;

        Self {
            interval: Duration::from_millis(interval_ms),
            load,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AnimationClock {
    current_frame: usize,
}

impl AnimationClock {
    pub fn next_frame(&mut self, frame_count: usize) -> usize {
        if frame_count == 0 {
            self.current_frame = 0;
            return 0;
        }

        let frame = self.current_frame % frame_count;
        self.current_frame = (self.current_frame + 1) % frame_count;
        frame
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{settings::FpsLimit, MetricsSnapshot};

    #[test]
    fn keeps_idle_animation_at_the_slowest_interval() {
        let snapshot = MetricsSnapshot {
            cpu_total: 0.0,
            ..MetricsSnapshot::default()
        };

        let plan = AnimationPlan::from_snapshot(&snapshot, SpeedSource::Cpu, FpsLimit::Fps40);

        assert_eq!(plan.interval, Duration::from_millis(500));
        assert_eq!(plan.load, 0.0);
    }

    #[test]
    fn increases_animation_speed_with_load() {
        let snapshot = MetricsSnapshot {
            cpu_total: 50.0,
            ..MetricsSnapshot::default()
        };

        let plan = AnimationPlan::from_snapshot(&snapshot, SpeedSource::Cpu, FpsLimit::Fps40);

        assert_eq!(plan.interval, Duration::from_millis(50));
        assert_eq!(plan.load, 50.0);
    }

    #[test]
    fn wraps_frame_indices() {
        let mut clock = AnimationClock::default();

        assert_eq!(clock.next_frame(3), 0);
        assert_eq!(clock.next_frame(3), 1);
        assert_eq!(clock.next_frame(3), 2);
        assert_eq!(clock.next_frame(3), 0);
    }
}
