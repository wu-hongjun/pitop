use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Per-worker state: an individual stop flag and its join handle.
struct Worker {
    stop_flag: Arc<AtomicBool>,
    /// Held to keep the spawned task alive; read on drop.
    #[allow(dead_code)]
    handle: tokio::task::JoinHandle<()>,
}

/// Manages CPU stress test worker threads.
pub struct StressTest {
    /// Global running state (true while the stress test is active).
    running: bool,
    /// Current number of active workers.
    num_workers: usize,
    /// Maximum allowed workers (2x initial worker count).
    max_workers: usize,
    /// Initial worker count (used for color thresholds in UI).
    initial_workers: usize,
    /// Per-worker state.
    workers: Vec<Worker>,
    /// Timestamp when stress test was started.
    started_at: Option<Instant>,
}

impl StressTest {
    pub fn new(num_workers: usize) -> Self {
        let clamped = num_workers.max(1);
        Self {
            running: false,
            num_workers: clamped,
            max_workers: clamped.saturating_mul(2),
            initial_workers: clamped,
            workers: Vec::new(),
            started_at: None,
        }
    }

    /// Start stress test workers.
    pub fn start(&mut self) {
        if self.running {
            return;
        }
        self.running = true;
        self.started_at = Some(Instant::now());

        for _ in 0..self.num_workers {
            self.spawn_worker();
        }
    }

