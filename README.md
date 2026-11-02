# Gravither

A minimal Rust prototype exploring separate responsibilities for **Getin**, **Signup**, and **Life — Entity Recorder**.

## Responsibility boundary

- **Getin** coordinates entry. It asks for a username and password, routes unknown users to Signup, and uses Life to verify existing credentials.
- **Signup** validates password confirmation and delegates account creation to Life. The same entered username and password can be carried through the registration branch without asking for them again.
- **Life — Entity Recorder** owns credential processing, stores password hashes, creates initial user files and records entity creation.
- **2FA, metrics and observation** are intentionally not implemented. Getin returns after successful authentication or registration; it does not start those processes or a long-running user session.

Gravither is an operating-system concept, not a social network with a forced login gate. A future feed can remain navigable before entry; Getin will indicate the visitor has not entered and cannot yet participate. That feed UI is not part of this terminal prototype.

## Run Getin

```sh
cargo run --bin getin
```

Getin asks for a username and password. If the account exists, Life verifies the password hash. If no account exists, Getin carries the entered values into the Signup flow and asks only for password confirmation.

An incorrect password for an existing account is rejected; it does not silently route to Signup.

## Run Signup directly

```sh
cargo run --bin signup
```

Signup asks for username, password and confirmation.

## Life recorder

```sh
cargo run --bin gravither
```

This standalone recorder prompts for an observer-user name and appends an entity record. It does not start navigation, metrics or observation.

## Credential storage

The default data directory is `.gravither-data`; set `GRAVITHER_HOME` to use another directory.

Usernames must contain 3–32 ASCII letters, digits, hyphens or underscores. Passwords must contain at least 8 bytes. Life stores an Argon2 password hash, never plaintext. Successful registration creates:

- `.gravither-data/users/<username>/credentials.txt`
- `.gravither-data/users/<username>/default.txt`, containing exactly `default=default`
- `.gravither-data/users/<username>/storage/`
- `.gravither-data/entities.txt`, recording entity creation

**Hashing clarification:** password hashes are one-way, not encrypted passwords. Getin does not decrypt credentials; Life verifies the supplied password against the stored Argon2 hash.

This is an early local prototype, not a production authentication system.

## Tests

```sh
cargo test
```

Tests cover Signup confirmation and creation, credential verification, Getin routing for existing and unknown users, wrong-password rejection, and empty-name handling. No real network connections, sessions, 2FA, metrics or observation processes are created.
