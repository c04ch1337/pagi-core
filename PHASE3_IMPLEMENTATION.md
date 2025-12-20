# Phase 3: PAGI-ExternalGateway + Dynamic Tool Registration (Kafka-only Core)

## What Phase 3 Delivers

Phase 3 establishes **PAGI-ExternalGateway** as the stable, long-lived interface for:

1. Dynamic tool registration (plugins register tools at runtime)
2. Tool discovery (ExecutiveEngine can fetch tool inventory)
3. Tool execution routing (HTTP plugins now, extensible execution backends later)
4. Core cleanup: **EventRouter is Kafka-only** (SSE dev mode removed)

## Services / Ports

| Service | Port | Notes |
|---|---:|---|
| EventRouter | 8000 | Kafka producer API (`/publish`) |
| ExternalGateway | 8010 | Dynamic tool registry + execution |
| ExecutiveEngine | 8006 | Main `/interact` entry point |

Docker compose defines the running topology in [`docker-compose.yml`](docker-compose.yml:1).

## ExternalGateway API

ExternalGateway lives in `services/pagi-external-gateway/`.

### Endpoints

- `GET /health` and `GET /healthz` → "OK" ([`main()`](services/pagi-external-gateway/src/main.rs:71))
- `POST /register_tool` → register or upsert a tool ([`register_tool()`](services/pagi-external-gateway/src/main.rs:124))
- `GET /tools` → list all tools ([`list_all_tools()`](services/pagi-external-gateway/src/main.rs:144))
- `GET /tools/:twin_id` → list tools visible to a twin (includes global tools) ([`list_tools_for_twin()`](services/pagi-external-gateway/src/main.rs:155))
- `POST /execute/:tool_name` → execute a tool ([`execute_tool()`](services/pagi-external-gateway/src/main.rs:180))

### Tool schema

Tools are registered as [`ToolSchema`](services/pagi-external-gateway/src/main.rs:28):

```json
{
  "name": "test_skill",
  "description": "A test tool",
  "plugin_url": "http://host.docker.internal:9999",
  "endpoint": "/execute",
  "parameters": {}
}
```

`twin_id` is optional; when omitted/null the tool is treated as **global**.

## Core Cleanup: EventRouter Kafka-only

EventRouter no longer supports any SSE/broadcast debug mode. It always runs in Kafka mode ([`main()`](services/pagi-event-router/src/main.rs:24)).

## ExecutiveEngine Integration

ExecutiveEngine is configured via `EXTERNAL_GATEWAY_URL` and queries ExternalGateway for tools during `/interact` ([`interact()`](services/pagi-executive-engine/src/main.rs:137)).

## Manual Smoke Test

1) Start the stack:

```bash
docker compose up --build
```

2) Register a tool:

```bash
curl -X POST http://localhost:8010/register_tool \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": null,
    "tool": {
      "name": "test_skill",
      "description": "A test tool",
      "plugin_url": "http://host.docker.internal:9999",
      "endpoint": "/execute",
      "parameters": {}
    }
  }'
```

3) List tools:

```bash
curl http://localhost:8010/tools
```

## Optional Extension: Wasm-backed Tools

ExternalGateway includes a **Wasm execution path** (for safe, portable plugins):

- Tools whose `plugin_url` starts with `wasm://...` are executed via [`wasm_plugin::execute_tool()`](services/pagi-external-gateway/src/wasm_plugin.rs:113)

For auto-discovery manifests, `plugin_type = "wasm"` is supported via [`PluginType::Wasm`](services/pagi-external-gateway/src/auto_discover.rs:44) and `wasm_path`.

Wasm registration uses a host import `pagi.register_tool(...)` implemented by [`host_register_tool()`](services/pagi-external-gateway/src/wasm_plugin.rs:37), allowing the module to declare its tool(s) during `init()`.
