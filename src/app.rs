use crate::board::{self, BoardProfile, BoardType, SystemInfo};
use crate::collectors::cpu::{CpuCollector, CpuData};
use crate::collectors::disk::{DiskCollector, DiskData};
use crate::collectors::memory::{MemoryCollector, MemoryData};
use crate::collectors::network::{NetworkCollector, NetworkData};
use crate::collectors::process::ProcessCollector;
use crate::collectors::process::ProcessInfo;
use crate::collectors::thermal::{ThermalCollector, ThermalData};
use crate::collectors::throttle::ThrottleData;
use crate::util::ring_buffer::RingBuffer;
use crate::util::vcgencmd::VcgencmdRunner;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub const TAB_COUNT: usize = 6;
pub const TAB_NAMES: [&str; TAB_COUNT] = [
    "Overview",
    "Processes",
    "Power",
    "Network",
    "Disk",
    "System",
];

pub struct App {
    // Board
    pub profile: Box<dyn BoardProfile>,
    pub system_info: SystemInfo,

    // Collector data
    pub cpu: CpuData,
    pub memory: MemoryData,
    pub thermal: ThermalData,
    pub throttle: ThrottleData,
    pub processes: Vec<ProcessInfo>,
    pub network: NetworkData,
    pub disk: DiskData,

    // Sparkline history
    pub cpu_history: RingBuffer<f64>,
    pub mem_history: RingBuffer<f64>,
    pub temp_history: RingBuffer<f64>,

    // UI state
    pub active_tab: usize,
    pub paused: bool,
    pub should_quit: bool,
    pub verbose: bool,

    // Process table state
    pub process_sort_column: usize,
    pub process_selected: usize,

    // Collectors (owned)
    cpu_collector: CpuCollector,
    memory_collector: MemoryCollector,
    thermal_collector: ThermalCollector,
    network_collector: NetworkCollector,
    disk_collector: DiskCollector,
    process_collector: ProcessCollector,
    vcgencmd: VcgencmdRunner,

    root: PathBuf,
}

impl App {
    pub fn new(board_type: BoardType, root: &Path, verbose: bool) -> Self {
        let profile = board::create_profile(board_type);
        let system_info = board::collect_system_info(root);

        Self {
            profile,
            system_info,
            cpu: CpuData::default(),
            memory: MemoryData::default(),
            thermal: ThermalData::default(),
            throttle: ThrottleData::default(),
            processes: Vec::new(),
            network: NetworkData::default(),
            disk: DiskData::default(),
            cpu_history: RingBuffer::default(),
            mem_history: RingBuffer::default(),
            temp_history: RingBuffer::default(),
            active_tab: 0,
            paused: false,
            should_quit: false,
            verbose,
            process_sort_column: 0,
            process_selected: 0,
            cpu_collector: CpuCollector::new(root),
            memory_collector: MemoryCollector::new(root),
            thermal_collector: ThermalCollector::new(root),
            network_collector: NetworkCollector::new(root),
            disk_collector: DiskCollector::new(root),
            process_collector: ProcessCollector::new(root),
            vcgencmd: VcgencmdRunner::new(),
            root: root.to_path_buf(),
        }
    }

    /// Run all always-on collectors (overview data) + tab-dependent collectors.
    pub async fn tick(&mut self) {
        if self.paused {
            return;
        }

        let verbose = self.verbose;

        // Always-on collectors (overview tab data)
        log_err(verbose, "cpu", self.cpu_collector.collect(&mut self.cpu));
        log_err(
            verbose,
            "memory",
            self.memory_collector.collect(&mut self.memory),
        );
        log_err(
            verbose,
            "thermal",
            self.thermal_collector.collect(&mut self.thermal),
        );
        log_err(
            verbose,
            "network",
            self.network_collector.collect(&mut self.network),
        );

        // Throttle via vcgencmd
        if let Some(output) = self.vcgencmd.run(&["get_throttled"]).await {
            self.throttle = ThrottleData::from_vcgencmd_output(&output);
        }

        // Update sparkline history
        self.cpu_history.push(self.cpu.aggregate_usage_percent);
        self.mem_history.push(self.memory.usage_percent);
        self.temp_history.push(self.thermal.soc_temp_celsius);

        // Tab-dependent collectors (lazy refresh)
        match self.active_tab {
            0 => { /* overview: covered by always-on */ }
            1 => {
                log_err(
                    verbose,
                    "process",
                    self.process_collector.collect(&mut self.processes),
                );
            }
            2 => { /* power: TODO in Epic 5 */ }
            3 => { /* network detail: already collected */ }
            4 => {
                log_err(verbose, "disk", self.disk_collector.collect(&mut self.disk));
            }
            5 => { /* system info: static, no refresh */ }
            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % TAB_COUNT;
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = if self.active_tab == 0 {
            TAB_COUNT - 1
        } else {
            self.active_tab - 1
        };
    }

    pub fn set_tab(&mut self, tab: usize) {
        if tab < TAB_COUNT {
            self.active_tab = tab;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Read uptime from /proc/uptime.
    pub fn uptime_seconds(&self) -> u64 {
        std::fs::read_to_string(self.root.join("proc/uptime"))
            .ok()
            .and_then(|s| s.split_whitespace().next().map(String::from))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| f as u64)
            .unwrap_or(0)
    }
}

fn log_err(verbose: bool, name: &str, result: Result<()>) {
    if let Err(e) = result {
        if verbose {
            eprintln!("pitop: {} collector error: {}", name, e);
        }
    }
}
