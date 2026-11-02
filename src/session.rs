//! Minimal Session process model.

use ulid::Ulid;

/// A Session maintains the logical PIDs of processes in its context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pid: u64,
    entity_id: Ulid,
    processes: Vec<u64>,
}

impl Session {
    /// Create a Session for an entity.
    pub fn new(pid: u64, entity_id: Ulid) -> Self {
        Self {
            pid,
            entity_id,
            processes: Vec::new(),
        }
    }

    /// Create the initial anchor-space Session with a generated entity ID.
    pub fn anchor() -> Self {
        Self::new(2, Ulid::new())
    }

    /// The Session process's logical PID.
    pub fn pid(&self) -> u64 {
        self.pid
    }

    /// The entity represented by this Session.
    pub fn entity_id(&self) -> Ulid {
        self.entity_id
    }

    /// The Session process name.
    pub fn process_name(&self) -> String {
        format!("session - {}", self.entity_id)
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

    /// A Session stays alive while it contains at least one process.
    pub fn should_remain_alive(&self) -> bool {
        !self.processes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Session;
    use ulid::Ulid;

    #[test]
    fn session_has_its_own_pid_and_entity_name() {
        let id = Ulid::new();
        let session = Session::new(2, id);

        assert_eq!(session.pid(), 2);
        assert_eq!(session.entity_id(), id);
        assert_eq!(session.process_name(), format!("session - {id}"));
    }

    #[test]
    fn session_lifetime_depends_only_on_membership() {
        let mut session = Session::new(2, Ulid::new());

        assert!(!session.should_remain_alive());
        assert!(session.add_process(1));
        assert!(session.should_remain_alive());
        assert!(!session.add_process(1));
        assert!(session.remove_process(1));
        assert!(!session.should_remain_alive());
        assert!(!session.remove_process(1));
    }
}
