//! Minimal logical model for Gravither's Admin program.
//!
//! Admin is represented independently from the entity it acts for. Its logical
//! PID identifies the Admin program instance; the identifier and kind identify
//! the user or process represented by that instance.

/// The kind of entity represented by an Admin instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepresentationKind {
    User,
    Process,
}

/// An Admin program instance.
///
/// The represented entity identifier is kept as a string so callers can supply
/// identifiers from different namespaces without forcing them into a numeric
/// format. The PID is Admin's own logical process identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admin {
    identifier: String,
    pid: u64,
    representation: RepresentationKind,
}

impl Admin {
    /// Create an Admin instance representing a process, with logical PID 1.
    ///
    /// The represented entity may use any non-empty identifier.
    pub fn new(identifier: impl Into<String>) -> Self {
        Self::with_pid_and_kind(identifier, 1, RepresentationKind::Process)
    }

    /// Create an Admin instance representing a user, with logical PID 1.
    pub fn for_user(identifier: impl Into<String>) -> Self {
        Self::with_pid_and_kind(identifier, 1, RepresentationKind::User)
    }

    /// Create an Admin instance representing a process with a specified PID.
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
    /// Create the initial Admin instance: process "1" with PID 1.
    fn default() -> Self {
        Self::new("1")
    }
}

#[cfg(test)]
mod tests {
    use super::{Admin, RepresentationKind};

    #[test]
    fn default_admin_uses_identifier_and_pid_one() {
        let admin = Admin::default();

        assert_eq!(admin.identifier(), "1");
        assert_eq!(admin.pid(), 1);
        assert_eq!(admin.representation(), RepresentationKind::Process);
    }

    #[test]
    fn accepts_arbitrary_process_identifier() {
        let admin = Admin::new("machine:operator/process-A");

        assert_eq!(admin.identifier(), "machine:operator/process-A");
        assert_eq!(admin.pid(), 1);
        assert_eq!(admin.representation(), RepresentationKind::Process);
    }

    #[test]
    fn can_represent_a_user() {
        let admin = Admin::for_user("user-ulid-01");

        assert_eq!(admin.identifier(), "user-ulid-01");
        assert_eq!(admin.pid(), 1);
        assert_eq!(admin.representation(), RepresentationKind::User);
    }

    #[test]
    fn can_set_an_independent_pid_and_representation() {
        let admin =
            Admin::with_pid_and_kind("operator-7", 42, RepresentationKind::User);

        assert_eq!(admin.identifier(), "operator-7");
        assert_eq!(admin.pid(), 42);
        assert_eq!(admin.representation(), RepresentationKind::User);
    }
}
