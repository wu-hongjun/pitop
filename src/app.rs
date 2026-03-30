use crate::board::{self, BoardProfile, BoardType, SystemInfo, VoltageSource};
use crate::collectors::cpu::{CpuCollector, CpuData};
use crate::collectors::disk::{DiskCollector, DiskData};
use crate::collectors::fan::{FanCollector, FanData};
use crate::collectors::gpu::{self, GpuData};
use crate::collectors::memory::{MemoryCollector, MemoryData};
use crate::collectors::network::{NetworkCollector, NetworkData};
use crate::collectors::pcie::{PcieCollector, PcieData};
use crate::collectors::poe::{PoeCollector, PoeData};
use crate::collectors::power::{self, PowerData};
use crate::collectors::process::ProcessCollector;
use crate::collectors::process::ProcessInfo;
use crate::collectors::thermal::{ThermalCollector, ThermalData};
use crate::collectors::throttle::ThrottleData;
use crate::config::Config;
use crate::stress::StressTest;
use crate::ui::theme::Theme;
use crate::util::ring_buffer::RingBuffer;
use crate::util::update_check::UpdateHandle;
use crate::util::vcgencmd::VcgencmdRunner;
use anyhow::Result;
use std::collections::HashMap;
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
    // Configuration
    pub config: Config,
    pub theme: Theme,
    pub theme_names: Vec<String>,
    pub theme_index: usize,

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
    pub power: PowerData,
    pub fan: FanData,
    pub pcie: PcieData,
    pub poe: PoeData,
    pub gpu: GpuData,

    // Sparkline history
    pub cpu_history: RingBuffer<f64>,
    pub mem_history: RingBuffer<f64>,
    pub temp_history: RingBuffer<f64>,
    pub power_history: RingBuffer<f64>,
    pub gpu_freq_history: RingBuffer<f64>,
    pub network_rx_history: HashMap<String, RingBuffer<f64>>,
    pub network_tx_history: HashMap<String, RingBuffer<f64>>,

    // UI state
    pub active_tab: usize,
    pub paused: bool,
    pub should_quit: bool,
    pub verbose: bool,

    // Process table state
    pub process_sort_column: usize,
    pub process_selected: usize,
    pub kill_confirm: Option<(u32, String)>,
    pub kill_result: Option<String>,
    pub show_help: bool,
    pub help_scroll: usize,
    pub stress: Option<StressTest>,
    pub update_status: Option<UpdateHandle>,

    // Collectors (owned)
    cpu_collector: CpuCollector,
    memory_collector: MemoryCollector,
    thermal_collector: ThermalCollector,
    network_collector: NetworkCollector,
    disk_collector: DiskCollector,
    process_collector: ProcessCollector,
    fan_collector: FanCollector,
    pcie_collector: PcieCollector,
    poe_collector: PoeCollector,
    vcgencmd: VcgencmdRunner,

    root: PathBuf,
}

