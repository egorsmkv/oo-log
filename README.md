# OpenObserve log feeder

## Dev

```
RUST_LOG=debug cargo run -- --source-file log_sources.txt --ob-url http://localhost:5080 --ob-username "root@example.com" --ob-password "Complexpass#123" --ob-org default --ob-stream default
```

## Build

```
cargo build --release

cp target/release/oo-log .
```
