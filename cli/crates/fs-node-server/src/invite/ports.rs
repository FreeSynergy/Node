// fs-node-server/src/invite/ports.rs — PortPool: per-invite port allocation.
//
// Each invite gets its own dedicated TCP port so the invited node can reach
// the inviting node through a firewall rule scoped to that single connection.
//
// The pool manages a contiguous range [min, max). Allocation is O(1) average;
// release puts the port back into the available set.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

// ── PortPool ──────────────────────────────────────────────────────────────────

/// Thread-safe port allocator for Node invitations.
///
/// Maintains a set of available ports in `[min, max)`.
/// Clone-able — all clones share the same underlying pool.
#[derive(Clone, Debug)]
pub struct PortPool {
    inner: Arc<Mutex<PoolInner>>,
}

#[derive(Debug)]
struct PoolInner {
    available: BTreeSet<u16>,
    min: u16,
    max: u16,
}

impl PortPool {
    /// Create a pool over the range `[min, max)`.
    ///
    /// # Panics
    ///
    /// Panics if `min >= max`.
    #[must_use]
    pub fn new(min: u16, max: u16) -> Self {
        assert!(min < max, "port range must be non-empty: {min} >= {max}");
        let available: BTreeSet<u16> = (min..max).collect();
        Self {
            inner: Arc::new(Mutex::new(PoolInner {
                available,
                min,
                max,
            })),
        }
    }

    /// Allocate one port from the pool.
    ///
    /// Returns `None` if all ports are in use.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned (should never happen in normal operation).
    #[must_use]
    pub fn allocate(&self) -> Option<u16> {
        let mut pool = self.inner.lock().expect("port pool lock poisoned");
        let port = *pool.available.iter().next()?;
        pool.available.remove(&port);
        Some(port)
    }

    /// Return a previously allocated port to the pool.
    ///
    /// No-op if `port` is outside the pool range or already available.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn release(&self, port: u16) {
        let mut pool = self.inner.lock().expect("port pool lock poisoned");
        if port >= pool.min && port < pool.max {
            pool.available.insert(port);
        }
    }

    /// Number of currently available ports.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn available_count(&self) -> usize {
        self.inner
            .lock()
            .expect("port pool lock poisoned")
            .available
            .len()
    }

    /// Whether `port` is currently allocated (not available).
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn is_allocated(&self, port: u16) -> bool {
        !self
            .inner
            .lock()
            .expect("port pool lock poisoned")
            .available
            .contains(&port)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_release() {
        let pool = PortPool::new(9100, 9103);
        assert_eq!(pool.available_count(), 3);

        let p1 = pool.allocate().unwrap();
        let p2 = pool.allocate().unwrap();
        let p3 = pool.allocate().unwrap();
        assert_eq!(pool.available_count(), 0);
        assert!(pool.allocate().is_none());

        pool.release(p2);
        assert_eq!(pool.available_count(), 1);
        assert!(pool.is_allocated(p1));
        assert!(!pool.is_allocated(p2));
        assert!(pool.is_allocated(p3));
    }

    #[test]
    fn out_of_range_release_is_noop() {
        let pool = PortPool::new(9100, 9102);
        pool.release(8000); // outside range — must not panic
        assert_eq!(pool.available_count(), 2);
    }

    #[test]
    fn clone_shares_pool() {
        let pool = PortPool::new(9200, 9202);
        let clone = pool.clone();
        let p = pool.allocate().unwrap();
        assert!(clone.is_allocated(p));
    }
}
