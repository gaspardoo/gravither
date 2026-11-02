# Gravither

A minimal Rust prototype exploring separate responsibilities for **Getin**, **Signup**, **Life — Entity Recorder**, and **Unicrypt**.

## Expected credential flow

- **Getin** coordinates voluntary entry and decides whether to authenticate an existing user or continue through Signup.
- **Signup** validates the password confirmation and delegates account creation to Life.
- **Life** owns identity, account files, default user data, and entity-creation records. It delegates hashing and verification to Unicrypt.
- **Unicrypt** hashes passwords with Argon2 and verifies a supplied password against a stored hash. Passwords are never stored as plaintext.

An unknown account can proceed to Signup. An incorrect password for an existing account is rejected and never routed into registration. Missing or malformed credential files are treated as storage/integrity errors, not as evidence that an account does not exist.

## Run the programs

    cargo run --bin getin
    cargo run --bin signup
    cargo run --bin gravither

- **getin** asks for a username and password. Existing users are authenticated; unknown users are routed to Signup and asked only to confirm the password.
- **signup** starts registration directly.
- **gravither** runs Life's standalone entity recorder.

Set the GRAVITHER_HOME environment variable to change the persistent data directory. The default is .gravither-data.

## Account data

Usernames must contain 3–32 ASCII letters, digits, hyphens, or underscores. Passwords must contain at least 8 bytes. Registration creates:

- .gravither-data/users/<username>/credentials.txt — an encoded Argon2 hash, never plaintext.
- .gravither-data/users/<username>/default.txt — initial default value.
- .gravither-data/users/<username>/storage/ — user storage directory.
- .gravither-data/entities.txt — entity-creation records.

This is an early local prototype, not a production authentication system. Sessions, 2FA, metrics, navigation, and observation are not started by successful entry yet.

## Quality checks

    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-targets --all-features
    cargo build --all-targets --all-features
