# Gravither

A minimal Rust prototype for Gravither's first connection model.

## First implementation

Each user has two independent logical sockets:

- **Outbound socket:** used to observe and navigate to another user.
- **Inbound socket:** receives observation connections and exposes only information explicitly marked as public.

An observation from A to B creates an information path from B to A: B is the source of exposed information, and A is the observer.

Mutual observation creates two directional lines. If neither user observes the other, no line exists.

This version models socket state in-process. It does not yet open operating-system TCP/UDP sockets or send data over a network.

## Run

```sh
cargo run
cargo test
```

The initial permission file is `default.txt` and contains only:

```text
default=default
```
