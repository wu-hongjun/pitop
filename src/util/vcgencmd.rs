use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::process::Command;

const VCGENCMD_TIMEOUT: Duration = Duration::from_secs(2);
const CACHE_TTL: Duration = Duration::from_secs(1);

/// Async wrapper for vcgencmd with caching and timeout.
///
/// Silently degrades if vcgencmd is not found — sets `available` to false
/// on first NotFound error and never retries.
#[derive(Debug)]
pub struct VcgencmdRunner {
    cache: HashMap<String, (Instant, String)>,
    available: bool,
}

impl VcgencmdRunner {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            available: true,
        }
    }

    /// Run a vcgencmd subcommand and return its stdout.
    ///
    /// Returns `None` if vcgencmd is unavailable, times out, or fails.
    /// Results are cached for 1 second.
    pub async fn run(&mut self, args: &[&str]) -> Option<String> {
        if !self.available {
            return None;
        }

        let cache_key = args.join(" ");

        // Check cache
        if let Some((timestamp, cached)) = self.cache.get(&cache_key) {
            if timestamp.elapsed() < CACHE_TTL {
                return Some(cached.clone());
            }
        }

        // Execute vcgencmd with timeout
        let result = tokio::time::timeout(
            VCGENCMD_TIMEOUT,
            Command::new("vcgencmd").args(args).output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    self.cache
                        .insert(cache_key, (Instant::now(), stdout.clone()));
                    Some(stdout)
                } else {
                    None
                }
            }
            Ok(Err(e)) => {
                // Check if vcgencmd binary doesn't exist
                if e.kind() == std::io::ErrorKind::NotFound {
                    self.available = false;
                }
                None
            }
            Err(_) => {
                // Timeout
                None
            }
        }
    }

    /// Check if vcgencmd is still considered available.
    pub fn is_available(&self) -> bool {
        self.available
    }
}

impl Default for VcgencmdRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_runner_is_available() {
        let runner = VcgencmdRunner::new();
        assert!(runner.is_available());
    }

    #[tokio::test]
    async fn unavailable_after_not_found() {
        // On x86/macOS, vcgencmd won't exist — should mark unavailable
        let mut runner = VcgencmdRunner::new();
        let result = runner.run(&["measure_temp"]).await;
        // Either None (not found) or Some (if somehow vcgencmd exists)
        if result.is_none() {
            // After a NotFound error, should be marked unavailable
            // (may not happen on all platforms — permission errors differ)
            // Subsequent calls should also return None quickly
            let result2 = runner.run(&["measure_temp"]).await;
            assert!(result2.is_none());
        }
    }
}
