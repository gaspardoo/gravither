# Gravither

A minimal Rust workspace for exploring the foundations of Gravither. Each component has a narrow responsibility.

## Components

### Life — Entity Recorder

Life records the creation of observer users in a plain text file. It does not create the anchor space or implement observer behavior.

Run Life from its package:

```sh
cargo run -p life
```

Records are appended to `.gravither-data/users.txt`. Each line contains the record type and name, separated by a tab.

### Unicrypt — Cryptographic Mechanism Manager

Unicrypt is a separate Rust library for cryptographic mechanisms. Its first implementation uses Argon2 for password hashing and verification.

- `hash_password(password)` returns a salted, encoded Argon2 password hash.
- `verify_password(password, encoded_hash)` returns `true` for a match and `false` for a valid hash with a different password.
- A malformed encoded hash or a technical failure returns an error.

Argon2 password hashing is one-way; it does not encrypt data for later decryption. Unicrypt does not store passwords. SHA-256, SHA-512, reversible encryption, and other mechanisms are not implemented yet.

## Development

Run the workspace checks:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo build --workspace --all-targets --all-features
```

This is an intentionally small prototype licensed under Apache-2.0. Life and Unicrypt do not yet implement a complete identity or authentication system, nor an anchor space, navigation, groups, pages, posts, reporting, or interaction.
