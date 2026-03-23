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
    pub operstate: String,
    pub mac: String,
    pub ipv6: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct NetworkData {
    pub interfaces: Vec<InterfaceData>,
    pub total_rx_bytes_per_sec: f64,
    pub total_tx_bytes_per_sec: f64,
    pub connection_count: usize,
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

        // Pre-parse IPv6 addresses from /proc/net/if_inet6
        let ipv6_map = parse_if_inet6(&self.root);

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

                // Read interface details from sysfs
                let operstate =
                    read_sysfs_string(&self.root.join(format!("sys/class/net/{}/operstate", name)));
                let mac =
                    read_sysfs_string(&self.root.join(format!("sys/class/net/{}/address", name)));
                let ipv6 = ipv6_map.get(&name).cloned().unwrap_or_default();

                data.interfaces.push(InterfaceData {
                    name: name.clone(),
                    rx_bytes,
                    tx_bytes,
                    rx_bytes_per_sec: rx_rate,
                    tx_bytes_per_sec: tx_rate,
                    operstate,
                    mac,
                    ipv6,
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

        // Parse connection count from /proc/net/tcp and /proc/net/tcp6
        data.connection_count = count_established_connections(&self.root);

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

/// Read a single-line sysfs file, returning trimmed content or empty string on error.
fn read_sysfs_string(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Parse /proc/net/if_inet6 and return a map of interface name -> formatted IPv6 address.
/// Each line format: "addr ifindex prefix_len scope flags ifname"
/// addr is a 32-char hex string representing 16 bytes.
fn parse_if_inet6(root: &Path) -> HashMap<String, String> {
    let path = root.join("proc/net/if_inet6");
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut result: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 6 {
            continue;
        }
        let hex_addr = fields[0];
        let ifname = fields[5].to_string();

        // Skip loopback
        if ifname == "lo" {
            continue;
        }

        // Only store the first (usually global-scope) address per interface.
        // Scope field is fields[3]: 00 = global, 20 = link-local, 10 = site, 80 = compat
        // Prefer global scope (00), skip if we already have a global one
        if let Some(addr) = format_ipv6_hex(hex_addr) {
            let scope = fields[3];
            let existing = result.get(&ifname);
            // Only replace if we don't have one yet, or if the new one is global scope
            if existing.is_none() || scope == "00" {
                result.insert(ifname, addr);
            }
        }
    }

    result
}

/// Convert a 32-char hex string to a formatted IPv6 address.
/// e.g., "fe800000000000000000000000000001" -> "fe80::1"
fn format_ipv6_hex(hex: &str) -> Option<String> {
    if hex.len() != 32 {
        return None;
    }

    // Split into 8 groups of 4 hex chars
    let groups: Vec<&str> = (0..8).map(|i| &hex[i * 4..(i + 1) * 4]).collect();

    // Build full colon-separated address, stripping leading zeros per group
    let full: Vec<String> = groups
        .iter()
        .map(|g| {
            let stripped = g.trim_start_matches('0');
            if stripped.is_empty() {
                "0".to_string()
            } else {
                stripped.to_string()
            }
        })
        .collect();

    // Find the longest run of consecutive "0" groups for :: compression
    let mut best_start = 0usize;
    let mut best_len = 0usize;
    let mut cur_start = 0usize;
    let mut cur_len = 0usize;

    for (i, g) in full.iter().enumerate() {
        if g == "0" {
            if cur_len == 0 {
                cur_start = i;
            }
            cur_len += 1;
            if cur_len > best_len {
                best_start = cur_start;
                best_len = cur_len;
            }
        } else {
            cur_len = 0;
        }
    }

    if best_len >= 2 {
        let before: Vec<&str> = full[..best_start].iter().map(|s| s.as_str()).collect();
        let after: Vec<&str> = full[best_start + best_len..]
            .iter()
            .map(|s| s.as_str())
            .collect();
        let before_str = before.join(":");
        let after_str = after.join(":");
        if before_str.is_empty() && after_str.is_empty() {
            Some("::".to_string())
        } else if before_str.is_empty() {
            Some(format!("::{}", after_str))
        } else if after_str.is_empty() {
            Some(format!("{}::", before_str))
        } else {
            Some(format!("{}::{}", before_str, after_str))
        }
    } else {
        Some(full.join(":"))
    }
}

/// Count ESTABLISHED TCP connections from /proc/net/tcp and /proc/net/tcp6.
/// In /proc/net/tcp, connection state is the second field (after "sl" header).
/// State "01" = ESTABLISHED.
fn count_established_connections(root: &Path) -> usize {
    let mut count = 0;
    for filename in &["proc/net/tcp", "proc/net/tcp6"] {
        let path = root.join(filename);
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for line in content.lines().skip(1) {
            // Skip header line
            let fields: Vec<&str> = line.split_whitespace().collect();
            // Field 3 (0-indexed) is the connection state
            if let Some(state) = fields.get(3) {
                if *state == "01" {
                    count += 1;
                }
            }
        }
    }
    count
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

    fn setup_sysfs(tmp: &TempDir) {
        // Create sysfs entries for eth0
        let eth0 = tmp.path().join("sys/class/net/eth0");
        fs::create_dir_all(&eth0).unwrap();
        fs::write(eth0.join("operstate"), "up\n").unwrap();
        fs::write(eth0.join("address"), "dc:a6:32:aa:bb:cc\n").unwrap();

        // Create sysfs entries for wlan0
        let wlan0 = tmp.path().join("sys/class/net/wlan0");
        fs::create_dir_all(&wlan0).unwrap();
        fs::write(wlan0.join("operstate"), "down\n").unwrap();
        fs::write(wlan0.join("address"), "dc:a6:32:dd:ee:ff\n").unwrap();
    }

    #[test]
    fn parses_net_dev() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("proc/net/dev");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, NET_DEV).unwrap();
        setup_sysfs(&tmp);

        let mut collector = NetworkCollector::new(tmp.path());
        let mut data = NetworkData::default();
        collector.collect(&mut data).unwrap();

        // lo should be skipped
        assert_eq!(data.interfaces.len(), 2);
        assert_eq!(data.interfaces[0].name, "eth0");
        assert_eq!(data.interfaces[0].rx_bytes, 98765432);
        assert_eq!(data.interfaces[0].tx_bytes, 54321098);
        assert_eq!(data.interfaces[0].operstate, "up");
        assert_eq!(data.interfaces[0].mac, "dc:a6:32:aa:bb:cc");
        assert_eq!(data.interfaces[1].name, "wlan0");
        assert_eq!(data.interfaces[1].operstate, "down");

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

    #[test]
    fn parses_ipv6_addresses() {
        let tmp = TempDir::new().unwrap();

        // Create /proc/net/if_inet6
        let inet6_path = tmp.path().join("proc/net/if_inet6");
        fs::create_dir_all(inet6_path.parent().unwrap()).unwrap();
        fs::write(
            &inet6_path,
            "fe80000000000000021e06fffe123456 02 40 20 80   eth0\n\
             00000000000000000000000000000001 01 80 10 80       lo\n\
             2001048800040001021e06fffe123456 02 40 00 00   eth0\n",
        )
        .unwrap();

        let map = parse_if_inet6(tmp.path());
        // Should have eth0 with global scope address (scope 00)
        assert!(map.contains_key("eth0"));
        // lo should be skipped
        assert!(!map.contains_key("lo"));
        // Prefer global scope
        let addr = map.get("eth0").unwrap();
        assert!(addr.starts_with("2001:"));
    }

    #[test]
    fn formats_ipv6_hex() {
        // fe80::1
        assert_eq!(
            format_ipv6_hex("fe800000000000000000000000000001"),
            Some("fe80::1".to_string())
        );

        // :: (all zeros)
        assert_eq!(
            format_ipv6_hex("00000000000000000000000000000000"),
            Some("::".to_string())
        );

        // ::1 (loopback)
        assert_eq!(
            format_ipv6_hex("00000000000000000000000000000001"),
            Some("::1".to_string())
        );

        // Invalid length
        assert_eq!(format_ipv6_hex("abc"), None);
    }

    #[test]
    fn counts_established_connections() {
        let tmp = TempDir::new().unwrap();
        let tcp_path = tmp.path().join("proc/net/tcp");
        fs::create_dir_all(tcp_path.parent().unwrap()).unwrap();

        // Header + 3 entries: 2 ESTABLISHED (01), 1 LISTEN (0A)
        let tcp_content = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345
   1: 0100007F:1F90 0100007F:D4C8 01 00000000:00000000 00:00000000 00000000  1000        0 23456
   2: C0A80164:01BB AC100A0A:E1B2 01 00000000:00000000 00:00000000 00000000  1000        0 34567
";
        fs::write(&tcp_path, tcp_content).unwrap();

        let tcp6_path = tmp.path().join("proc/net/tcp6");
        // Header + 1 ESTABLISHED
        let tcp6_content = "\
  sl  local_address                         remote_address                        st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000000000000000000000000000:0050 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 45678
   1: 00000000000000000000000001000000:1F90 00000000000000000000000001000000:D4C8 01 00000000:00000000 00:00000000 00000000  1000        0 56789
";
        fs::write(&tcp6_path, tcp6_content).unwrap();

        let count = count_established_connections(tmp.path());
        assert_eq!(count, 3); // 2 from tcp + 1 from tcp6
    }

    #[test]
    fn connection_count_handles_missing_files() {
        let tmp = TempDir::new().unwrap();
        let count = count_established_connections(tmp.path());
        assert_eq!(count, 0);
    }
}
