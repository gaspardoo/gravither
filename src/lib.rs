//! Life — Entity Recorder.
//!
//! Life owns the credential-processing boundary and records entity creation.
//! Signup is a separate registration flow that validates confirmation before
//! delegating account creation to Life. No sessions, 2FA, metrics or observation
//! processes are started here.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

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
            Self::InvalidUsername => write!(f, "username must contain 3-32 ASCII letters, digits, '-' or '_'"),
            Self::AccountAlreadyExists => write!(f, "account already exists"),
            Self::AccountNotFound => write!(f, "account not found"),
            Self::InvalidCredentials => write!(f, "invalid username or password"),
            Self::PasswordHash(error) => write!(f, "password hashing error: {error}"),
        }
    }
}

impl std::error::Error for LifeError {}
impl From<io::Error> for LifeError {
    fn from(error: io::Error) -> Self { Self::Io(error) }
}

/// Life's entity recorder and credential-processing boundary.
#[derive(Clone, Debug)]
pub struct Life {
    root: PathBuf,
    records_file: PathBuf,
}

impl Life {
    /// Store accounts and entity records under the supplied data directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self { records_file: root.join("entities.txt"), root }
    }

    pub fn root(&self) -> &Path { &self.root }
    pub fn records_file(&self) -> &Path { &self.records_file }

    fn valid_username(username: &str) -> bool {
        (3..=32).contains(&username.len())
            && username.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    }

    fn account_dir(&self, username: &str) -> Result<PathBuf, LifeError> {
        if !Self::valid_username(username) { return Err(LifeError::InvalidUsername); }
        Ok(self.root.join("users").join(username))
    }

    /// Create an account after Signup has confirmed the password.
    /// The password is persisted only as an Argon2 hash, never as plaintext.
    pub fn create_user(&self, username: &str, password: &str) -> Result<String, LifeError> {
        let username = username.trim();
        if username.is_empty() { return Err(LifeError::EmptyName); }
        if password.len() < 8 { return Err(LifeError::InvalidCredentials); }

        let dir = self.account_dir(username)?;
        if dir.exists() { return Err(LifeError::AccountAlreadyExists); }
        fs::create_dir_all(&dir)?;

        let result = (|| -> Result<(), LifeError> {
            let salt = SaltString::generate(&mut OsRng);
            let hash = Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|error| LifeError::PasswordHash(error.to_string()))?
                .to_string();
            fs::write(dir.join("credentials.txt"), format!("{hash}\n"))?;
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

    /// Authenticate credentials against the hash stored by Life.
    pub fn authenticate_user(&self, username: &str, password: &str) -> Result<String, LifeError> {
        let username = username.trim();
        let dir = self.account_dir(username)?;
        let encoded = fs::read_to_string(dir.join("credentials.txt")).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound { LifeError::AccountNotFound }
            else { LifeError::Io(error) }
        })?;
        let parsed = PasswordHash::new(encoded.trim()).map_err(|_| LifeError::InvalidCredentials)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| LifeError::InvalidCredentials)?;
        Ok(username.to_string())
    }

    /// Append one entity record. This records entity creation, not visits or metrics.
    pub fn record_observer_user(&self, name: &str) -> Result<(), LifeError> {
        let name = name.trim();
        if name.is_empty() { return Err(LifeError::EmptyName); }

        if let Some(parent) = self.records_file.parent() {
            if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; }
        }
        let mut file = OpenOptions::new().create(true).append(true).open(&self.records_file)?;
        writeln!(file, "{OBSERVER_USER}\t{name}")?;
        Ok(())
    }
}

/// Signup validates the registration form, then delegates credential handling
/// and entity creation to Life. It does not authenticate existing users.
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

impl From<LifeError> for SignupError {
    fn from(error: LifeError) -> Self {
        let kind = match error {
            LifeError::Io(_) => LifeErrorKind::Io,
            LifeError::EmptyName => LifeErrorKind::EmptyName,
            LifeError::InvalidUsername => LifeErrorKind::InvalidUsername,
            LifeError::AccountAlreadyExists => LifeErrorKind::AccountAlreadyExists,
            LifeError::AccountNotFound => LifeErrorKind::AccountNotFound,
            LifeError::InvalidCredentials => LifeErrorKind::InvalidCredentials,
            LifeError::PasswordHash(_) => LifeErrorKind::PasswordHash,
        };
        Self::Life(kind)
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
    pub fn new(life: Life) -> Self { Self { life } }

    /// Register a user after confirming that both password fields match.
    pub fn register(
        &self,
        username: &str,
        password: &str,
        password_confirmation: &str,
    ) -> Result<String, SignupError> {
        if password != password_confirmation { return Err(SignupError::PasswordMismatch); }
        if password.len() < 8 { return Err(SignupError::PasswordTooShort); }
        self.life.create_user(username, password).map_err(Into::into)
    }
}


/// Getin coordinates the initial entry flow. It never hashes or decrypts
/// credentials itself; Life verifies stored password hashes.
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
    pub fn new(life: Life) -> Self { Self { life } }

