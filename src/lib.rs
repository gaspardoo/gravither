//! Gravither's first connection model.
//!
//! Each user owns two independent logical sockets:
//! - outbound: used to observe and navigate to another user;
//! - inbound: used to expose information explicitly marked as public.
//!
//! This first version models socket state locally. It does not open operating-system
//! TCP/UDP sockets yet; network transport can be added behind this model later.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundSocket {
    pub state: SocketState,
    /// The user this socket is observing, if any.
    pub target: Option<UserId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundSocket {
    pub state: SocketState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub online: bool,
    pub outbound_socket: OutboundSocket,
    pub inbound_socket: InboundSocket,
    /// Information explicitly exposed by this user.
    pub exposed_information: Vec<String>,
}

impl User {
    pub fn new(id: UserId) -> Self {
        Self {
            id,
            online: true,
            outbound_socket: OutboundSocket {
                state: SocketState::Open,
                target: None,
            },
            inbound_socket: InboundSocket {
                state: SocketState::Open,
            },
            exposed_information: Vec::new(),
        }
    }

    fn set_online(&mut self, online: bool) {
        self.online = online;
        if !online {
            self.outbound_socket.state = SocketState::Closed;
            self.outbound_socket.target = None;
            self.inbound_socket.state = SocketState::Closed;
        } else {
            self.outbound_socket.state = SocketState::Open;
            self.inbound_socket.state = SocketState::Open;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionLine {
    /// Information travels from source to observer.
    pub source: UserId,
    pub observer: UserId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InformationFlow {
    pub source: UserId,
    pub observer: UserId,
    pub information: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GravitherError {
    UnknownUser(UserId),
    UserCannotObserveItself,
    UserOffline(UserId),
    InboundSocketClosed(UserId),
    OutboundSocketClosed(UserId),
}

pub struct Gravither {
    users: BTreeMap<UserId, User>,
}

impl Gravither {
    /// Creates the initial two-user environment.
    pub fn two_users(first: impl Into<String>, second: impl Into<String>) -> Self {
        let first = User::new(UserId::new(first));
        let second = User::new(UserId::new(second));

        let mut users = BTreeMap::new();
        users.insert(first.id.clone(), first);
        users.insert(second.id.clone(), second);

        Self { users }
    }

    pub fn user(&self, id: &UserId) -> Option<&User> {
        self.users.get(id)
    }

    pub fn set_user_online(&mut self, id: &UserId, online: bool) -> Result<(), GravitherError> {
        let user = self
            .users
            .get_mut(id)
            .ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        user.set_online(online);

        // If a user goes offline, remove references to that user's inbound socket.
        if !online {
            for other in self.users.values_mut() {
                if other.outbound_socket.target.as_ref() == Some(id) {
                    other.outbound_socket.target = None;
                }
            }
        }
        Ok(())
    }

    /// Opens the observer's outbound socket toward the target's inbound socket.
    /// Information, when exposed by the target, travels target -> observer.
    pub fn observe(
        &mut self,
        observer_id: &UserId,
        target_id: &UserId,
    ) -> Result<(), GravitherError> {
        if observer_id == target_id {
            return Err(GravitherError::UserCannotObserveItself);
        }

        let observer = self
            .users
            .get(observer_id)
            .ok_or_else(|| GravitherError::UnknownUser(observer_id.clone()))?;
        if !observer.online {
            return Err(GravitherError::UserOffline(observer_id.clone()));
        }
        if observer.outbound_socket.state == SocketState::Closed {
            return Err(GravitherError::OutboundSocketClosed(observer_id.clone()));
        }

        let target = self
            .users
            .get(target_id)
            .ok_or_else(|| GravitherError::UnknownUser(target_id.clone()))?;
        if !target.online {
            return Err(GravitherError::UserOffline(target_id.clone()));
        }
        if target.inbound_socket.state == SocketState::Closed {
            return Err(GravitherError::InboundSocketClosed(target_id.clone()));
        }

        self.users
            .get_mut(observer_id)
            .expect("observer was checked above")
            .outbound_socket
            .target = Some(target_id.clone());

        Ok(())
    }

    /// Closes only the observer's outbound connection. The user's inbound
    /// socket remains available for other users to observe them.
    pub fn stop_observing(&mut self, observer_id: &UserId) -> Result<(), GravitherError> {
        let observer = self
            .users
            .get_mut(observer_id)
            .ok_or_else(|| GravitherError::UnknownUser(observer_id.clone()))?;
        observer.outbound_socket.target = None;
        Ok(())
    }

    /// Controls whether other users can connect to this user's inbound socket.
    pub fn set_inbound_socket(
        &mut self,
        id: &UserId,
        state: SocketState,
    ) -> Result<(), GravitherError> {
        let user = self
            .users
            .get_mut(id)
            .ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        if !user.online && state == SocketState::Open {
            return Err(GravitherError::UserOffline(id.clone()));
        }
        user.inbound_socket.state = state;
        Ok(())
    }

    /// Controls whether this user's outbound navigation socket is open.
    pub fn set_outbound_socket(
        &mut self,
        id: &UserId,
        state: SocketState,
    ) -> Result<(), GravitherError> {
        let user = self
            .users
            .get_mut(id)
            .ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        if !user.online && state == SocketState::Open {
            return Err(GravitherError::UserOffline(id.clone()));
        }
        user.outbound_socket.state = state;
        if state == SocketState::Closed {
            user.outbound_socket.target = None;
        }
        Ok(())
    }

    /// Adds information to the user's exposed/public set.
    pub fn expose_information(
        &mut self,
        id: &UserId,
        information: impl Into<String>,
    ) -> Result<(), GravitherError> {
        let user = self
            .users
            .get_mut(id)
            .ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        user.exposed_information.push(information.into());
        Ok(())
    }

    /// Removes all currently exposed information for a user.
    pub fn clear_exposed_information(&mut self, id: &UserId) -> Result<(), GravitherError> {
        let user = self
            .users
            .get_mut(id)
            .ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        user.exposed_information.clear();
        Ok(())
    }

    /// Returns active connection lines. A line exists when an observer's
    /// outbound socket points to a target whose inbound socket is open.
    pub fn connection_lines(&self) -> Vec<ConnectionLine> {
        self.users
            .values()
            .filter_map(|observer| {
                if !observer.online || observer.outbound_socket.state != SocketState::Open {
                    return None;
                }

                let target_id = observer.outbound_socket.target.as_ref()?;
                let target = self.users.get(target_id)?;

                if !target.online || target.inbound_socket.state != SocketState::Open {
                    return None;
                }

                Some(ConnectionLine {
                    source: target.id.clone(),
                    observer: observer.id.clone(),
                })
            })
            .collect()
    }

    /// Returns information flows for active lines when the source has
    /// explicitly exposed information. An empty exposed set means a line
    /// can exist without carrying information.
    pub fn information_flows(&self) -> Vec<InformationFlow> {
        self.connection_lines()
            .into_iter()
            .filter_map(|line| {
                let source = self.users.get(&line.source)?;
                if source.exposed_information.is_empty() {
                    return None;
                }
                Some(InformationFlow {
                    source: line.source,
                    observer: line.observer,
                    information: source.exposed_information.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Gravither, UserId, UserId) {
        let a = UserId::new("user-a");
        let b = UserId::new("user-b");
        (Gravither::two_users("user-a", "user-b"), a, b)
    }

    #[test]
    fn no_line_when_neither_user_observes_the_other() {
        let (system, _, _) = pair();
        assert!(system.connection_lines().is_empty());
    }

    #[test]
    fn one_way_observation_creates_one_directional_line() {
        let (mut system, a, b) = pair();
        system.observe(&a, &b).unwrap();

        let lines = system.connection_lines();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].source, b);
        assert_eq!(lines[0].observer, a);
    }

    #[test]
    fn mutual_observation_creates_two_directional_lines() {
        let (mut system, a, b) = pair();
        system.observe(&a, &b).unwrap();
        system.observe(&b, &a).unwrap();

        assert_eq!(system.connection_lines().len(), 2);
    }

    #[test]
    fn exposed_information_flows_from_observed_user_to_observer() {
        let (mut system, a, b) = pair();
        system.expose_information(&b, "public profile").unwrap();
        system.observe(&a, &b).unwrap();

        let flows = system.information_flows();
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].source, b);
        assert_eq!(flows[0].observer, a);
        assert_eq!(flows[0].information, vec!["public profile"]);
    }

    #[test]
    fn closing_inbound_socket_removes_line_but_not_other_user() {
        let (mut system, a, b) = pair();
        system.observe(&a, &b).unwrap();
        system.set_inbound_socket(&b, SocketState::Closed).unwrap();

        assert!(system.connection_lines().is_empty());
        assert!(system.user(&a).unwrap().online);
    }

    #[test]
    fn taking_user_offline_closes_both_sockets_and_removes_lines() {
        let (mut system, a, b) = pair();
        system.observe(&a, &b).unwrap();
        system.set_user_online(&b, false).unwrap();

        let offline = system.user(&b).unwrap();
        assert_eq!(offline.outbound_socket.state, SocketState::Closed);
        assert_eq!(offline.inbound_socket.state, SocketState::Closed);
        assert!(system.connection_lines().is_empty());
    }

    #[test]
    fn stopping_observation_keeps_inbound_socket_open() {
        let (mut system, a, b) = pair();
        system.observe(&a, &b).unwrap();
        system.stop_observing(&a).unwrap();

        assert!(system.connection_lines().is_empty());
        assert_eq!(
            system.user(&a).unwrap().inbound_socket.state,
            SocketState::Open
        );
    }
}
