//! Life — Entity Recorder and credential boundary.
//!
//! Life owns user identity and persistence. Cryptographic work is delegated to
//! Unicrypt; plaintext passwords are never written to storage.

pub mod admin;
pub mod anchor;
pub mod session;
pub mod sysruning;
pub mod user;

pub use admin::Admin;
pub use anchor::AnchorSpace;
pub use session::Session;
pub use sysruning::{LaunchPhase, LaunchStep, Program, Sysruning};
pub use user::User;

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use unicrypt::{hash_password, verify_password, UnicryptError};

const OBSERVER_USER: &str = "observer-user";

#[derive(Debug)]
pub enum LifeError {
    Io(io::Error),
    EmptyName,
    InvalidUsername,
    AccountAlreadyExists,
    AccountNotFound,
    InvalidCredentials,
    PasswordHash(String),
}

impl std::fmt::Display for LifeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "storage error: {error}"),
            Self::EmptyName => write!(f, "user name cannot be empty"),
            Self::InvalidUsername => {
                write!(f, "username must contain 3-32 ASCII letters, digits, '-' or '_'")
            }
            Self::AccountAlreadyExists => write!(f, "account already exists"),
            Self::AccountNotFound => write!(f, "account not found"),
            Self::InvalidCredentials => write!(f, "invalid username or password"),
            Self::PasswordHash(error) => write!(f, "credential hash error: {error}"),
        }
    }
}

impl std::error::Error for LifeError {}

impl From<io::Error> for LifeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug)]
pub struct Life {
    root: PathBuf,
    records_file: PathBuf,
}

impl Life {
    /// Use the supplied directory as Life's persistent data root.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            records_file: root.join("entities.txt"),
            root,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn records_file(&self) -> &Path {
        &self.records_file
    }

    fn valid_username(username: &str) -> bool {
        (3..=32).contains(&username.len())
            && username
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    }

    fn account_dir(&self, username: &str) -> Result<PathBuf, LifeError> {
        if username.is_empty() {
            return Err(LifeError::EmptyName);
        }
        if !Self::valid_username(username) {
            return Err(LifeError::InvalidUsername);
        }
        Ok(self.root.join("users").join(username))
    }

    fn existing_account_dir(&self, username: &str) -> Result<PathBuf, LifeError> {
        let dir = self.account_dir(username)?;
        match fs::metadata(&dir) {
            Ok(metadata) if metadata.is_dir() => Ok(dir),
            Ok(_) => Err(LifeError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "account path is not a directory",
            ))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Err(LifeError::AccountNotFound)
            }
            Err(error) => Err(LifeError::Io(error)),
        }
    }

    /// Create an account after Signup has confirmed the password.
    /// Unicrypt returns the encoded Argon2 hash; Life is responsible for storing it.
    pub fn create_user(&self, username: &str, password: &str) -> Result<String, LifeError> {
        let username = username.trim();
        let dir = self.account_dir(username)?;
        if password.len() < 8 {
            return Err(LifeError::InvalidCredentials);
        }

        fs::create_dir_all(self.root.join("users"))?;
        match fs::create_dir(&dir) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                return Err(LifeError::AccountAlreadyExists);
            }
            Err(error) => return Err(LifeError::Io(error)),
        }

        let result = (|| -> Result<(), LifeError> {
            let encoded_hash = hash_password(password)
                .map_err(|error| LifeError::PasswordHash(error.to_string()))?;

            fs::write(dir.join("credentials.txt"), format!("{encoded_hash}\n"))?;
            fs::write(dir.join("default.txt"), "default=default\n")?;
            fs::create_dir_all(dir.join("storage"))?;
            self.record_observer_user(username)?;
            Ok(())
        })();

        if let Err(error) = result {
            let _ = fs::remove_dir_all(&dir);
            return Err(error);
        }

        Ok(username.to_string())
    }

    /// Verify credentials through Unicrypt against the hash stored by Life.
    pub fn authenticate_user(&self, username: &str, password: &str) -> Result<String, LifeError> {
        let username = username.trim();
        let dir = self.existing_account_dir(username)?;
        let encoded_hash = fs::read_to_string(dir.join("credentials.txt"))?;

        match verify_password(password, encoded_hash.trim()) {
            Ok(true) => Ok(username.to_string()),
            Ok(false) => Err(LifeError::InvalidCredentials),
            Err(UnicryptError::InvalidEncodedHash) => Err(LifeError::PasswordHash(
                UnicryptError::InvalidEncodedHash.to_string(),
            )),
            Err(error) => Err(LifeError::PasswordHash(error.to_string())),
        }
    }

    /// Append an entity-creation record. This does not record visits or metrics.
    pub fn record_observer_user(&self, name: &str) -> Result<(), LifeError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(LifeError::EmptyName);
        }

        if let Some(parent) = self.records_file.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.records_file)?;
        writeln!(file, "{OBSERVER_USER}\t{name}")?;
        Ok(())
    }
}

