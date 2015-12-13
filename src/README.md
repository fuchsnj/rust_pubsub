Local publish/subscribe
====

## Documentation

```
cargo doc
```

## Usage

This will be uploaded to crates.io when it is more stable.
For now, add the following to your `Cargo.toml`

```toml
[dependencies.aws_dynamodb]
git = "https://github.com/fuchsnj/rust_pubsub.git"
```

and this to your crate root:

```rust
extern crate pubsub;
```