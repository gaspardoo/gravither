# Gravither

A minimal Rust prototype exploring separate responsibilities for **Getin**, **Signup**, and **Life — Entity Recorder**, with cryptographic operations delegated to **Unicrypt**.

## Responsibility boundary

- **Getin** coordinates entry and chooses between authentication and registration.
- **Signup** validates password confirmation, then delegates account creation to Life.
- **Life — Entity Recorder** records entity creation, stores account files, and asks Unicrypt to hash or verify passwords. Life does not implement a cryptographic algorithm itself.
- **Unicrypt — Cryptographic Mechanism Manager** is a separate Rust library. For now it implements password hashing and verification using Argon2.
- **2FA, metrics and observation** are intentionally not implemented.

Gravither is an operating-system concept, not a social network with a forced login gate. A future feed can remain navigable before entry; that feed UI is not part of this terminal prototype.

## Run programs

Run Getin (when its branch is included):

```sh
cargo run --bin getin
```

Run Signup directly:

```sh
cargo run --bin signup
```

Run Life's standalone recorder:

```sh
cargo run --bin gravither
```

The default data directory is `.gravither-data`; set `GRAVITHER_HOME` to use another directory. Usernames must contain 3–32 ASCII letters, digits, hyphens or underscores. Passwords must contain at least 8 bytes.

## Unicrypt

The library is located in `unicrypt/`.

- `hash_password(password)` creates an encoded Argon2 hash with a cryptographically secure random salt.
- `verify_password(password, encoded_hash)` returns `true` for a match and `false` for a mismatch.
- Malformed hashes and technical failures are reported as errors.
- Unicrypt never stores the original password.

Argon2 password hashing is one-way; it does not encrypt data for later decryption. SHA-256, SHA-512, and reversible encryption are not implemented yet.

## Tests

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo build --workspace --all-targets --all-features
```

This is an early local prototype, not a production authentication system. No real network connections, long-running sessions, 2FA, metrics, or observation processes are created.