/// Signup validates registration input, then delegates account creation to Life.
#[derive(Clone, Debug)]
pub struct Signup {
    life: Life,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SignupError {
    PasswordMismatch,
    PasswordTooShort,
    Life(LifeErrorKind),
}

#[derive(Debug, PartialEq, Eq)]
pub enum LifeErrorKind {
    Io,
    EmptyName,
    InvalidUsername,
    AccountAlreadyExists,
    AccountNotFound,
    InvalidCredentials,
    PasswordHash,
}

impl From<LifeError> for LifeErrorKind {
    fn from(error: LifeError) -> Self {
        match error {
            LifeError::Io(_) => Self::Io,
            LifeError::EmptyName => Self::EmptyName,
            LifeError::InvalidUsername => Self::InvalidUsername,
            LifeError::AccountAlreadyExists => Self::AccountAlreadyExists,
            LifeError::AccountNotFound => Self::AccountNotFound,
            LifeError::InvalidCredentials => Self::InvalidCredentials,
            LifeError::PasswordHash(_) => Self::PasswordHash,
        }
    }
}

impl From<LifeError> for SignupError {
    fn from(error: LifeError) -> Self {
        Self::Life(error.into())
    }
}

impl std::fmt::Display for SignupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PasswordMismatch => write!(f, "password confirmation does not match"),
            Self::PasswordTooShort => write!(f, "password must contain at least 8 bytes"),
            Self::Life(kind) => write!(f, "Life rejected registration: {kind:?}"),
        }
    }
}

impl std::error::Error for SignupError {}

impl Signup {
    pub fn new(life: Life) -> Self {
        Self { life }
    }

    pub fn register(
        &self,
        username: &str,
        password: &str,
        password_confirmation: &str,
    ) -> Result<String, SignupError> {
        if password != password_confirmation {
            return Err(SignupError::PasswordMismatch);
        }
        if password.len() < 8 {
            return Err(SignupError::PasswordTooShort);
        }
        self.life.create_user(username, password).map_err(Into::into)
    }
}

/// Getin coordinates voluntary entry. It delegates all credential work to Life.
#[derive(Clone, Debug)]
pub struct Getin {
    life: Life,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetinOutcome {
    Authenticated(String),
    Registered(String),
    SignupRequired,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetinError {
    Signup(SignupError),
    Life(LifeErrorKind),
}

impl std::fmt::Display for GetinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signup(error) => write!(f, "Signup failed: {error}"),
            Self::Life(kind) => write!(f, "Life rejected credentials: {kind:?}"),
        }
    }
}

impl std::error::Error for GetinError {}

impl Getin {
    pub fn new(life: Life) -> Self {
        Self { life }
    }

