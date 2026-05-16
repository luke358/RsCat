use std::collections::VecDeque;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

const SAMPLE_WINDOW: usize = 5;

#[derive(Clone, Copy, Debug, Default)]
pub struct MetricsSnapshot {
    pub cpu_total: f32,
    pub memory_load: f32,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_available_bytes: u64,
}

impl MetricsSnapshot {
    pub fn tooltip(self) -> String {
        format!(
            "CPU: {:.1}%\nMemory: {:.1}% ({}/{})",
            self.cpu_total,
            self.memory_load,
            format_bytes(self.memory_used_bytes),
            format_bytes(self.memory_total_bytes),
        )
    }
}

#[derive(Debug)]
pub struct MetricSampler {
    system: System,
    samples: VecDeque<MetricsSnapshot>,
}

impl MetricSampler {
    pub fn new() -> Self {
        let refresh = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        let mut sampler = Self {
            system: System::new_with_specifics(refresh),
            samples: VecDeque::with_capacity(SAMPLE_WINDOW),
        };
        sampler.sample();
        sampler
    }

    pub fn sample(&mut self) -> MetricsSnapshot {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();

        let total = self.system.total_memory().saturating_mul(1024);
        let available = self.system.available_memory().saturating_mul(1024);
        let used = total.saturating_sub(available);
        let memory_load = if total == 0 {
            0.0
        } else {
            (used as f32 / total as f32) * 100.0
        };

        let snapshot = MetricsSnapshot {
            cpu_total: self.system.global_cpu_usage(),
            memory_load,
            memory_total_bytes: total,
            memory_used_bytes: used,
            memory_available_bytes: available,
        };

        self.samples.push_back(snapshot);
        while self.samples.len() > SAMPLE_WINDOW {
            self.samples.pop_front();
        }

        self.average()
    }

    pub fn average(&self) -> MetricsSnapshot {
        if self.samples.is_empty() {
            return MetricsSnapshot::default();
        }

        let len = self.samples.len() as f32;
        let mut result = MetricsSnapshot::default();

        for sample in &self.samples {
            result.cpu_total += sample.cpu_total / len;
            result.memory_load += sample.memory_load / len;
            result.memory_total_bytes = sample.memory_total_bytes;
            result.memory_used_bytes += sample.memory_used_bytes / self.samples.len() as u64;
            result.memory_available_bytes +=
                sample.memory_available_bytes / self.samples.len() as u64;
        }

        result
    }
}

impl Default for MetricSampler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_with_binary_units() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
    }
}
