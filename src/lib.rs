//! Gravither prototype: portal, user process, common-space process and observation service.
//!
//! All components are logical processes hosted in one Rust process for this experiment.
//! Sockets are in-memory models; no network transport is opened.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState { Open, Closed }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundSocket {
    pub state: SocketState,
    pub target: Option<UserId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundSocket { pub state: SocketState }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub online: bool,
    pub outbound_socket: OutboundSocket,
    pub inbound_socket: InboundSocket,
    pub exposed_information: Vec<String>,
}

impl User {
    pub fn new(id: UserId) -> Self {
        Self {
            id,
            online: true,
            outbound_socket: OutboundSocket { state: SocketState::Open, target: None },
            inbound_socket: InboundSocket { state: SocketState::Open },
            exposed_information: Vec::new(),
        }
    }

    fn set_online(&mut self, online: bool) {
        self.online = online;
        self.outbound_socket.state = if online { SocketState::Open } else { SocketState::Closed };
        self.inbound_socket.state = if online { SocketState::Open } else { SocketState::Closed };
        if !online { self.outbound_socket.target = None; }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionLine { pub source: UserId, pub observer: UserId }

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

pub struct Gravither { users: BTreeMap<UserId, User> }

impl Gravither {
    pub fn new() -> Self { Self { users: BTreeMap::new() } }

    /// Compatibility helper for the original two-user experiment.
    pub fn two_users(first: impl Into<String>, second: impl Into<String>) -> Self {
        let mut system = Self::new();
        system.add_user(UserId::new(first));
        system.add_user(UserId::new(second));
        system
    }

    pub fn add_user(&mut self, id: UserId) -> bool {
        if self.users.contains_key(&id) { return false; }
        self.users.insert(id.clone(), User::new(id));
        true
    }

    pub fn users(&self) -> impl Iterator<Item = &User> { self.users.values() }

    pub fn user(&self, id: &UserId) -> Option<&User> { self.users.get(id) }

    pub fn set_user_online(&mut self, id: &UserId, online: bool) -> Result<(), GravitherError> {
        let user = self.users.get_mut(id).ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        user.set_online(online);
        if !online {
            for other in self.users.values_mut() {
                if other.outbound_socket.target.as_ref() == Some(id) {
                    other.outbound_socket.target = None;
                }
            }
        }
        Ok(())
    }

    pub fn observe(&mut self, observer_id: &UserId, target_id: &UserId) -> Result<(), GravitherError> {
        if observer_id == target_id { return Err(GravitherError::UserCannotObserveItself); }
        let observer = self.users.get(observer_id).ok_or_else(|| GravitherError::UnknownUser(observer_id.clone()))?;
        if !observer.online { return Err(GravitherError::UserOffline(observer_id.clone())); }
        if observer.outbound_socket.state == SocketState::Closed {
            return Err(GravitherError::OutboundSocketClosed(observer_id.clone()));
        }
        let target = self.users.get(target_id).ok_or_else(|| GravitherError::UnknownUser(target_id.clone()))?;
        if !target.online { return Err(GravitherError::UserOffline(target_id.clone())); }
        if target.inbound_socket.state == SocketState::Closed {
            return Err(GravitherError::InboundSocketClosed(target_id.clone()));
        }
        self.users.get_mut(observer_id).expect("observer checked").outbound_socket.target = Some(target_id.clone());
        Ok(())
    }

    pub fn stop_observing(&mut self, observer_id: &UserId) -> Result<(), GravitherError> {
        let observer = self.users.get_mut(observer_id).ok_or_else(|| GravitherError::UnknownUser(observer_id.clone()))?;
        observer.outbound_socket.target = None;
        Ok(())
    }

    pub fn set_inbound_socket(&mut self, id: &UserId, state: SocketState) -> Result<(), GravitherError> {
        let user = self.users.get_mut(id).ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        if !user.online && state == SocketState::Open { return Err(GravitherError::UserOffline(id.clone())); }
        user.inbound_socket.state = state;
        Ok(())
    }

    pub fn set_outbound_socket(&mut self, id: &UserId, state: SocketState) -> Result<(), GravitherError> {
        let user = self.users.get_mut(id).ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        if !user.online && state == SocketState::Open { return Err(GravitherError::UserOffline(id.clone())); }
        user.outbound_socket.state = state;
        if state == SocketState::Closed { user.outbound_socket.target = None; }
        Ok(())
    }

    pub fn expose_information(&mut self, id: &UserId, information: impl Into<String>) -> Result<(), GravitherError> {
        let user = self.users.get_mut(id).ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        user.exposed_information.push(information.into());
        Ok(())
    }

    pub fn clear_exposed_information(&mut self, id: &UserId) -> Result<(), GravitherError> {
        let user = self.users.get_mut(id).ok_or_else(|| GravitherError::UnknownUser(id.clone()))?;
        user.exposed_information.clear();
        Ok(())
    }

    pub fn connection_lines(&self) -> Vec<ConnectionLine> {
        self.users.values().filter_map(|observer| {
            if !observer.online || observer.outbound_socket.state != SocketState::Open { return None; }
            let target = self.users.get(observer.outbound_socket.target.as_ref()?)?;
            if !target.online || target.inbound_socket.state != SocketState::Open { return None; }
            Some(ConnectionLine { source: target.id.clone(), observer: observer.id.clone() })
        }).collect()
    }

    pub fn information_flows(&self) -> Vec<InformationFlow> {
        self.connection_lines().into_iter().filter_map(|line| {
            let source = self.users.get(&line.source)?;
            if source.exposed_information.is_empty() { return None; }
            Some(InformationFlow {
                source: line.source,
                observer: line.observer,
                information: source.exposed_information.clone(),
            })
        }).collect()
    }
}

impl Default for Gravither { fn default() -> Self { Self::new() } }

#[derive(Debug)]
pub enum PortalError {
    Io(std::io::Error),
    InvalidUsername,
    AccountAlreadyExists,
    AccountNotFound,
    InvalidCredentials,
    PasswordHash(String),
    InvalidSessionState,
}

impl std::fmt::Display for PortalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::InvalidUsername => write!(f, "username must contain 3-32 letters, digits, '-' or '_'"),
            Self::AccountAlreadyExists => write!(f, "account already exists"),
            Self::AccountNotFound => write!(f, "account not found"),
            Self::InvalidCredentials => write!(f, "invalid username or password"),
            Self::PasswordHash(e) => write!(f, "password hashing error: {e}"),
            Self::InvalidSessionState => write!(f, "invalid session state"),
        }
    }
}
impl std::error::Error for PortalError {}
impl From<std::io::Error> for PortalError { fn from(value: std::io::Error) -> Self { Self::Io(value) } }

