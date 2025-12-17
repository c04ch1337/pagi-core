# PAGI-Core

Rust monorepo (Cargo workspace) implementing the **PAGI-** microservice blueprint from the design document.

## What’s included

Core services (HTTP/JSON):

- `PAGI-IdentityService`
- `PAGI-EventRouter` (SSE-based event bus for local/dev)
- `PAGI-WorkingMemory`
- `PAGI-ContextBuilder`
- `PAGI-InferenceGateway` (mock inference)
- `PAGI-ExecutiveEngine`
- `PAGI-EmotionStateManager`
- `PAGI-SensorActuator`

Plus a sample plugin:

- `pagi-monitoring-plugin` (subscribes to the EventRouter SSE stream and logs events)

## Build

```bash
cargo build --workspace
```

## Run (dev)

Each service reads `BIND_ADDR` (default varies per service) and some read other `*_URL` variables.

Example:

```bash
RUST_LOG=info BIND_ADDR=0.0.0.0:7001 cargo run -p pagi-event-router
```

Then in another terminal:

```bash
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:7001 BIND_ADDR=0.0.0.0:7002 cargo run -p pagi-identity-service
```

## Docker Compose

`docker-compose.yml` is provided to run the core locally.

```bash
docker compose up --build
```