    /// Stop all workers. Signals each worker to exit and waits briefly
    /// for them to actually stop, so CPU load drops before we return.
    pub fn stop(&mut self) {
        self.running = false;
        self.started_at = None;
        // Signal all workers to exit
        for worker in &self.workers {
            worker.stop_flag.store(true, Ordering::SeqCst);
        }
        // Give workers a moment to observe the flag and exit their hot loop.
        // The flag check uses Relaxed ordering, so a short yield is sufficient
        // for the cache line to propagate on all real hardware.
        std::thread::sleep(std::time::Duration::from_millis(5));
        self.workers.clear();
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn num_workers(&self) -> usize {
        self.num_workers
    }

    pub fn max_workers(&self) -> usize {
        self.max_workers
    }

    pub fn initial_workers(&self) -> usize {
        self.initial_workers
    }

    /// Returns elapsed time since stress test was started.
    pub fn elapsed(&self) -> Option<Duration> {
        self.started_at.map(|t| t.elapsed())
    }

    /// Set the number of active workers, clamped to [1, max_workers].
    /// If the test is running, spawns or stops workers to match.
    pub fn set_workers(&mut self, n: usize) {
        let clamped = n.clamp(1, self.max_workers);
        self.num_workers = clamped;

        if !self.running {
            return;
        }

        let current = self.workers.len();
        if clamped > current {
            // Spawn additional workers
            for _ in current..clamped {
                self.spawn_worker();
            }
        } else if clamped < current {
            // Stop excess workers from the end
            for _ in clamped..current {
                if let Some(worker) = self.workers.pop() {
                    worker.stop_flag.store(true, Ordering::SeqCst);
                }
            }
            // Brief yield so stopped workers observe the flag before we return
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    /// Add one worker (up to max_workers).
    pub fn add_worker(&mut self) {
        self.set_workers(self.num_workers.saturating_add(1));
    }

    /// Remove one worker (minimum 1).
    pub fn remove_worker(&mut self) {
        self.set_workers(self.num_workers.saturating_sub(1));
    }

    /// Spawn a single worker task with its own stop flag.
    fn spawn_worker(&mut self) {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag = stop_flag.clone();
        let handle = tokio::task::spawn_blocking(move || {
            // CPU-intensive work: compute primes via trial division
            let mut n: u64 = 2;
            while !flag.load(Ordering::Relaxed) {
                is_prime(n);
                n = n.wrapping_add(1);
            }
        });
        self.workers.push(Worker { stop_flag, handle });
    }
}

impl Drop for StressTest {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Simple primality test for CPU load generation.
fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n.is_multiple_of(2) {
        return false;
    }
    let mut i = 3u64;
    while i.saturating_mul(i) <= n {
        if n.is_multiple_of(i) {
            return false;
        }
        i += 2;
    }
    true
}

/// Count the number of CPU cores by reading /proc/cpuinfo.
/// Returns the count of "processor" lines, or a default of 4.
pub fn num_cpus_from_proc(root: &std::path::Path) -> usize {
    std::fs::read_to_string(root.join("proc/cpuinfo"))
        .ok()
        .map(|contents| {
            contents
                .lines()
                .filter(|line| line.starts_with("processor"))
                .count()
        })
        .unwrap_or(0)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(!is_prime(9));
        assert!(is_prime(97));
    }

    #[test]
    fn test_num_cpus_from_proc_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        // No /proc/cpuinfo exists, should return 1 (max of 0 and 1)
        assert_eq!(num_cpus_from_proc(tmp.path()), 1);
    }

    #[test]
    fn test_stress_new() {
        let stress = StressTest::new(4);
        assert!(!stress.is_running());
        assert_eq!(stress.num_workers(), 4);
        assert_eq!(stress.max_workers(), 8);
        assert_eq!(stress.initial_workers(), 4);
    }

    #[tokio::test]
    async fn test_stress_start_stop() {
        let mut stress = StressTest::new(2);
        assert!(!stress.is_running());

        stress.start();
        assert!(stress.is_running());

        stress.stop();
        assert!(!stress.is_running());
    }

    #[tokio::test]
    async fn test_stress_start_idempotent() {
        let mut stress = StressTest::new(1);
        stress.start();
        assert_eq!(stress.workers.len(), 1);

        // Starting again while running should not add more workers
        stress.start();
        assert_eq!(stress.workers.len(), 1);

        stress.stop();
    }

    #[test]
    fn test_elapsed_none_when_not_running() {
        let stress = StressTest::new(2);
        assert!(stress.elapsed().is_none());
    }

    #[tokio::test]
    async fn test_elapsed_some_when_running() {
        let mut stress = StressTest::new(1);
        stress.start();
        // Small sleep to ensure elapsed > 0
        tokio::time::sleep(Duration::from_millis(10)).await;
        let elapsed = stress.elapsed();
        assert!(elapsed.is_some());
        assert!(elapsed.unwrap_or_default() >= Duration::from_millis(1));
        stress.stop();
        assert!(stress.elapsed().is_none());
    }

    #[tokio::test]
    async fn test_add_worker_increases_count() {
        let mut stress = StressTest::new(2);
        stress.start();
        assert_eq!(stress.num_workers(), 2);
        assert_eq!(stress.workers.len(), 2);

        stress.add_worker();
        assert_eq!(stress.num_workers(), 3);
        assert_eq!(stress.workers.len(), 3);

        stress.stop();
    }

    #[tokio::test]
    async fn test_remove_worker_decreases_count() {
        let mut stress = StressTest::new(3);
        stress.start();
        assert_eq!(stress.num_workers(), 3);
        assert_eq!(stress.workers.len(), 3);

        stress.remove_worker();
        assert_eq!(stress.num_workers(), 2);
        assert_eq!(stress.workers.len(), 2);

        stress.stop();
    }

    #[tokio::test]
    async fn test_remove_worker_minimum_one() {
        let mut stress = StressTest::new(1);
        stress.start();
        assert_eq!(stress.num_workers(), 1);

        stress.remove_worker();
        // Should stay at 1 (minimum)
        assert_eq!(stress.num_workers(), 1);
        assert_eq!(stress.workers.len(), 1);

        stress.stop();
    }

    #[tokio::test]
    async fn test_set_workers_with_clamping() {
        let mut stress = StressTest::new(4);
        // max_workers = 8
        stress.start();

        // Try to set above max
        stress.set_workers(100);
        assert_eq!(stress.num_workers(), 8);
        assert_eq!(stress.workers.len(), 8);

        // Try to set below minimum
        stress.set_workers(0);
        assert_eq!(stress.num_workers(), 1);
        assert_eq!(stress.workers.len(), 1);

        // Set to a normal value
        stress.set_workers(3);
        assert_eq!(stress.num_workers(), 3);
        assert_eq!(stress.workers.len(), 3);

        stress.stop();
    }

    #[test]
    fn test_set_workers_when_not_running() {
        let mut stress = StressTest::new(4);
        // Not running — should update num_workers but not spawn anything
        stress.set_workers(6);
        assert_eq!(stress.num_workers(), 6);
        assert_eq!(stress.workers.len(), 0);
    }
}
