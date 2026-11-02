# Gravither

A minimal Rust experiment that starts with **Life — Entity Recorder**.

## Life

Life records the creation of observer users in a plain text file. It does not create the anchor space and does not implement observer behavior.

Run the program with:

```sh
cargo run
```

Enter a user name when prompted. Life appends a line to `.gravither-data/users.txt`. Each line contains the record type and the supplied name, separated by a tab.

Run the tests with:

```sh
cargo test
```

## Current limits

- Only observer-user records can be created.
- The file is a simple record list, not a complete identity system.
- Names are not unique identifiers and no authentication is implemented.
- No anchor space, navigation, groups, pages, posts, reporting, or interaction logic is implemented.
- The component description is **Entity Recorder**; the component name is **Life**.

This is an intentionally small prototype, licensed under Apache-2.0.
