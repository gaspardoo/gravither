//! Gravither prototype: Life entity recorder and minimal process contexts.
//!
//! Life records the creation of observer users. It does not create an anchor
//! space or implement user behavior.

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

/// The only kind of record Life can create for now.
const OBSERVER_USER: &str = "observer-user";

#[derive(Debug)]
pub enum LifeError {
    Io(io::Error),
    EmptyName,
}

impl std::fmt::Display for LifeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "storage error: {error}"),
            Self::EmptyName => write!(f, "user name cannot be empty"),
        }
    }
}

impl std::error::Error for LifeError {}

impl From<io::Error> for LifeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Life's simple text-file recorder.
pub struct Life {
    file: PathBuf,
}

impl Life {
    /// Create a recorder that stores records in the given file.
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self { file: file.into() }
    }

    /// Record one observer user by name.
    ///
    /// Each line contains the record type and the supplied name.
    pub fn record_observer_user(&self, name: &str) -> Result<(), LifeError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(LifeError::EmptyName);
        }

        if let Some(parent) = self.file.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)?;

        writeln!(file, "{OBSERVER_USER}\t{name}")?;
        Ok(())
    }

    /// Return the path where records are kept.
    pub fn file_path(&self) -> &Path {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn temp_file() -> PathBuf {
        let number = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "gravither-life-test-{}-{number}.txt",
            std::process::id()
        ))
    }

    #[test]
    fn records_an_observer_user() {
        let path = temp_file();
        let life = Life::new(&path);

        life.record_observer_user("Alice").unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "observer-user\tAlice\n"
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn keeps_multiple_records() {
        let path = temp_file();
        let life = Life::new(&path);

        life.record_observer_user("Alice").unwrap();
        life.record_observer_user("Bob").unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "observer-user\tAlice\nobserver-user\tBob\n"
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_an_empty_name() {
        let path = temp_file();
        let life = Life::new(&path);

        assert!(matches!(
            life.record_observer_user("   "),
            Err(LifeError::EmptyName)
        ));
        assert!(!path.exists());
    }
}