/// Entry point for account creation, authentication and session startup.
pub struct Portal { root: PathBuf }

impl Portal {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    pub fn root(&self) -> &Path { &self.root }

    fn valid_username(username: &str) -> bool {
        (3..=32).contains(&username.len())
            && username.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    }

    fn account_dir(&self, username: &str) -> Result<PathBuf, PortalError> {
        if !Self::valid_username(username) { return Err(PortalError::InvalidUsername); }
        Ok(self.root.join("users").join(username))
    }

    pub fn register(&self, username: &str, password: &str) -> Result<UserId, PortalError> {
        if password.len() < 8 { return Err(PortalError::InvalidCredentials); }
        let dir = self.account_dir(username)?;
        if dir.exists() { return Err(PortalError::AccountAlreadyExists); }
        fs::create_dir_all(&dir)?;
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default().hash_password(password.as_bytes(), &salt)
            .map_err(|e| PortalError::PasswordHash(e.to_string()))?.to_string();
        fs::write(dir.join("credentials.txt"), format!("{hash}\n"))?;
        fs::write(dir.join("default.txt"), "default=default\n")?;
        fs::create_dir_all(dir.join("storage"))?;
        fs::write(dir.join("activity.log"), "Gravither user activity log\n")?;
        Ok(UserId::new(username))
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<UserId, PortalError> {
        let dir = self.account_dir(username)?;
        let encoded = fs::read_to_string(dir.join("credentials.txt")).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound { PortalError::AccountNotFound } else { PortalError::Io(e) }
        })?;
        let parsed = PasswordHash::new(encoded.trim()).map_err(|_| PortalError::InvalidCredentials)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed)
            .map_err(|_| PortalError::InvalidCredentials)?;
        Ok(UserId::new(username))
    }

    pub fn launch_session(&self, user_id: UserId, device_id: impl Into<String>) -> Result<RuntimeSession, PortalError> {
        let dir = self.account_dir(user_id.as_str())?;
        if !dir.is_dir() { return Err(PortalError::AccountNotFound); }

        // Startup sequence: user process -> common-space process -> observation service.
        let mut user_process = UserProcess::start(user_id.clone(), dir)?;
        user_process.record_activity("user process started")?;

        let mut common_space = CommonSpaceProcess::start("common".to_string());
        common_space.discover(user_id.clone());
        user_process.record_activity("common space process started")?;

        let observation = ObservationService::start(device_id.into(), common_space.id.clone());
        user_process.record_activity("observation service started; navigation control handed off")?;

        Ok(RuntimeSession { user_process, common_space, observation })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState { Running, Stopped }

pub struct UserProcess {
    pub user_id: UserId,
    pub state: ProcessState,
    /// Incoming observation endpoint belongs to the user process, not the observer service.
    pub exposure_socket: SocketState,
    storage_dir: PathBuf,
    activity_log: PathBuf,
}

impl UserProcess {
    fn start(user_id: UserId, account_dir: PathBuf) -> Result<Self, PortalError> {
        Ok(Self {
            user_id,
            state: ProcessState::Running,
            exposure_socket: SocketState::Open,
            storage_dir: account_dir.join("storage"),
            activity_log: account_dir.join("activity.log"),
        })
    }

    pub fn record_activity(&mut self, activity: &str) -> Result<(), PortalError> {
        if self.state != ProcessState::Running { return Err(PortalError::InvalidSessionState); }
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let mut file = OpenOptions::new().create(true).append(true).open(&self.activity_log)?;
        writeln!(file, "{timestamp}\t{activity}")?;
        Ok(())
    }

    pub fn storage_dir(&self) -> &Path { &self.storage_dir }
}

pub struct CommonSpaceProcess {
    pub id: String,
    pub state: ProcessState,
    entities: BTreeSet<UserId>,
}

impl CommonSpaceProcess {
    fn start(id: String) -> Self {
        Self { id, state: ProcessState::Running, entities: BTreeSet::new() }
    }
    pub fn discover(&mut self, entity: UserId) { self.entities.insert(entity); }
    pub fn entities(&self) -> impl Iterator<Item = &UserId> { self.entities.iter() }
}

pub struct ObservationService {
    pub device_id: String,
    pub state: ProcessState,
    pub observer_socket: SocketState,
    pub common_space_id: String,
}

impl ObservationService {
    fn start(device_id: String, common_space_id: String) -> Self {
        Self { device_id, state: ProcessState::Running, observer_socket: SocketState::Open, common_space_id }
    }
    pub fn stop(&mut self) {
        self.observer_socket = SocketState::Closed;
        self.state = ProcessState::Stopped;
    }
}

/// Session coordinator. Stopping navigation does not stop the user process.
pub struct RuntimeSession {
    pub user_process: UserProcess,
    pub common_space: CommonSpaceProcess,
    pub observation: ObservationService,
}

impl RuntimeSession {
    pub fn stop_navigation(&mut self) -> Result<(), PortalError> {
        self.observation.stop();
        self.common_space.state = ProcessState::Stopped;
        self.user_process.record_activity("navigation stopped; user process remains active")
    }

    pub fn logout(&mut self) -> Result<(), PortalError> {
        self.observation.stop();
        self.common_space.state = ProcessState::Stopped;
        self.user_process.record_activity("user logged out")?;
        self.user_process.exposure_socket = SocketState::Closed;
        self.user_process.state = ProcessState::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp_root() -> PathBuf {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("gravither-test-{}-{n}", std::process::id()))
    }

    fn pair() -> (Gravither, UserId, UserId) {
        let a = UserId::new("user-a");
        let b = UserId::new("user-b");
        (Gravither::two_users("user-a", "user-b"), a, b)
    }

    #[test]
    fn supports_more_than_two_users() {
        let mut system = Gravither::new();
        for name in ["a-user", "b-user", "c-user"] { assert!(system.add_user(UserId::new(name))); }
        assert_eq!(system.users().count(), 3);
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
    fn closing_inbound_socket_removes_line() {
        let (mut system, a, b) = pair();
        system.observe(&a, &b).unwrap();
        system.set_inbound_socket(&b, SocketState::Closed).unwrap();
        assert!(system.connection_lines().is_empty());
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
        assert_eq!(system.user(&a).unwrap().inbound_socket.state, SocketState::Open);
    }

    #[test]
    fn portal_registers_authenticates_and_starts_runtime() {
        let root = temp_root();
        let portal = Portal::new(&root);
        let id = portal.register("alice_01", "correct-horse-battery").unwrap();
        assert_eq!(portal.authenticate("alice_01", "correct-horse-battery").unwrap(), id);
        assert!(matches!(portal.authenticate("alice_01", "wrong-password"), Err(PortalError::InvalidCredentials)));
        let mut session = portal.launch_session(id, "device-1").unwrap();
        assert_eq!(session.user_process.state, ProcessState::Running);
        assert_eq!(session.common_space.state, ProcessState::Running);
        assert_eq!(session.observation.observer_socket, SocketState::Open);
        session.stop_navigation().unwrap();
        assert_eq!(session.user_process.state, ProcessState::Running);
        assert_eq!(session.observation.state, ProcessState::Stopped);
        session.logout().unwrap();
        assert_eq!(session.user_process.state, ProcessState::Stopped);
        fs::remove_dir_all(root).unwrap();
    }
}