impl App {
    pub fn new(board_type: BoardType, root: &Path, verbose: bool, config: Config) -> Self {
        let profile = board::create_profile(board_type);
        let system_info = board::collect_system_info(root);

        let history_size = config.general.history_size;

        let mut theme_names = vec![
            "default".to_string(),
            "monochrome".to_string(),
            "solarized".to_string(),
        ];
        if config.custom_theme.is_some() {
            theme_names.push("custom".to_string());
        }

        Self {
            config,
            theme: Theme::default(),
            theme_names,
            theme_index: 0,
            profile,
            system_info,
            cpu: CpuData::default(),
            memory: MemoryData::default(),
            thermal: ThermalData::default(),
            throttle: ThrottleData::default(),
            processes: Vec::new(),
            network: NetworkData::default(),
            disk: DiskData::default(),
            power: PowerData::default(),
            fan: FanData::default(),
            pcie: PcieData::default(),
            poe: PoeData::default(),
            gpu: GpuData::default(),
            cpu_history: RingBuffer::new(history_size),
            mem_history: RingBuffer::new(history_size),
            temp_history: RingBuffer::new(history_size),
            power_history: RingBuffer::new(history_size),
            gpu_freq_history: RingBuffer::new(history_size),
            network_rx_history: HashMap::new(),
            network_tx_history: HashMap::new(),
            active_tab: 0,
            paused: false,
            should_quit: false,
            verbose,
            process_sort_column: 0,
            process_selected: 0,
            kill_confirm: None, // (pid, name)
            kill_result: None,
            show_help: false,
            help_scroll: 0,
            stress: None,
            update_status: None,
            cpu_collector: CpuCollector::new(root),
            memory_collector: MemoryCollector::new(root),
            thermal_collector: ThermalCollector::new(root),
            network_collector: NetworkCollector::new(root),
            disk_collector: DiskCollector::new(root),
            process_collector: ProcessCollector::new(root),
            fan_collector: FanCollector::new(root),
            pcie_collector: PcieCollector::new(root),
            poe_collector: PoeCollector::new(root),
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

        // Fan monitoring (always-on for overview display)
        log_err(verbose, "fan", self.fan_collector.collect(&mut self.fan));

        // GPU monitoring via vcgencmd (always-on for overview display)
        {
            let mut gpu_available = false;

            // Try V3D clock first (more relevant on Pi 5), fall back to core clock
            let mut got_freq = false;
            if let Some(output) = self.vcgencmd.run(&["measure_clock", "v3d"]).await {
                if let Some(freq) = gpu::parse_clock_core(&output) {
                    self.gpu.frequency_mhz = freq;
                    gpu_available = true;
                    got_freq = true;
                }
            }
            if !got_freq {
                if let Some(output) = self.vcgencmd.run(&["measure_clock", "core"]).await {
                    if let Some(freq) = gpu::parse_clock_core(&output) {
                        self.gpu.frequency_mhz = freq;
                        gpu_available = true;
                    }
                }
            }

            if let Some(output) = self.vcgencmd.run(&["get_mem", "gpu"]).await {
                if let Some(mem) = gpu::parse_get_mem_gpu(&output) {
                    self.gpu.memory_mb = mem;
                    gpu_available = true;
                }
            }
            if let Some(output) = self.vcgencmd.run(&["measure_temp"]).await {
                if let Some(temp) = gpu::parse_measure_temp(&output) {
                    self.gpu.temperature_celsius = temp;
                    gpu_available = true;
                }
            }
            // Query V3D codec status
            let mut codecs = Vec::new();
            for codec in &["H264", "HEVC", "MJPG"] {
                if let Some(output) = self.vcgencmd.run(&["codec_enabled", codec]).await {
                    if let Some(status) = gpu::parse_codec_enabled(codec, &output) {
                        codecs.push(status);
                    }
                }
            }
            if !codecs.is_empty() {
                self.gpu.codecs = codecs;
            }

            // Pi 5 codec fixup: all codecs report "disabled" because BCM2712 uses
            // a dedicated hardware HEVC decoder, not the old VideoCore codec block.
            let is_pi5 = self.profile.board_type() == BoardType::Pi5;
            if is_pi5 {
                let all_disabled = !self.gpu.codecs.is_empty()
                    && self.gpu.codecs.iter().all(|(_, enabled)| !enabled);
                if all_disabled {
                    self.gpu.video_decoder = Some("Hardware HEVC (BCM2712)".to_string());
                    self.gpu.codecs.clear();
                }
            }

            // Pi 5 shared memory fixup: vcgencmd reports 4M because GPU uses
            // shared system memory, not a dedicated allocation.
            if is_pi5 && self.gpu.memory_mb <= 4 {
                self.gpu.shared_memory = true;
            }

            self.gpu.available = gpu_available;
        }

        // Throttle via vcgencmd
        if let Some(output) = self.vcgencmd.run(&["get_throttled"]).await {
            self.throttle = ThrottleData::from_vcgencmd_output(&output);
        }

        // Update sparkline history
        self.cpu_history.push(self.cpu.aggregate_usage_percent);
        self.mem_history.push(self.memory.usage_percent);
        self.temp_history.push(self.thermal.soc_temp_celsius);
        self.gpu_freq_history.push(self.gpu.frequency_mhz as f64);

        // Power sparkline is updated inside the PMIC branch below (tab 2)
        // to avoid recording stale values when the Power tab is not active.

        // Update per-interface network sparkline history
        let current_ifaces: std::collections::HashSet<String> = self
            .network
            .interfaces
            .iter()
            .map(|i| i.name.clone())
            .collect();
        for iface in &self.network.interfaces {
            self.network_rx_history
                .entry(iface.name.clone())
                .or_default()
                .push(iface.rx_bytes_per_sec);
            self.network_tx_history
                .entry(iface.name.clone())
                .or_default()
                .push(iface.tx_bytes_per_sec);
        }
        // Prune departed interfaces to avoid unbounded growth
        self.network_rx_history
            .retain(|k, _| current_ifaces.contains(k));
        self.network_tx_history
            .retain(|k, _| current_ifaces.contains(k));

        // Tab-dependent collectors (lazy refresh)
        match self.active_tab {
            0 => {
                // Overview shows process list, disk, and power info
                log_err(
                    verbose,
                    "process",
                    self.process_collector.collect(&mut self.processes),
                );
                log_err(verbose, "disk", self.disk_collector.collect(&mut self.disk));
                // Collect power data for overview info panel
                match self.profile.voltage_source() {
                    VoltageSource::Pmic => {
                        if let Some(output) = self.vcgencmd.run(&["pmic_read_adc"]).await {
                            self.power.pmic = Some(power::parse_pmic_read_adc(&output));
                        }
                        if let Some(ref pmic) = self.power.pmic {
                            self.power_history.push(pmic.estimated_real_watts);
                        }
                    }
                    VoltageSource::MeasureVolts => {
                        let mut voltages = Vec::new();
                        for rail in &["core", "sdram_c", "sdram_i", "sdram_p"] {
                            if let Some(output) = self.vcgencmd.run(&["measure_volts", rail]).await
                            {
                                if let Some(reading) = power::parse_measure_volts(rail, &output) {
                                    voltages.push(reading);
                                }
                            }
                        }
                        self.power.voltages = voltages;
                    }
                    VoltageSource::None => {}
                }
            }
            1 => {
                log_err(
                    verbose,
                    "process",
                    self.process_collector.collect(&mut self.processes),
                );
            }
            2 => {
                // Power tab: collect PMIC / voltage data + PCIe + PoE
                match self.profile.voltage_source() {
                    VoltageSource::Pmic => {
                        if let Some(output) = self.vcgencmd.run(&["pmic_read_adc"]).await {
                            self.power.pmic = Some(power::parse_pmic_read_adc(&output));
                        }
                        if let Some(ref pmic) = self.power.pmic {
                            self.power_history.push(pmic.estimated_real_watts);
                        }
                    }
                    VoltageSource::MeasureVolts => {
                        let mut voltages = Vec::new();
                        for rail in &["core", "sdram_c", "sdram_i", "sdram_p"] {
                            if let Some(output) = self.vcgencmd.run(&["measure_volts", rail]).await
                            {
                                if let Some(reading) = power::parse_measure_volts(rail, &output) {
                                    voltages.push(reading);
                                }
                            }
                        }
                        self.power.voltages = voltages;
                    }
                    VoltageSource::None => {}
                }
                log_err(verbose, "pcie", self.pcie_collector.collect(&mut self.pcie));
                log_err(verbose, "poe", self.poe_collector.collect(&mut self.poe));
            }
            3 => { /* network detail: already collected */ }
            4 => {
                log_err(verbose, "disk", self.disk_collector.collect(&mut self.disk));
            }
            5 => { /* system info: static, no refresh */ }
            _ => {}
        }
    }

    /// Return processes sorted according to the current sort column.
    /// Both the UI and event handler use this to ensure consistent indexing.
    pub fn sorted_processes(&self) -> Vec<ProcessInfo> {
        let mut sorted = self.processes.clone();
        match self.process_sort_column {
            0 => sorted.sort_by_key(|p| p.pid),
            1 => sorted.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
            2 => sorted.sort_by(|a, b| {
                b.cpu_percent
                    .partial_cmp(&a.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            3 => sorted.sort_by(|a, b| b.rss_bytes.cmp(&a.rss_bytes)),
            4 => sorted.sort_by(|a, b| a.user.cmp(&b.user)),
            _ => {}
        }
        sorted
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

    /// Advance to the next theme in the cycle.
    pub fn cycle_theme(&mut self) {
        if self.theme_names.is_empty() {
            return;
        }
        self.theme_index = (self.theme_index + 1) % self.theme_names.len();
        let name = &self.theme_names[self.theme_index];
        if name == "custom" {
            if let Some(ref ct) = self.config.custom_theme {
                self.theme = Theme::from_config(ct);
            }
        } else {
            self.theme = Theme::from_name(name).unwrap_or_default();
        }
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
