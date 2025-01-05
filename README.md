# OpenObserve log feeder

## Run OpenObserve instance

```
docker compose up -d
```

## Run

### Dev

```
RUST_LOG=debug cargo run -- \
  --source-file log_sources.txt \
  --ob-url http://localhost:5080 \
  --ob-username "root@example.com" \
  --ob-password "Complexpass#123" \
  --ob-org default \
  --ob-stream default
```

### Prod

```
cargo build --release

cp target/release/oo-log .

./oo-log --source-file log_sources.txt \
  --ob-url http://localhost:5080 \
  --ob-username "root@example.com" \
  --ob-password "Complexpass#123" \
  --ob-org default \
  --ob-stream default
```
