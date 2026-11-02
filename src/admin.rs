//! Minimal Admin process model.

/// The Admin process in Gravither's anchor space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admin {
    pid: u64,
    represented_id: String,
}

impl Admin {
    /// Create an Admin process with a logical PID and represented identifier.
    pub fn new(pid: u64, represented_id: impl Into<String>) -> Self {
        Self {
            pid,
            represented_id: represented_id.into(),
        }
    }

    /// The process's logical PID.
    pub fn pid(&self) -> u64 {
        self.pid
    }

    /// The process name.
    pub fn process_name(&self) -> String {
        format!("admin - {}", self.represented_id)
    }
}

impl Default for Admin {
    /// The initial anchor-space Admin process.
    fn default() -> Self {
        Self::new(1, "1")
    }
}

#[cfg(test)]
mod tests {
    use super::Admin;

    #[test]
    fn default_admin_is_the_anchor_admin_process() {
        let admin = Admin::default();
        assert_eq!(admin.pid(), 1);
        assert_eq!(admin.process_name(), "admin - 1");
    }

    #[test]
    fn admin_can_represent_another_identifier() {
        let admin = Admin::new(7, "entity-ulid");
        assert_eq!(admin.pid(), 7);
        assert_eq!(admin.process_name(), "admin - entity-ulid");
    }
}
