//! Minimal logical model for Gravither's Admin program.
//!
//! Each Admin instance has its own logical PID and a process name composed of
//! the program name and the identifier of the user or process it represents.

/// The kind of entity represented by an Admin instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepresentationKind {
    User,
    Process,
}

/// An Admin program instance.
///
/// The represented entity identifier accepts values from different identifier
/// namespaces. Admin's process name follows the format `admin - <identifier>`,
/// while its logical PID independently identifies this Admin instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admin {
    identifier: String,
    pid: u64,
    representation: RepresentationKind,
}

impl Admin {
    /// Create an Admin instance representing a process, with logical PID 1.
    ///
    /// The represented entity may use any identifier.
    pub fn new(identifier: impl Into<String>) -> Self {
        Self::with_pid_and_kind(identifier, 1, RepresentationKind::Process)
    }

    /// Create an Admin instance representing a user, with logical PID 1.
    pub fn for_user(identifier: impl Into<String>) -> Self {
        Self::with_pid_and_kind(identifier, 1, RepresentationKind::User)
    }

    /// Create an Admin instance representing a process, with logical PID 1.
    pub fn for_process(identifier: impl Into<String>) -> Self {
        Self::with_pid_and_kind(identifier, 1, RepresentationKind::Process)
    }

    /// Create an Admin instance with an explicit logical PID and entity kind.
    pub fn with_pid_and_kind(
        identifier: impl Into<String>,
        pid: u64,
        representation: RepresentationKind,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            pid,
            representation,
        }
    }

    /// Return the identifier of the user or process represented by this Admin.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Return this process's name, combining the program name and represented ID.
    ///
    /// For example, an Admin representing process "1" is named "admin - 1".
    pub fn process_name(&self) -> String {
        format!("admin - {}", self.identifier)
    }

    /// Return Admin's own logical process identifier.
    pub fn pid(&self) -> u64 {
        self.pid
    }

    /// Return whether this instance represents a user or a process.
    pub fn representation(&self) -> RepresentationKind {
        self.representation
    }
}

impl Default for Admin {
    /// Create the initial Admin instance representing process "1", with PID 1.
    fn default() -> Self {
        Self::new("1")
    }
}

#[cfg(test)]
mod tests {
    use super::{Admin, RepresentationKind};

    #[test]
    fn default_admin_uses_process_name_admin_one_and_pid_one() {
        let admin = Admin::default();

        assert_eq!(admin.identifier(), "1");
        assert_eq!(admin.process_name(), "admin - 1");
        assert_eq!(admin.pid(), 1);
        assert_eq!(admin.representation(), RepresentationKind::Process);
    }

    #[test]
    fn accepts_arbitrary_process_identifier() {
        let admin = Admin::new("machine:operator/process-A");

        assert_eq!(admin.identifier(), "machine:operator/process-A");
        assert_eq!(admin.process_name(), "admin - machine:operator/process-A");
        assert_eq!(admin.pid(), 1);
        assert_eq!(admin.representation(), RepresentationKind::Process);
    }

    #[test]
    fn can_represent_a_user() {
        let admin = Admin::for_user("user-ulid-01");

        assert_eq!(admin.identifier(), "user-ulid-01");
        assert_eq!(admin.process_name(), "admin - user-ulid-01");
        assert_eq!(admin.pid(), 1);
        assert_eq!(admin.representation(), RepresentationKind::User);
    }

    #[test]
    fn can_set_an_independent_pid_and_representation() {
        let admin =
            Admin::with_pid_and_kind("operator-7", 42, RepresentationKind::User);

        assert_eq!(admin.identifier(), "operator-7");
        assert_eq!(admin.process_name(), "admin - operator-7");
        assert_eq!(admin.pid(), 42);
        assert_eq!(admin.representation(), RepresentationKind::User);
    }
}
