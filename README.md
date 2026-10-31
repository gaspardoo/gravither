# Gravither

A small Rust prototype for experimenting with Gravither's startup and observation concepts. The directory layout is provisional and may change as the model evolves.

## Startup model

The portal performs account registration or authentication, then starts these logical components in order:

1. **User process** — represents the authenticated identity, provides storage and appends an activity log.
2. **Common-space process** — represents the initial shared navigation context and tracks discoverable entities.
3. **Observation service** — receives navigation control and owns one logical observer socket per device.

These components currently run inside a single Rust process. They are separate logical responsibilities, not operating-system child processes. The user process remains active when navigation stops. Logging out stops the observation service and common-space process, then stops the user process without deleting the account.

## Try it

Run cargo run to start the portal, or cargo test to run the test suite.

The portal offers sign-in for an existing local account and identity creation for a new local account. After startup, use the commands status, stop-navigation, and logout.

Accounts are stored under .gravither-data/users/<username> by default. Set GRAVITHER_HOME to use another data directory. Usernames are restricted to 3–32 ASCII letters, digits, hyphens or underscores. Passwords must contain at least 8 bytes and are stored as Argon2 password hashes, not plaintext. This is a local prototype, not a production authentication system.

## Observation model

An entity has two independent logical sockets:

- **Observer socket:** used by an observation service on a device to observe another entity.
- **Exposure socket:** belongs to the user process and accepts incoming observation when enabled.

An observation from A to B allows exposed information to flow B → A. Mutual observation produces two directional lines. The common space enables discovery but does not automatically connect every entity.

The Gravither model supports a dynamic collection of users. Socket states and information flows remain in memory; no TCP/UDP transport is implemented yet.

## Permissions

The initial default permission file contains exactly:

    default=default

This prototype does not yet implement property ownership, encryption, advanced permissions, transfer protocols, or real network connections.
