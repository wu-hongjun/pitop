use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_PROCESSES: usize = 50;

#[derive(Debug, Default, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub state: char,
    pub cpu_percent: f64,
    pub rss_bytes: u64,
    pub user: String,
}

/// Previous CPU sample for delta computation.
struct PrevProcess {
    utime: u64,
    stime: u64,
    total_cpu: u64,
}

pub struct ProcessCollector {
    root: PathBuf,
    prev: HashMap<u32, PrevProcess>,
    prev_total_cpu: u64,
    max_processes: usize,
}

impl ProcessCollector {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            prev: HashMap::new(),
            prev_total_cpu: 0,
            max_processes: DEFAULT_MAX_PROCESSES,
        }
    }

    pub fn collect(&mut self, processes: &mut Vec<ProcessInfo>) -> Result<()> {
        processes.clear();

        // Read total CPU time for percentage calculation
        let total_cpu = read_total_cpu_jiffies(&self.root);

        let proc_dir = self.root.join("proc");
        let entries = match std::fs::read_dir(&proc_dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        let mut current_procs = HashMap::new();

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Only numeric directories (PIDs)
            let pid: u32 = match name_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let pid_dir = entry.path();

            // Read process name from /proc/PID/comm
            let comm = std::fs::read_to_string(pid_dir.join("comm"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            if comm.is_empty() {
                continue; // Process likely disappeared
            }

            // Parse /proc/PID/stat for CPU time and state
            let (state, utime, stime) = parse_proc_stat(&pid_dir);

            // Read RSS from /proc/PID/status
            let rss_bytes = read_rss_from_status(&pid_dir);

            // Read user from /proc/PID/status (Uid field)
            let user = read_user_from_status(&pid_dir);

            // Compute CPU% from delta
            let cpu_percent = if self.prev_total_cpu > 0 {
                if let Some(prev) = self.prev.get(&pid) {
                    let proc_delta = (utime + stime).saturating_sub(prev.utime + prev.stime) as f64;
                    let total_delta = total_cpu.saturating_sub(self.prev_total_cpu) as f64;
                    if total_delta > 0.0 {
                        (proc_delta / total_delta) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            current_procs.insert(
                pid,
                PrevProcess {
                    utime,
                    stime,
                    total_cpu,
                },
            );

            processes.push(ProcessInfo {
                pid,
                name: comm,
                state,
                cpu_percent,
                rss_bytes,
                user,
            });
        }

        // Sort by CPU% descending, then truncate
        processes.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(self.max_processes);

        // Store for next delta
        self.prev = current_procs;
        self.prev_total_cpu = total_cpu;

        Ok(())
    }
}

/// Parse /proc/PID/stat for state, utime, stime.
/// Format: pid (comm) state ... utime stime ...
/// Fields are space-separated, but comm can contain spaces/parens.
fn parse_proc_stat(pid_dir: &Path) -> (char, u64, u64) {
    let content = match std::fs::read_to_string(pid_dir.join("stat")) {
        Ok(c) => c,
        Err(_) => return ('?', 0, 0),
    };

    // Find the last ')' to skip the comm field safely
    let after_comm = match content.rfind(')') {
        Some(pos) => &content[pos + 2..], // skip ") "
        None => return ('?', 0, 0),
    };

    let parts: Vec<&str> = after_comm.split_whitespace().collect();
    // parts[0] = state, parts[11] = utime, parts[12] = stime
    let state = parts.first().and_then(|s| s.chars().next()).unwrap_or('?');
    let utime = parts.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
    let stime = parts.get(12).and_then(|s| s.parse().ok()).unwrap_or(0);

    (state, utime, stime)
}

/// Read RSS from /proc/PID/status (VmRSS line, in kB).
fn read_rss_from_status(pid_dir: &Path) -> u64 {
    let content = match std::fs::read_to_string(pid_dir.join("status")) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    for line in content.lines() {
        if line.starts_with("VmRSS:") {
            return line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<u64>().ok())
                .map(|kb| kb * 1024)
                .unwrap_or(0);
        }
    }

    0
}

/// Read UID from /proc/PID/status and resolve to username.
fn read_user_from_status(pid_dir: &Path) -> String {
    let content = match std::fs::read_to_string(pid_dir.join("status")) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    for line in content.lines() {
        if line.starts_with("Uid:") {
            let uid_str = line.split_whitespace().nth(1).unwrap_or("0");
            let uid: u32 = uid_str.parse().unwrap_or(0);
            return uid_to_name(uid);
        }
    }

    String::new()
}

/// Best-effort UID to username resolution.
fn uid_to_name(uid: u32) -> String {
    // Common UIDs
    match uid {
        0 => "root".to_string(),
        _ => uid.to_string(),
    }
}

/// Read total CPU jiffies from /proc/stat.
fn read_total_cpu_jiffies(root: &Path) -> u64 {
    let path = root.join("proc/stat");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    for line in content.lines() {
        if line.starts_with("cpu ") {
            return line
                .split_whitespace()
                .skip(1)
                .filter_map(|s| s.parse::<u64>().ok())
                .sum();
        }
    }

    0
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

    #[test]
    fn scans_processes() {
        let tmp = TempDir::new().unwrap();

        write_fixture(
            tmp.path(),
            "proc/stat",
            "cpu  10132153 290696 3084719 46828483 16683 0 25195 0 0 0\n",
        );

        // Create a fake process
        write_fixture(tmp.path(), "proc/1234/comm", "my_process\n");
        write_fixture(
            tmp.path(),
            "proc/1234/stat",
            "1234 (my_process) S 1 1234 1234 0 -1 4194304 100 0 0 0 500 200 0 0 20 0 1 0 100 1000000 100 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
        );
        write_fixture(
            tmp.path(),
            "proc/1234/status",
            "Name:\tmy_process\nUid:\t1000\t1000\t1000\t1000\nVmRSS:\t5120 kB\n",
        );

        let mut collector = ProcessCollector::new(tmp.path());
        let mut procs = Vec::new();
        collector.collect(&mut procs).unwrap();

        assert_eq!(procs.len(), 1);
        assert_eq!(procs[0].pid, 1234);
        assert_eq!(procs[0].name, "my_process");
        assert_eq!(procs[0].state, 'S');
        assert_eq!(procs[0].rss_bytes, 5120 * 1024);
        assert_eq!(procs[0].user, "1000");
    }

    #[test]
    fn handles_missing_proc() {
        let tmp = TempDir::new().unwrap();
        let mut collector = ProcessCollector::new(tmp.path());
        let mut procs = Vec::new();
        collector.collect(&mut procs).unwrap();
        assert!(procs.is_empty());
    }

    #[test]
    fn handles_disappearing_process() {
        let tmp = TempDir::new().unwrap();
        write_fixture(tmp.path(), "proc/stat", "cpu  100 0 0 100 0 0 0 0 0 0\n");
        // Directory exists but comm is missing (process died)
        fs::create_dir_all(tmp.path().join("proc/9999")).unwrap();

        let mut collector = ProcessCollector::new(tmp.path());
        let mut procs = Vec::new();
        collector.collect(&mut procs).unwrap();
        // Should not crash, just skip
        assert!(procs.is_empty());
    }
}
