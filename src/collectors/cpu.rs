use crate::util::sysfs::{read_sysfs_string, read_sysfs_u64};
use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize)]
pub struct CoreUsage {
    pub core_id: usize,
    pub usage_percent: f64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CpuData {
    pub cores: Vec<CoreUsage>,
    pub aggregate_usage_percent: f64,
    pub frequency_khz: u64,
    pub min_frequency_khz: u64,
    pub max_frequency_khz: u64,
    pub governor: String,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
}

/// Raw CPU jiffies from a single /proc/stat line.
#[derive(Debug, Clone, Default)]
struct CpuSample {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl CpuSample {
    fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }

    fn busy(&self) -> u64 {
        self.user + self.nice + self.system + self.irq + self.softirq + self.steal
    }
}

pub struct CpuCollector {
    root: PathBuf,
    prev_samples: Vec<CpuSample>,
    prev_aggregate: Option<CpuSample>,
}

impl CpuCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            prev_samples: Vec::new(),
            prev_aggregate: None,
        }
    }

    pub fn collect(&mut self, data: &mut CpuData) -> Result<()> {
        let stat_path = self.root.join("proc/stat");
        let content = std::fs::read_to_string(&stat_path).unwrap_or_default();

        let mut aggregate = CpuSample::default();
        let mut per_core: Vec<(usize, CpuSample)> = Vec::new();

        for line in content.lines() {
            if line.starts_with("cpu ") {
                aggregate = parse_cpu_line(line);
            } else if line.starts_with("cpu") {
                if let Some(core_id) = line
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.strip_prefix("cpu"))
                    .and_then(|s| s.parse::<usize>().ok())
                {
                    per_core.push((core_id, parse_cpu_line(line)));
                }
            }
        }

        // Compute aggregate usage from delta (0% on first call)
        data.aggregate_usage_percent = match &self.prev_aggregate {
            Some(prev) => compute_usage(prev, &aggregate),
            None => 0.0,
        };

        // Compute per-core usage from deltas
        data.cores.clear();
        for (core_id, sample) in &per_core {
            let usage = if self.prev_aggregate.is_some() {
                let prev = self.prev_samples.get(*core_id).cloned().unwrap_or_default();
                compute_usage(&prev, sample)
            } else {
                0.0
            };
            data.cores.push(CoreUsage {
                core_id: *core_id,
                usage_percent: usage,
            });
        }

        // Store current samples for next delta
        self.prev_aggregate = Some(aggregate);
        self.prev_samples = per_core.into_iter().map(|(_, s)| s).collect();

        // Read frequency info
        let cpufreq = self.root.join("sys/devices/system/cpu/cpufreq/policy0");
        data.frequency_khz = read_sysfs_u64(&cpufreq.join("scaling_cur_freq")).unwrap_or(0);
        data.min_frequency_khz = read_sysfs_u64(&cpufreq.join("scaling_min_freq")).unwrap_or(0);
        data.max_frequency_khz = read_sysfs_u64(&cpufreq.join("scaling_max_freq")).unwrap_or(0);
        data.governor = read_sysfs_string(&cpufreq.join("scaling_governor")).unwrap_or_default();

        // Read load average
        let loadavg_path = self.root.join("proc/loadavg");
        if let Ok(content) = std::fs::read_to_string(&loadavg_path) {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 3 {
                data.load_avg_1 = parts[0].parse().unwrap_or(0.0);
                data.load_avg_5 = parts[1].parse().unwrap_or(0.0);
                data.load_avg_15 = parts[2].parse().unwrap_or(0.0);
            }
        }

        Ok(())
    }
}

fn parse_cpu_line(line: &str) -> CpuSample {
    let parts: Vec<u64> = line
        .split_whitespace()
        .skip(1) // skip "cpu" or "cpuN"
        .filter_map(|s| s.parse().ok())
        .collect();

    CpuSample {
        user: parts.first().copied().unwrap_or(0),
        nice: parts.get(1).copied().unwrap_or(0),
        system: parts.get(2).copied().unwrap_or(0),
        idle: parts.get(3).copied().unwrap_or(0),
        iowait: parts.get(4).copied().unwrap_or(0),
        irq: parts.get(5).copied().unwrap_or(0),
        softirq: parts.get(6).copied().unwrap_or(0),
        steal: parts.get(7).copied().unwrap_or(0),
    }
}

fn compute_usage(prev: &CpuSample, curr: &CpuSample) -> f64 {
    let total_delta = curr.total().saturating_sub(prev.total());
    let busy_delta = curr.busy().saturating_sub(prev.busy());

    if total_delta == 0 {
        0.0
    } else {
        (busy_delta as f64 / total_delta as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_fixture(dir: &Path, relative: &str, content: &str) {
        let path = dir.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
    }

    const STAT_SAMPLE1: &str = "\
cpu  10132153 290696 3084719 46828483 16683 0 25195 0 0 0
cpu0 2503850 72399 771178 11714261 4163 0 6313 0 0 0
cpu1 2531410 72875 770550 11714666 4178 0 6316 0 0 0
cpu2 2534755 72623 771183 11714234 4170 0 6283 0 0 0
cpu3 2562138 72799 771808 11685322 4172 0 6283 0 0 0
";

    const STAT_SAMPLE2: &str = "\
cpu  10232153 290696 3184719 47828483 16683 0 25195 0 0 0
cpu0 2603850 72399 796178 11764261 4163 0 6313 0 0 0
cpu1 2603410 72875 795550 11764666 4178 0 6316 0 0 0
cpu2 2634755 72623 796183 11764234 4170 0 6283 0 0 0
cpu3 2662138 72799 796808 11735322 4172 0 6283 0 0 0
";

    #[test]
    fn cpu_first_sample_returns_zero() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "proc/stat", STAT_SAMPLE1);
        write_fixture(tmp.path(), "proc/loadavg", "0.50 0.75 0.80 1/200 12345\n");

        let mut collector = CpuCollector::new(tmp.path());
        let mut data = CpuData::default();
        collector.collect(&mut data).unwrap();

        // First sample: no previous data, so 0%
        assert_eq!(data.aggregate_usage_percent, 0.0);
        assert_eq!(data.cores.len(), 4);

        // Load average should be read
        assert!((data.load_avg_1 - 0.50).abs() < 0.01);
        assert!((data.load_avg_5 - 0.75).abs() < 0.01);
    }

    #[test]
    fn cpu_delta_computes_usage() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "proc/loadavg", "0.50 0.75 0.80 1/200 12345\n");

        // First sample
        write_fixture(tmp.path(), "proc/stat", STAT_SAMPLE1);
        let mut collector = CpuCollector::new(tmp.path());
        let mut data = CpuData::default();
        collector.collect(&mut data).unwrap();

        // Second sample — should compute real deltas
        write_fixture(tmp.path(), "proc/stat", STAT_SAMPLE2);
        collector.collect(&mut data).unwrap();

        assert!(data.aggregate_usage_percent > 0.0);
        assert!(data.aggregate_usage_percent < 100.0);
        assert_eq!(data.cores.len(), 4);
        for core in &data.cores {
            assert!(core.usage_percent >= 0.0);
            assert!(core.usage_percent <= 100.0);
        }
    }

    #[test]
    fn cpu_handles_missing_stat() {
        let tmp = TempDir::new().unwrap();
        let mut collector = CpuCollector::new(tmp.path());
        let mut data = CpuData::default();
        // Should not panic — returns default data
        collector.collect(&mut data).unwrap();
        assert_eq!(data.aggregate_usage_percent, 0.0);
    }
}
