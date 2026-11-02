//! Minimal User process model.

use ulid::Ulid;

/// Logical PID of User when registered in the anchor space.
pub const ANCHOR_PID: u64 = 3;

/// Logical PID of User when registered inside its Session.
pub const SESSION_PID: u64 = 0;

/// A User process represents an existing user entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    entity_id: Ulid,
}

impl User {
    /// Create a User process for an existing user identifier.
    pub fn new(entity_id: Ulid) -> Self {
        Self { entity_id }
    }

    /// The identifier of the user represented by this process.
    pub fn entity_id(&self) -> Ulid {
        self.entity_id
    }

    /// The process name.
    pub fn process_name(&self) -> String {
        format!("user - {}", self.entity_id)
    }

    /// User's logical PID in the anchor space for this prototype.
    pub fn anchor_pid(&self) -> u64 {
        ANCHOR_PID
    }

    /// User's logical PID inside its Session for this prototype.
    pub fn session_pid(&self) -> u64 {
        SESSION_PID
    }
}

#[cfg(test)]
mod tests {
    use super::{User, ANCHOR_PID, SESSION_PID};
    use ulid::Ulid;

    #[test]
    fn user_keeps_its_entity_id_and_process_name() {
        let id = Ulid::new();
        let user = User::new(id);

        assert_eq!(user.entity_id(), id);
        assert_eq!(user.process_name(), format!("user - {id}"));
    }

    #[test]
    fn user_has_prototype_pids_for_both_contexts() {
        let user = User::new(Ulid::new());

        assert_eq!(user.anchor_pid(), 3);
        assert_eq!(user.anchor_pid(), ANCHOR_PID);
        assert_eq!(user.session_pid(), 0);
        assert_eq!(user.session_pid(), SESSION_PID);
    }
}
