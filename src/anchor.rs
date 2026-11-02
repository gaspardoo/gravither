//! Minimal process list for Gravither's anchor space.

/// The anchor space keeps track of its foundational processes by logical PID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorSpace {
    processes: Vec<u64>,
}

impl AnchorSpace {
    /// Create an empty anchor space.
    pub fn new() -> Self {
        Self { processes: Vec::new() }
    }

    /// Register a process PID. Returns false if it is already registered.
    pub fn add_process(&mut self, pid: u64) -> bool {
        if self.processes.contains(&pid) {
            return false;
        }
        self.processes.push(pid);
        true
    }

    /// Remove a process PID. Returns false if it was not registered.
    pub fn remove_process(&mut self, pid: u64) -> bool {
        if let Some(index) = self.processes.iter().position(|process| *process == pid) {
            self.processes.remove(index);
            true
        } else {
            false
        }
    }

    /// Check whether a process PID belongs to the anchor space.
    pub fn contains_process(&self, pid: u64) -> bool {
        self.processes.contains(&pid)
    }
}

impl Default for AnchorSpace {
    /// The initial anchor space contains Sysruning (PID 0), Admin (PID 1), and Session (PID 2).
    fn default() -> Self {
        let mut space = Self::new();
        space.add_process(0);
        space.add_process(1);
        space.add_process(2);
        space
    }
}

#[cfg(test)]
mod tests {
    use super::AnchorSpace;

    #[test]
    fn default_anchor_space_keeps_admin_and_session() {
        let space = AnchorSpace::default();
        assert!(space.contains_process(0));
        assert!(space.contains_process(1));
        assert!(space.contains_process(2));
        assert!(!space.contains_process(3));
    }

    #[test]
    fn tracks_process_membership_without_duplicates() {
        let mut space = AnchorSpace::new();
        assert!(space.add_process(1));
        assert!(!space.add_process(1));
        assert!(space.contains_process(1));
        assert!(space.remove_process(1));
        assert!(!space.contains_process(1));
        assert!(!space.remove_process(1));
    }
}
