use serde::Serialize;

const DEFAULT_CAPACITY: usize = 60;

/// Fixed-size circular buffer for sparkline history.
///
/// Stores the last N samples in chronological order. When full,
/// the oldest sample is evicted on push.
#[derive(Debug, Clone, Serialize)]
pub struct RingBuffer<T> {
    buf: Vec<T>,
    capacity: usize,
    write_pos: usize,
    full: bool,
}

impl<T: Clone + Default> Default for RingBuffer<T> {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: vec![T::default(); capacity],
            capacity,
            write_pos: 0,
            full: false,
        }
    }

    /// Push a new sample, evicting the oldest if full.
    pub fn push(&mut self, value: T) {
        self.buf[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.write_pos == 0 && !self.full {
            self.full = true;
        }
    }

    /// Return samples in chronological order (oldest first).
    pub fn as_slice(&self) -> Vec<T> {
        if self.full {
            // Buffer has wrapped: [write_pos..] + [..write_pos]
            let mut result = Vec::with_capacity(self.capacity);
            result.extend_from_slice(&self.buf[self.write_pos..]);
            result.extend_from_slice(&self.buf[..self.write_pos]);
            result
        } else {
            // Buffer hasn't wrapped yet: [0..write_pos]
            self.buf[..self.write_pos].to_vec()
        }
    }

    pub fn len(&self) -> usize {
        if self.full {
            self.capacity
        } else {
            self.write_pos
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.full && self.write_pos == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Return the most recently pushed value, if any.
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let idx = if self.write_pos == 0 {
                self.capacity - 1
            } else {
                self.write_pos - 1
            };
            Some(&self.buf[idx])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let buf: RingBuffer<f64> = RingBuffer::new(5);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 5);
        assert!(buf.as_slice().is_empty());
    }

    #[test]
    fn push_within_capacity() {
        let mut buf = RingBuffer::new(5);
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);

        assert_eq!(buf.len(), 3);
        assert!(!buf.is_empty());
        assert_eq!(buf.as_slice(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn push_at_capacity() {
        let mut buf = RingBuffer::new(3);
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.as_slice(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn push_wraps_around() {
        let mut buf = RingBuffer::new(3);
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        buf.push(4.0); // evicts 1.0

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.as_slice(), vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn push_multiple_wraps() {
        let mut buf = RingBuffer::new(3);
        for i in 1..=10 {
            buf.push(i as f64);
        }

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.as_slice(), vec![8.0, 9.0, 10.0]);
    }

    #[test]
    fn default_capacity_is_60() {
        let buf: RingBuffer<f64> = RingBuffer::default();
        assert_eq!(buf.capacity(), 60);
    }

    #[test]
    fn last_returns_most_recent() {
        let mut buf = RingBuffer::new(3);
        assert!(buf.last().is_none());

        buf.push(10.0);
        assert_eq!(*buf.last().unwrap(), 10.0);

        buf.push(20.0);
        assert_eq!(*buf.last().unwrap(), 20.0);

        buf.push(30.0);
        buf.push(40.0); // wraps
        assert_eq!(*buf.last().unwrap(), 40.0);
    }
}
