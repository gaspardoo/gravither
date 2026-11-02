# Gravither

A minimal Rust prototype exploring the separation between **Signup** and **Life — Entity Recorder**.

## Responsibility boundary

- **Signup** collects a username, password and password confirmation. It rejects mismatched or too-short passwords and delegates account creation to Life.
- **Life — Entity Recorder** owns the credential-processing boundary, stores password hashes, creates the initial user files and records entity creation.
- **Getin** will later coordinate the linear entry flow and choose between registration and authentication. It is not implemented in this increment.
- **2FA, metrics and observation** are intentionally not implemented. Successful Signup does not start them.

These are separate responsibilities. Signup currently exists as a library component and a small CLI binary; Life is a library component with its own recorder binary.

## Run Signup

```sh
cargo run --bin signup
```

Signup asks for a username, password and confirmation. Password input is hidden by the terminal prompt. The default data directory is `.gravither-data`; set `GRAVITHER_HOME` to use another directory.

Usernames must contain 3–32 ASCII letters, digits, hyphens or underscores. Passwords must contain at least 8 bytes. Life stores an Argon2 password hash, never the plaintext password. Successful registration creates:

- `.gravither-data/users/<username>/credentials.txt`
- `.gravither-data/users/<username>/default.txt`, containing exactly `default=default`
- `.gravither-data/users/<username>/storage/`
- `.gravither-data/entities.txt`, recording entity creation

This is an early local prototype, not a production authentication system. Account storage and identity lifecycle will evolve as Getin and the remaining system services are implemented.

## Run Life's recorder

```sh
cargo run --bin gravither
```

This standalone recorder prompts for an observer-user name and appends an entity record. It does not start navigation, metrics or observation.

## Tests

```sh
cargo test
```

Tests cover password confirmation, short passwords, successful registration, password hashing/authentication, and empty-name rejection. No real network connections, sessions, 2FA, metrics, or observation processes are created.