    /// Authenticate an existing account, or continue through Signup for a new one.
    ///
    /// Supplying a confirmation completes Signup only when the account is absent.
    /// A wrong password for an existing account is always rejected.
    pub fn enter(
        &self,
        username: &str,
        password: &str,
        password_confirmation: Option<&str>,
    ) -> Result<GetinOutcome, GetinError> {
        match self.life.authenticate_user(username, password) {
            Ok(username) => Ok(GetinOutcome::Authenticated(username)),
            Err(LifeError::AccountNotFound) => match password_confirmation {
                None => Ok(GetinOutcome::SignupRequired),
                Some(confirmation) => Signup::new(self.life.clone())
                    .register(username, password, confirmation)
                    .map(GetinOutcome::Registered)
                    .map_err(GetinError::Signup),
            },
            Err(error) => Err(GetinError::Life(error.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn temp_root() -> PathBuf {
        let number = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "gravither-identity-test-{}-{number}",
            std::process::id()
        ))
    }

    #[test]
    fn signup_confirms_password_then_delegates_to_life() {
        let root = temp_root();
        let life = Life::new(&root);
        let signup = Signup::new(life.clone());

        assert_eq!(
            signup
                .register("Alice", "correct-horse", "correct-horse")
                .unwrap(),
            "Alice"
        );
        assert!(root.join("users/Alice/credentials.txt").exists());
        assert_eq!(
            fs::read_to_string(root.join("users/Alice/default.txt")).unwrap(),
            "default=default\n"
        );
        assert_eq!(
            fs::read_to_string(life.records_file()).unwrap(),
            "observer-user\tAlice\n"
        );
        assert_eq!(
            life.authenticate_user("Alice", "correct-horse").unwrap(),
            "Alice"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn signup_rejects_mismatched_password_without_creating_user() {
        let root = temp_root();
        let signup = Signup::new(Life::new(&root));

        assert_eq!(
            signup.register("Alice", "correct-horse", "different-password"),
            Err(SignupError::PasswordMismatch)
        );
        assert!(!root.join("users/Alice").exists());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn signup_rejects_short_password() {
        let root = temp_root();
        let signup = Signup::new(Life::new(&root));

        assert_eq!(
            signup.register("Alice", "short", "short"),
            Err(SignupError::PasswordTooShort)
        );
        assert!(!root.join("users/Alice").exists());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn signup_rejects_duplicate_account() {
        let root = temp_root();
        let signup = Signup::new(Life::new(&root));
        signup
            .register("Alice", "correct-horse", "correct-horse")
            .unwrap();

        assert!(matches!(
            signup.register("Alice", "another-password", "another-password"),
            Err(SignupError::Life(LifeErrorKind::AccountAlreadyExists))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn getin_authenticates_existing_user_through_life() {
        let root = temp_root();
        let life = Life::new(&root);
        Signup::new(life.clone())
            .register("Alice", "correct-horse", "correct-horse")
            .unwrap();
        let getin = Getin::new(life);

        assert_eq!(
            getin.enter("Alice", "correct-horse", None).unwrap(),
            GetinOutcome::Authenticated("Alice".to_string())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn getin_routes_unknown_user_to_signup_with_original_values() {
        let root = temp_root();
        let getin = Getin::new(Life::new(&root));

        assert_eq!(
            getin.enter("Alice", "correct-horse", None).unwrap(),
            GetinOutcome::SignupRequired
        );
        assert_eq!(
            getin
                .enter("Alice", "correct-horse", Some("correct-horse"))
                .unwrap(),
            GetinOutcome::Registered("Alice".to_string())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn getin_never_routes_a_wrong_password_to_signup() {
        let root = temp_root();
        let life = Life::new(&root);
        Signup::new(life.clone())
            .register("Alice", "correct-horse", "correct-horse")
            .unwrap();
        let getin = Getin::new(life);

        assert_eq!(
            getin.enter("Alice", "wrong-password", Some("wrong-password")),
            Err(GetinError::Life(LifeErrorKind::InvalidCredentials))
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_stored_hash_is_not_treated_as_a_wrong_password() {
        let root = temp_root();
        let account = root.join("users/Alice");
        fs::create_dir_all(&account).unwrap();
        fs::write(account.join("credentials.txt"), "not-a-phc-hash\n").unwrap();
        let getin = Getin::new(Life::new(&root));

        assert_eq!(
            getin.enter("Alice", "some-password", None),
            Err(GetinError::Life(LifeErrorKind::PasswordHash))
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn incomplete_account_is_not_routed_to_signup() {
        let root = temp_root();
        fs::create_dir_all(root.join("users/Alice")).unwrap();
        let getin = Getin::new(Life::new(&root));

        assert!(matches!(
            getin.enter("Alice", "some-password", None),
            Err(GetinError::Life(LifeErrorKind::Io))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recorder_rejects_empty_name() {
        let root = temp_root();
        let life = Life::new(&root);

        assert!(matches!(
            life.record_observer_user("  "),
            Err(LifeError::EmptyName)
        ));
        assert!(!life.records_file().exists());
        fs::remove_dir_all(root).ok();
    }
}
