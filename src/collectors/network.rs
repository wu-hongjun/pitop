use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Default, Clone, Serialize)]
pub struct InterfaceData {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct NetworkData {
    pub interfaces: Vec<InterfaceData>,
    pub total_rx_bytes_per_sec: f64,
    pub total_tx_bytes_per_sec: f64,
}

struct PrevSample {
    rx_bytes: u64,
    tx_bytes: u64,
    timestamp: Instant,
}

pub struct NetworkCollector {
    root: PathBuf,
    prev: HashMap<String, PrevSample>,
}

impl NetworkCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            prev: HashMap::new(),
        }
    }

    pub fn collect(&mut self, data: &mut NetworkData) -> Result<()> {
        let net_dev_path = self.root.join("proc/net/dev");
        let content = std::fs::read_to_string(&net_dev_path).unwrap_or_default();
        let now = Instant::now();

        data.interfaces.clear();
        data.total_rx_bytes_per_sec = 0.0;
        data.total_tx_bytes_per_sec = 0.0;

        // Skip first two header lines
        for line in content.lines().skip(2) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((name, rx_bytes, tx_bytes)) = parse_net_dev_line(line) {
                // Skip loopback
                if name == "lo" {
                    continue;
                }

                let (rx_rate, tx_rate) = if let Some(prev) = self.prev.get(&name) {
                    let elapsed = now.duration_since(prev.timestamp).as_secs_f64();
                    if elapsed > 0.0 {
                        let rx = rx_bytes.saturating_sub(prev.rx_bytes) as f64 / elapsed;
                        let tx = tx_bytes.saturating_sub(prev.tx_bytes) as f64 / elapsed;
                        (rx, tx)
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };

                data.total_rx_bytes_per_sec += rx_rate;
                data.total_tx_bytes_per_sec += tx_rate;

                data.interfaces.push(InterfaceData {
                    name: name.clone(),
                    rx_bytes,
                    tx_bytes,
                    rx_bytes_per_sec: rx_rate,
                    tx_bytes_per_sec: tx_rate,
                });

                self.prev.insert(
                    name,
                    PrevSample {
                        rx_bytes,
                        tx_bytes,
                        timestamp: now,
                    },
                );
            }
        }

        Ok(())
    }
}

/// Parse a line from /proc/net/dev.
/// Format: "  eth0: 12345 ... 67890 ..."
/// Returns (name, rx_bytes, tx_bytes)
fn parse_net_dev_line(line: &str) -> Option<(String, u64, u64)> {
    let mut parts = line.splitn(2, ':');
    let name = parts.next()?.trim().to_string();
    let rest = parts.next()?.trim();
    let values: Vec<u64> = rest
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    // Field 0 = rx_bytes, field 8 = tx_bytes
    let rx_bytes = values.first().copied()?;
    let tx_bytes = values.get(8).copied()?;
    Some((name, rx_bytes, tx_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const NET_DEV: &str = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 1234567     890    0    0    0     0          0         0  1234567     890    0    0    0     0       0          0
  eth0: 98765432   12345    0    0    0     0          0         0 54321098   9876    0    0    0     0       0          0
 wlan0:  5678901    2345    0    0    0     0          0         0  3456789    1234    0    0    0     0       0          0
";

    #[test]
    fn parses_net_dev() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("proc/net/dev");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, NET_DEV).unwrap();

        let mut collector = NetworkCollector::new(tmp.path());
        let mut data = NetworkData::default();
        collector.collect(&mut data).unwrap();

        // lo should be skipped
        assert_eq!(data.interfaces.len(), 2);
        assert_eq!(data.interfaces[0].name, "eth0");
        assert_eq!(data.interfaces[0].rx_bytes, 98765432);
        assert_eq!(data.interfaces[0].tx_bytes, 54321098);
        assert_eq!(data.interfaces[1].name, "wlan0");

        // First sample: rates should be 0
        assert_eq!(data.interfaces[0].rx_bytes_per_sec, 0.0);
    }

    #[test]
    fn handles_missing_file() {
        let tmp = TempDir::new().unwrap();
        let mut collector = NetworkCollector::new(tmp.path());
        let mut data = NetworkData::default();
        collector.collect(&mut data).unwrap();
        assert!(data.interfaces.is_empty());
    }
}
