//! Minimal logical model for Gravither's Session process.
//!
//! A Session represents an entity identified by a ULID. It remains alive while
//! at least one User or Admin process belongs to it, including hibernating ones.

use ulid::Ulid;

/// A process type that can keep a Session alive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProcessKind {
    Admin,
    User,
}

/// The lifecycle state of a process associated with a Session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Hibernating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SessionProcess {
    pid: u64,
    kind: SessionProcessKind,
    state: ProcessState,
}

/// A Session process and the entity whose context it coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pid: u64,
    entity_id: Ulid,
    processes: Vec<SessionProcess>,
}

impl Session {
    /// Create a Session with an explicit logical PID and entity ULID.
    pub fn new(pid: u64, entity_id: Ulid) -> Self {
        Self {
            pid,
            entity_id,
            processes: Vec::new(),
        }
    }

    /// Create a Session with a randomly generated entity ULID.
    pub fn new_random(pid: u64) -> Self {
        Self::new(pid, Ulid::new())
    }

    /// Create the initial anchor-space Session, using logical PID 2.
    pub fn anchor() -> Self {
        Self::new_random(2)
    }

    /// Return the Session process's logical PID.
    pub fn pid(&self) -> u64 {
        self.pid
    }

    /// Return the ULID of the entity represented by this Session.
    pub fn entity_id(&self) -> Ulid {
        self.entity_id
    }

    /// Return the process name for this Session.
    pub fn process_name(&self) -> String {
        format!("session - {}", self.entity_id)
    }

    /// Attach an Admin or User process to this Session.
    ///
    /// Process PIDs must be unique within the Session.
    pub fn add_process(
        &mut self,
        pid: u64,
        kind: SessionProcessKind,
    ) -> Result<(), SessionError> {
        if self.processes.iter().any(|process| process.pid == pid) {
            return Err(SessionError::DuplicateProcessPid(pid));
        }

        self.processes.push(SessionProcess {
            pid,
            kind,
            state: ProcessState::Running,
        });
        Ok(())
    }

    /// Mark an associated process as hibernating without removing it.
    pub fn hibernate_process(&mut self, pid: u64) -> Result<(), SessionError> {
        let process = self
            .processes
            .iter_mut()
            .find(|process| process.pid == pid)
            .ok_or(SessionError::ProcessNotFound(pid))?;
        process.state = ProcessState::Hibernating;
        Ok(())
    }

    /// Resume an associated hibernating process.
    pub fn resume_process(&mut self, pid: u64) -> Result<(), SessionError> {
        let process = self
            .processes
            .iter_mut()
            .find(|process| process.pid == pid)
            .ok_or(SessionError::ProcessNotFound(pid))?;
        process.state = ProcessState::Running;
        Ok(())
    }

    /// Remove an associated process from this Session.
    pub fn remove_process(&mut self, pid: u64) -> Result<(), SessionError> {
        let index = self
            .processes
            .iter()
            .position(|process| process.pid == pid)
            .ok_or(SessionError::ProcessNotFound(pid))?;
        self.processes.remove(index);
        Ok(())
    }

    /// Return whether this Session has at least one associated Admin or User.
    ///
    /// Hibernating processes still count, so they keep the Session alive.
    pub fn should_remain_alive(&self) -> bool {
        !self.processes.is_empty()
    }

    /// Return the number of associated processes.
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }
}

/// Errors from Session process membership operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    DuplicateProcessPid(u64),
    ProcessNotFound(u64),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateProcessPid(pid) => {
                write!(f, "process PID {pid} is already in this session")
            }
            Self::ProcessNotFound(pid) => {
                write!(f, "process PID {pid} was not found in this session")
            }
        }
    }
}

impl std::error::Error for SessionError {}

#[cfg(test)]
mod tests {
    use super::{ProcessState, Session, SessionError, SessionProcessKind};
    use ulid::Ulid;

    #[test]
    fn anchor_session_uses_pid_two_and_has_a_ulid_name() {
        let session = Session::anchor();

        assert_eq!(session.pid(), 2);
        assert!(session.process_name().starts_with("session - "));
        assert_eq!(session.process_name(), format!("session - {}", session.entity_id()));
    }

    #[test]
    fn explicit_session_preserves_its_entity_ulid() {
        let id = Ulid::new();
        let session = Session::new(2, id);

        assert_eq!(session.entity_id(), id);
        assert_eq!(session.process_name(), format!("session - {id}"));
    }

    #[test]
    fn session_needs_a_process_to_remain_alive() {
        let mut session = Session::new_random(2);
        assert!(!session.should_remain_alive());

        session.add_process(1, SessionProcessKind::Admin).unwrap();
        assert!(session.should_remain_alive());
        assert_eq!(session.process_count(), 1);
    }

    #[test]
    fn hibernating_process_keeps_session_alive() {
        let mut session = Session::new_random(2);
        session.add_process(1, SessionProcessKind::Admin).unwrap();

        session.hibernate_process(1).unwrap();

        assert!(session.should_remain_alive());
        assert_eq!(session.processes[0].state, ProcessState::Hibernating);
    }

    #[test]
    fn removing_last_process_allows_session_to_terminate() {
        let mut session = Session::new_random(2);
        session.add_process(1, SessionProcessKind::Admin).unwrap();
        session.hibernate_process(1).unwrap();

        session.remove_process(1).unwrap();

        assert!(!session.should_remain_alive());
        assert_eq!(session.process_count(), 0);
    }

    #[test]
    fn rejects_duplicate_process_pids() {
        let mut session = Session::new_random(2);
        session.add_process(1, SessionProcessKind::Admin).unwrap();

        assert_eq!(
            session.add_process(1, SessionProcessKind::User),
            Err(SessionError::DuplicateProcessPid(1))
        );
        assert_eq!(session.process_count(), 1);
    }

    #[test]
    fn reports_missing_processes() {
        let mut session = Session::new_random(2);

        assert_eq!(
            session.hibernate_process(7),
            Err(SessionError::ProcessNotFound(7))
        );
        assert_eq!(
            session.remove_process(7),
            Err(SessionError::ProcessNotFound(7))
        );
    }

    #[test]
    fn user_process_also_keeps_session_alive() {
        let mut session = Session::new_random(2);
        session.add_process(3, SessionProcessKind::User).unwrap();
        session.hibernate_process(3).unwrap();

        assert!(session.should_remain_alive());
        session.resume_process(3).unwrap();
        assert_eq!(session.processes[0].state, ProcessState::Running);
    }
}