    /// Check existing credentials. If the account does not exist, return
    /// SignupRequired unless a confirmation was supplied, then delegate to Signup.
    /// No session, 2FA, metrics or observation process is started here.
    pub fn enter(
        &self,
        username: &str,
        password: &str,
        password_confirmation: Option<&str>,
    ) -> Result<GetinOutcome, GetinError> {
        match self.life.authenticate_user(username, password) {
            Ok(username) => Ok(GetinOutcome::Authenticated(username)),
            Err(LifeError::AccountNotFound) => {
                match password_confirmation {
                    None => Ok(GetinOutcome::SignupRequired),
                    Some(confirmation) => Signup::new(self.life.clone())
                        .register(username, password, confirmation)
                        .map(GetinOutcome::Registered)
                        .map_err(GetinError::Signup),
                }
            }
            Err(LifeError::Io(_)) => Err(GetinError::Life(LifeErrorKind::Io)),
            Err(LifeError::EmptyName) => Err(GetinError::Life(LifeErrorKind::EmptyName)),
            Err(LifeError::InvalidUsername) => Err(GetinError::Life(LifeErrorKind::InvalidUsername)),
            Err(LifeError::AccountAlreadyExists) => Err(GetinError::Life(LifeErrorKind::AccountAlreadyExists)),
            Err(LifeError::InvalidCredentials) => Err(GetinError::Life(LifeErrorKind::InvalidCredentials)),
            Err(LifeError::PasswordHash(_)) => Err(GetinError::Life(LifeErrorKind::PasswordHash)),
            Err(LifeError::AccountNotFound) => unreachable!("handled above"),
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
        std::env::temp_dir().join(format!("gravither-signup-test-{}-{number}", std::process::id()))
    }

    #[test]
    fn signup_confirms_password_then_delegates_to_life() {
        let root = temp_root();
        let life = Life::new(&root);
        let signup = Signup::new(life.clone());

        assert_eq!(
            signup.register("Alice", "correct-horse", "correct-horse").unwrap(),
            "Alice"
        );
        assert!(root.join("users/Alice/credentials.txt").exists());
        assert_eq!(fs::read_to_string(root.join("users/Alice/default.txt")).unwrap(), "default=default\n");
        assert_eq!(fs::read_to_string(life.records_file()).unwrap(), "observer-user\tAlice\n");
        assert_eq!(life.authenticate_user("Alice", "correct-horse").unwrap(), "Alice");
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
    fn life_authentication_rejects_wrong_password() {
        let root = temp_root();
        let life = Life::new(&root);
        Signup::new(life.clone()).register("Alice", "correct-horse", "correct-horse").unwrap();
        assert!(matches!(life.authenticate_user("Alice", "wrong-password"), Err(LifeError::InvalidCredentials)));
        fs::remove_dir_all(root).unwrap();
    }


    #[test]
    fn getin_authenticates_existing_user_through_life() {
        let root = temp_root();
        let life = Life::new(&root);
        Signup::new(life.clone()).register("Alice", "correct-horse", "correct-horse").unwrap();
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
        assert_eq!(getin.enter("Alice", "correct-horse", None).unwrap(), GetinOutcome::SignupRequired);
        assert_eq!(
            getin.enter("Alice", "correct-horse", Some("correct-horse")).unwrap(),
            GetinOutcome::Registered("Alice".to_string())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn getin_does_not_route_wrong_password_to_signup() {
        let root = temp_root();
        let life = Life::new(&root);
        Signup::new(life.clone()).register("Alice", "correct-horse", "correct-horse").unwrap();
        let getin = Getin::new(life);
        assert_eq!(
            getin.enter("Alice", "wrong-password", None),
            Err(GetinError::Life(LifeErrorKind::InvalidCredentials))
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn getin_rejects_mismatched_confirmation_for_new_user() {
        let root = temp_root();
        let getin = Getin::new(Life::new(&root));
        assert_eq!(
            getin.enter("Alice", "correct-horse", Some("different-password")),
            Err(GetinError::Signup(SignupError::PasswordMismatch))
        );
        assert!(!root.join("users/Alice").exists());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn recorder_rejects_empty_name() {
        let root = temp_root();
        let life = Life::new(&root);
        assert!(matches!(life.record_observer_user("  "), Err(LifeError::EmptyName)));
        assert!(!life.records_file().exists());
        fs::remove_dir_all(root).ok();
    }
}
