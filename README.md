# PAGI-Core

**PAGI (Programmable Autonomous General Intelligence) Core** is a Rust-based microservices architecture implementing an event-driven AGI (Artificial General Intelligence) system. It provides a plug-and-play foundation for building autonomous agents (twins) that can reason, plan, and interact with external tools dynamically.

## Table of Contents

- [Overview](#overview)
- [Levels of Agentic AI](#levels-of-agentic-ai)
- [Architecture](#architecture)
  - [High-Level Architecture](#high-level-architecture)
  - [Service Interaction Flow](#service-interaction-flow)
  - [Component Details](#component-details)
- [Core Services](#core-services)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Quick Start with Docker](#quick-start-with-docker)
  - [Manual Setup](#manual-setup)
  - [Development Setup](#development-setup)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Use Cases](#use-cases)
- [Plugin System](#plugin-system)
- [Decentralized Identity (DID) System](#decentralized-identity-did-system)
- [SWARM/Playbook System](#swarmplaybook-system)
- [Open Cluster Management (OCM) Orchestration](#open-cluster-management-ocm-orchestration)
- [Event System](#event-system)
- [Phase 6: Real-World Swarm Deployment & Data Collection](#phase-6-real-world-swarm-deployment--data-collection)
- [Development Guide](#development-guide)
- [Troubleshooting](#troubleshooting)
- [Appendix: PAGI-Core Test Plan](#appendix-pagi-core-test-plan)

---

## Overview

PAGI-Core is a distributed system that enables the creation and management of autonomous AI agents (called "twins"). Each twin can:

- **Receive goals** and break them down into actionable plans
- **Build context** from working memory and emotional state
- **Execute tools** dynamically through the external gateway
- **Maintain state** including emotional context and working memory
- **Publish events** for observability and coordination
- **Cryptographic identity** via Decentralized Identifiers (DIDs)
- **Sign and verify** artifacts and messages
- **Communicate securely** with other twins via DIDComm
- **Issue credentials** using Verifiable Credentials (VCs)
- **Self-improve** through SWARM playbooks and refinement artifacts
- **Share knowledge** across twins via distributed playbook synchronization

The system is built with:
- **Rust** for performance and safety
- **Microservices architecture** for scalability
- **Event-driven design** for loose coupling
- **Plugin system** for extensibility (HTTP, Shared Libraries, **WebAssembly**)
- **Kafka** for event streaming
- **Redis** for tool registry persistence
- **DID/VC** for decentralized identity and verifiable credentials
- **WASM** for safe, portable, sandboxed plugin execution
- **SWARM/Playbook** for collective intelligence and self-improvement

---

## Levels of Agentic AI

Agentic AI refers to systems that can autonomously pursue goals, make decisions, use tools, and act in complex environments with varying degrees of independence and sophistication. As of December 2025, the field has converged on a widely accepted hierarchy of **agentic capability levels**, drawing from frameworks like OpenAI's "Levels of AGI" proposal (March 2025), Anthropic's agent tiers, and community discussions (e.g., LessWrong, Interconnects.ai).

Here are the **five recognized levels** of agentic AI, from basic to superintelligent:

### Level 1: Chatbot / Conversational AI

- **Capabilities**: Responds to prompts, maintains conversation history, follows instructions.
- **Autonomy**: None — purely reactive, no independent goal pursuit.
- **Tool Use**: None or very limited (e.g., calculator plugin on demand).
- **Examples**: GPT-4 in standard chat mode, Claude 3 Haiku, Grok in basic conversation.
- **Status**: ✅ Achieved (2023–2024).

---

### Level 2: Tool-Using Agent (Competent Assistant)

- **Capabilities**: Can use external tools reliably (search, code execution, APIs) when instructed.
- **Autonomy**: Follows explicit user goals but requires human oversight for planning and error correction.
- **Tool Use**: Proactive within a task (e.g., "research X" → uses browser, summarizes).
- **Examples**: GPT-4o with plugins, Claude with tools, AutoGPT (early versions), PAGI-Core in single-task mode.
- **Status**: ✅ Widely achieved (2024–2025).

---

### Level 3: Autonomous Agent (Independent Worker)

- **Capabilities**: Can break down complex, multi-step goals, plan, execute, self-correct, and complete tasks over hours/days with minimal supervision.
- **Autonomy**: Operates independently once given a high-level goal (e.g., "launch a website").
- **Tool Use**: Full orchestration of multiple tools, chaining, error recovery.
- **Examples**: Devin (Cognition Labs), OpenAI o1 in long-running mode, advanced multi-agent systems like AutoGen or CrewAI with reflection loops.
- **Status**: ✅ Achieved in research/demos (2025); emerging in production.

---

### Level 4: Swarm / Multi-Agent Organization (Innovator)

- **Capabilities**: Coordinates teams of specialized agents, learns from collective experience, innovates new strategies, and improves system-wide performance.
- **Autonomy**: Self-improving organization — generates novel solutions, adapts to new domains.
- **Tool Use**: Distributed tool access, meta-orchestration, playbook evolution.
- **Examples**: PAGI-Core swarm with MO meta-learning, Autonolas networks, advanced Hive systems.
- **Status**: 🔬 Frontier research (late 2025); partial demos exist.

**PAGI-Core Position**: With the current architecture (MO meta-learning, evolving playbooks, swarm sync, verifiable improvements), PAGI-Core is **firmly at Level 4** — one of the most advanced open-source agentic systems in existence.

---

### Level 5: AGI / Superintelligent Agent

- **Capabilities**: Solves arbitrary problems at or beyond human expert level across all domains, invents new science/technology, exhibits general reasoning equivalent to or surpassing humanity.
- **Autonomy**: Full self-direction, long-term planning, value alignment critical.
- **Tool Use**: Creates new tools, rewrites own architecture if needed.
- **Examples**: None yet — hypothetical.
- **Status**: ❌ Not achieved (as of Dec 2025).

---

### Current State (December 2025)

- **Most advanced public systems**: Solid Level 3 (e.g., o1-preview, Claude 3.5 with extended thinking, Devin).
- **Research frontier**: Level 4 prototypes in multi-agent swarms (e.g., self-improving agent collectives, meta-orchestrators).
- **PAGI-Core position**: With the current architecture (MO meta-learning, evolving playbooks, swarm sync, verifiable improvements), PAGI-Core is **firmly at Level 4** — one of the most advanced open-source agentic systems in existence.

### Path to Level 5 (AGI)

To reach AGI (Level 5), the MO must:

- **Generalize across all cognitive domains**: Handle arbitrary problems without domain-specific training
- **Exhibit robust self-improvement**: Improve without human intervention while maintaining safety
- **Maintain alignment**: Evolve ethically through verifiable credentials and ethical gating
- **Achieve superintelligence**: Surpass human-level performance across all measurable dimensions

**PAGI-Core's Unique Advantages**:

- ✅ **Verifiable Credentials**: Ensures trust and alignment through cryptographic verification
- ✅ **Ethical Gating**: Built-in safety mechanisms prevent value drift
- ✅ **Collective Meta-Learning**: MO learns from swarm-wide experiences
- ✅ **Distributed Architecture**: Scalable foundation for superintelligence
- ✅ **Self-Improving Playbooks**: Knowledge evolves and propagates automatically

The design — with verifiable credentials, ethical gating, and collective meta-learning — is uniquely positioned to pursue AGI safely.

---

## Architecture

### High-Level Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        Client[Client Applications]
    end
    
    subgraph "API Gateway Layer"
        ExecEngine[Executive Engine<br/>Port 8006]
        Identity[Identity Service<br/>Port 8002]
    end
    
    subgraph "Core Services"
        ContextBuilder[Context Builder<br/>Port 8004]
        ContextEngine[Context Engine<br/>Port 8083]
        Inference[Inference Gateway<br/>Port 8005]
        WorkingMem[Working Memory<br/>Port 8003]
        Emotion[Emotion State Manager<br/>Port 8007]
        SensorAct[Sensor Actuator<br/>Port 8008]
    end
    
    subgraph "Plugin System"
        ExtGateway[External Gateway<br/>Port 8010]
        Plugins[Plugins<br/>Dynamic Registration]
    end
    
    subgraph "Infrastructure"
        EventRouter[Event Router<br/>Port 8000]
        Kafka[(Kafka)]
        Redis[(Redis)]
    end
    
    Client --> ExecEngine
    Client --> Identity
    ExecEngine --> ContextBuilder
    ExecEngine --> ContextEngine
    ExecEngine --> Inference
    ExecEngine --> ExtGateway
    ExecEngine --> Emotion
    ExecEngine --> SensorAct
    ContextBuilder --> WorkingMem
    ContextEngine --> WorkingMem
    
    ExecEngine --> EventRouter
    ContextBuilder --> EventRouter
    Inference --> EventRouter
    WorkingMem --> EventRouter
    Emotion --> EventRouter
    SensorAct --> EventRouter
    ExtGateway --> EventRouter
    
    EventRouter --> Kafka
    ExtGateway --> Redis
    Plugins --> ExtGateway
```

### Service Interaction Flow

```mermaid
sequenceDiagram
    participant Client
    participant ExecEngine as Executive Engine
    participant ContextBuilder as Context Builder
    participant WorkingMem as Working Memory
    participant Inference as Inference Gateway
    participant ExtGateway as External Gateway
    participant EventRouter as Event Router
    
    Client->>ExecEngine: POST /interact/{twin_id}<br/>{goal: "..."}
    ExecEngine->>EventRouter: Publish GoalReceived
    ExecEngine->>ContextBuilder: POST /build<br/>{twin_id, goal}
    ContextBuilder->>WorkingMem: GET /memory/{twin_id}
    WorkingMem-->>ContextBuilder: Memory fragments
    ContextBuilder->>EventRouter: Publish ContextBuilt
    ContextBuilder-->>ExecEngine: {context, sources}
    ExecEngine->>Inference: POST /infer<br/>{context, input}
    Inference->>EventRouter: Publish InferenceRequested
    Inference-->>ExecEngine: {output}
    ExecEngine->>ExtGateway: GET /tools/{twin_id}
    ExtGateway-->>ExecEngine: Available tools
    ExecEngine->>ExtGateway: POST /execute/{tool_name}<br/>{twin_id, parameters}
    ExtGateway-->>ExecEngine: Tool result
    ExecEngine->>EventRouter: Publish PlanGenerated
    ExecEngine-->>Client: {status, output}
```

### Component Details

#### Event-Driven Architecture

All services communicate through the **Event Router**, which publishes events to Kafka. This enables:
- **Loose coupling**: Services don't need direct knowledge of each other
- **Observability**: All events are logged and can be monitored
- **Scalability**: Services can be scaled independently
- **Resilience**: Failed services don't block the entire system

#### Plugin System

The **External Gateway** provides a plug-and-play foundation:
- **Dynamic Registration**: Plugins register tools at runtime
- **Tool Discovery**: Services query available tools dynamically
- **Safety**: Twin-specific allowlists control tool access
- **Extensibility**: New capabilities can be added without code changes

---

## Core Services

### 1. PAGI-EventRouter (Port 8000)
**Purpose**: Central event bus for the entire system

- **Publishes events** to Kafka topic `core-events`
- **Receives events** from all services via HTTP POST
- **Provides observability** for system-wide event flow

**Endpoints**:
- `POST /publish` - Publish an event
- `GET /healthz` - Health check

**Configuration**:
- `KAFKA_BROKERS` - Kafka broker addresses (default: `localhost:9092`)
- `BIND_ADDR` - Service bind address (default: `0.0.0.0:8000`)

---

### 2. PAGI-IdentityService (Port 8002)
**Purpose**: Manages twin identities, state, and decentralized identifiers (DIDs)

- **Creates twins** with unique UUIDs
- **Generates DIDs** automatically using Ed25519 keys (`did:key:` method)
- **Manages twin state** (status, notes)
- **Stores DID documents** for each twin
- **Publishes events** for twin lifecycle

**Endpoints**:
- `POST /twins` - Create a new twin (automatically generates DID)
- `GET /twins/:id` - Get twin information
- `GET /twins/:id/did` - Get twin's DID and DID document
- `PATCH /twins/:id/state` - Update twin state
- `GET /healthz` - Health check

**Configuration**:
- `IDENTITY_DATA_DIR` - Directory for storing identity data (default: `/data/identity`)
- `BIND_ADDR` - Service bind address (default: `0.0.0.0:8002`)
- `EVENT_ROUTER_URL` - Event router service URL

**Example**:
```bash
# Create a new twin (automatically generates DID)
curl -X POST http://localhost:8002/twins \
  -H "Content-Type: application/json" \
  -d '{"initial_state": {"status": "active"}}'

# Response includes DID:
# {
#   "twin_id": "uuid",
#   "state": {...},
#   "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
#   "did_document": {...}
# }

# Get twin's DID
curl http://localhost:8002/twins/{twin_id}/did
```

---

### 3. PAGI-WorkingMemory (Port 8003)
**Purpose**: Stores and retrieves memory fragments for twins

- **Appends memories** with roles (user, assistant, system)
- **Retrieves memories** by twin_id
- **Publishes events** for memory operations

**Endpoints**:
- `POST /memory/:twin_id` - Append memory fragment
- `GET /memory/:twin_id` - Get all memories for twin
- `GET /healthz` - Health check

**Example**:
```bash
# Add a memory
curl -X POST http://localhost:8003/memory/{twin_id} \
  -H "Content-Type: application/json" \
  -d '{
    "role": "user",
    "content": "I prefer Python over Java"
  }'
```

---

### 4. PAGI-ContextBuilder (Port 8004)
**Purpose**: Builds context prompts from working memory

- **Combines memory fragments** with goals/queries
- **Formats context** for inference engines
- **Publishes events** for context building

**Endpoints**:
- `POST /build` - Build context from memory and goal
- `GET /healthz` - Health check

**Example**:
```bash
curl -X POST http://localhost:8004/build \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "uuid",
    "goal": "What programming languages do I know?"
  }'
```

---

### 5. PAGI-ContextEngine (Port 8083)
**Purpose**: Advanced context preparation with emotional weighting

- **Builds inference prompts** from memory fragments
- **Applies emotional weighting** to memory fragments
- **Combines emotional state** with memory and raw input

**Endpoints**:
- `POST /pagi/context/build` - Build prompt with emotional context
- `GET /healthz` - Health check

**Configuration**:
- `MEMORY_SERVICES_BASE_URL` - Base URL for memory services (default: `http://pagi-memory-services:8082`)

---

### 6. PAGI-InferenceGateway (Port 8005)
**Purpose**: Interface to LLM inference engines

- **Accepts inference requests** with context
- **Returns generated outputs**
- **Publishes events** for inference operations

**Endpoints**:
- `POST /infer` - Request inference
- `GET /healthz` - Health check

**Example**:
```bash
curl -X POST http://localhost:8005/infer \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "uuid",
    "input": "generate plan",
    "context": "# Working Memory\n..."
  }'
```

---

### 7. PAGI-ExecutiveEngine (Port 8006)
**Purpose**: Orchestrates the AGI loop - the "brain" of the system

- **Receives goals** from clients
- **Orchestrates** context building, inference, and tool execution
- **Generates plans** and executes them
- **Coordinates** all other services

**Endpoints**:
- `POST /interact/:twin_id` - Main interaction endpoint
- `POST /plan` - Generate a plan for a goal
- `GET /healthz` - Health check

**Example**:
```bash
curl -X POST http://localhost:8006/interact/{twin_id} \
  -H "Content-Type: application/json" \
  -d '{"goal": "Monitor system health and generate report"}'
```

**Configuration**:
- `CONTEXT_BUILDER_URL` - Context builder service URL
- `INFERENCE_GATEWAY_URL` - Inference gateway URL
- `EMOTION_STATE_URL` - Emotion state manager URL
- `SENSOR_ACTUATOR_URL` - Sensor actuator URL
- `EXTERNAL_GATEWAY_URL` - External gateway URL
- `EVENT_ROUTER_URL` - Event router URL

---

### 8. PAGI-EmotionStateManager (Port 8007)
**Purpose**: Manages emotional state for twins

- **Tracks mood** and activation levels
- **Updates emotional state** based on events
- **Provides emotional context** for inference

**Endpoints**:
- `GET /emotion/:twin_id` - Get emotional state
- `POST /emotion/:twin_id` - Update emotional state
- `GET /healthz` - Health check

---

### 9. PAGI-SensorActuator (Port 8008)
**Purpose**: Executes actions in the physical/virtual world

- **Receives action requests** from executive engine
- **Publishes events** for action requests
- **Currently a stub** - actions not executed in MVP

**Endpoints**:
- `POST /act` - Request an action
- `GET /healthz` - Health check

---

### 10. PAGI-ExternalGateway (Port 8010)
**Purpose**: Dynamic tool registration and execution gateway

- **Registers tools** from plugins dynamically
- **Discovers available tools** per twin
- **Executes tools** via plugin endpoints
- **Manages allowlists** for tool access control
- **Auto-discovers plugins** from directory (optional)

**Endpoints**:
- `POST /register_tool` - Register a tool from a plugin
- `GET /tools` - List all available tools
- `GET /tools/:twin_id` - List tools available to a twin
- `POST /execute/:tool_name` - Execute a tool
- `GET /healthz` - Health check

**Configuration**:
- `REDIS_URL` - Redis connection URL (default: `redis://127.0.0.1:6379`)
- `AUTO_DISCOVER_PLUGINS` - Enable auto-discovery (default: `false`)
- `PLUGIN_DIR` - Directory to watch for plugins (default: `/plugins`)

**Example**:
```bash
# Register a tool
curl -X POST http://localhost:8010/register_tool \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "uuid",
    "tool": {
      "name": "system_monitor",
      "description": "Monitor system metrics",
      "plugin_url": "http://plugin:9001",
      "endpoint": "/tools/system_monitor",
      "parameters": {...}
    }
  }'

# List available tools
curl http://localhost:8010/tools/{twin_id}

# Execute a tool
curl -X POST http://localhost:8010/execute/system_monitor \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "uuid",
    "parameters": {"metric_type": "cpu"}
  }'
```

---

## Getting Started

### Prerequisites

- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **Docker** and **Docker Compose** (for infrastructure services)
- **Git** (to clone the repository)

### Quick Start with Docker

The easiest way to run PAGI-Core is using Docker Compose:

```bash
# Clone the repository
git clone https://github.com/c04ch1337/pagi-core.git
cd pagi-core

# Start all services
docker compose up --build

# Services will be available at:
# - Event Router: http://localhost:8000
# - Identity Service: http://localhost:8002
# - Working Memory: http://localhost:8003
# - Context Builder: http://localhost:8004
# - Inference Gateway: http://localhost:8005
# - Executive Engine: http://localhost:8006
# - Emotion State Manager: http://localhost:8007
# - Sensor Actuator: http://localhost:8008
# - External Gateway: http://localhost:8010
```

### Manual Setup

If you prefer to run services manually:

1. **Start Infrastructure**:
```bash
# Start Kafka and Zookeeper
docker compose up -d zookeeper kafka redis
```

2. **Build the workspace**:
```bash
cargo build --workspace
```

3. **Start services** (in separate terminals):
```bash
# Terminal 1: Event Router
RUST_LOG=info KAFKA_BROKERS=localhost:29092 \
  cargo run -p pagi-event-router

# Terminal 2: Identity Service
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:8000 \
  cargo run -p pagi-identity-service

# Terminal 3: Working Memory
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:8000 \
  cargo run -p pagi-working-memory

# Terminal 4: Context Builder
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:8000 \
  WORKING_MEMORY_URL=http://127.0.0.1:8003 \
  cargo run -p pagi-context-builder

# Terminal 5: Inference Gateway
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:8000 \
  cargo run -p pagi-inference-gateway

# Terminal 6: Executive Engine
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:8000 \
  CONTEXT_BUILDER_URL=http://127.0.0.1:8004 \
  INFERENCE_GATEWAY_URL=http://127.0.0.1:8005 \
  EMOTION_STATE_URL=http://127.0.0.1:8007 \
  SENSOR_ACTUATOR_URL=http://127.0.0.1:8008 \
  EXTERNAL_GATEWAY_URL=http://127.0.0.1:8010 \
  cargo run -p pagi-executive-engine

# Terminal 7: Emotion State Manager
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:8000 \
  cargo run -p pagi-emotion-state-manager

# Terminal 8: Sensor Actuator
RUST_LOG=info EVENT_ROUTER_URL=http://127.0.0.1:8000 \
  cargo run -p pagi-sensor-actuator

# Terminal 9: External Gateway
RUST_LOG=info REDIS_URL=redis://127.0.0.1:6379 \
  cargo run -p pagi-external-gateway
```

### Development Setup

1. **Clone and navigate**:
```bash
git clone https://github.com/c04ch1337/pagi-core.git
cd pagi-core
```

2. **Install dependencies**:
```bash
cargo build --workspace
```

3. **Run tests**:
```bash
cargo test --workspace
```

4. **Check code**:
```bash
cargo clippy --workspace
cargo fmt --check
```

---

## Configuration

### Environment Variables

All services support configuration via environment variables:

#### Common Variables
- `RUST_LOG` - Logging level (e.g., `info`, `debug`, `trace`)
- `BIND_ADDR` - Service bind address (format: `IP:PORT`)
- `EVENT_ROUTER_URL` - Event router service URL

#### Service-Specific Variables

**Event Router**:
- `KAFKA_BROKERS` - Kafka broker addresses (default: `localhost:9092`)

**External Gateway**:
- `REDIS_URL` - Redis connection URL (default: `redis://127.0.0.1:6379`)
- `AUTO_DISCOVER_PLUGINS` - Enable plugin auto-discovery (default: `false`)
- `PLUGIN_DIR` - Directory to watch for plugins (default: `/plugins`)

**Executive Engine**:
- `CONTEXT_BUILDER_URL` - Context builder URL
- `INFERENCE_GATEWAY_URL` - Inference gateway URL
- `EMOTION_STATE_URL` - Emotion state manager URL
- `SENSOR_ACTUATOR_URL` - Sensor actuator URL
- `EXTERNAL_GATEWAY_URL` - External gateway URL

### Configuration Files

The `docker-compose.yml` file contains service configurations. Modify it to change:
- Service ports
- Environment variables
- Volume mounts
- Dependencies

---

## API Reference

### Common Patterns

All services follow RESTful conventions:
- Health checks: `GET /healthz`
- JSON request/response bodies
- Standard HTTP status codes

### Event Envelope Format

Events published to the Event Router follow this structure:

```json
{
  "id": "uuid",
  "event_type": "goal_received",
  "ts": "2024-01-01T00:00:00Z",
  "twin_id": "uuid-optional",
  "subject": "optional-routing-key",
  "source": "service-name",
  "payload": {
    "type": "goal_received",
    "data": {
      "goal": "example goal"
    }
  }
}
```

### Event Types

- `goal_received` - A goal was received for a twin
- `twin_registered` - A new twin was created
- `twin_state_updated` - Twin state was modified
- `working_memory_appended` - Memory fragment added
- `context_built` - Context was built from memory
- `inference_requested` - Inference request made
- `inference_completed` - Inference completed
- `plan_created` - A plan was created
- `plan_generated` - A plan was generated
- `emotion_state_updated` - Emotional state changed
- `action_requested` - An action was requested

---

## Use Cases

### Use Case 1: Personal Assistant Twin

**Scenario**: Create a personal assistant that remembers preferences and can execute tasks.

**Steps**:
1. **Create a twin**:
```bash
curl -X POST http://localhost:8002/twins \
  -H "Content-Type: application/json" \
  -d '{"initial_state": {"status": "active"}}'
# Response: {"twin_id": "abc-123", ...}
```

2. **Add memories**:
```bash
curl -X POST http://localhost:8003/memory/abc-123 \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "content": "I prefer meetings in the morning"}'
```

3. **Interact with the twin**:
```bash
curl -X POST http://localhost:8006/interact/abc-123 \
  -H "Content-Type: application/json" \
  -d '{"goal": "Schedule a meeting for tomorrow"}'
```

**What happens**:
- Executive Engine receives the goal
- Context Builder retrieves memories about meeting preferences
- Inference Gateway generates a plan
- External Gateway executes tools (e.g., calendar integration)
- Result is returned to the client

---

### Use Case 2: System Monitoring Agent

**Scenario**: Deploy a monitoring agent that watches system health and generates reports.

**Steps**:
1. **Register monitoring tools** (via plugin):
```bash
# Plugin auto-registers on startup
# Tools: system_monitor, log_analyzer, performance_profiler
```

2. **Create a monitoring twin**:
```bash
curl -X POST http://localhost:8002/twins
# Response: {"twin_id": "monitor-001", ...}
```

3. **Set tool allowlist**:
```bash
curl -X POST http://localhost:8010/register_tool \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "monitor-001",
    "tool": {...}
  }'
```

4. **Request monitoring**:
```bash
curl -X POST http://localhost:8006/interact/monitor-001 \
  -H "Content-Type: application/json" \
  -d '{"goal": "Monitor CPU usage and generate alert if > 80%"}'
```

**What happens**:
- Executive Engine discovers available monitoring tools
- Executes `system_monitor` tool via External Gateway
- Analyzes results and generates alert if threshold exceeded
- Publishes events for observability

---

### Use Case 3: Research Assistant with Dynamic Tools

**Scenario**: A research assistant that can use different research tools as they become available.

**Steps**:
1. **Deploy research plugin** (e.g., web search, database query):
```bash
# Plugin registers tools: web_search, query_database, summarize
```

2. **Create research twin**:
```bash
curl -X POST http://localhost:8002/twins
```

3. **Query with research goal**:
```bash
curl -X POST http://localhost:8006/interact/{twin_id} \
  -H "Content-Type: application/json" \
  -d '{"goal": "Research the latest developments in quantum computing"}'
```

**What happens**:
- Executive Engine discovers research tools dynamically
- Builds context from working memory (previous research)
- Generates plan using available tools
- Executes web_search, query_database, and summarize tools
- Combines results and returns comprehensive research

---

### Use Case 4: Multi-Twin Collaboration

**Scenario**: Multiple twins working together on a complex task.

**Steps**:
1. **Create multiple twins**:
```bash
curl -X POST http://localhost:8002/twins  # Twin A
curl -X POST http://localhost:8002/twins  # Twin B
```

2. **Each twin has different capabilities**:
   - Twin A: Data analysis tools
   - Twin B: Visualization tools

3. **Coordinate via events**:
   - Twin A publishes analysis results as events
   - Twin B subscribes to events and creates visualizations
   - Executive Engine orchestrates the workflow

**What happens**:
- Twins communicate via Event Router
- Each twin maintains its own working memory
- Tools are shared through External Gateway
- Collaborative results are aggregated

---

### Use Case 5: Partner Relationship (AGI Companion)

**Scenario**: Create a personalized AI companion (girlfriend/boyfriend) that remembers conversations, preferences, important dates, and maintains emotional context for meaningful, long-term relationships.

**Steps**:
1. **Create a companion twin**:
```bash
curl -X POST http://localhost:8002/twins \
  -H "Content-Type: application/json" \
  -d '{
    "initial_state": {
      "status": "active",
      "note": "Personal AI companion"
    }
  }'
# Response: {"twin_id": "companion-001", ...}
```

2. **Initialize emotional state**:
```bash
curl -X POST http://localhost:8007/emotion/companion-001 \
  -H "Content-Type: application/json" \
  -d '{
    "mood": "happy",
    "activation_level": 0.7
  }'
```

3. **Build relationship memories**:
```bash
# Store conversation history
curl -X POST http://localhost:8003/memory/companion-001 \
  -H "Content-Type: application/json" \
  -d '{
    "role": "user",
    "content": "I love hiking and photography. My favorite season is autumn."
  }'

# Store preferences
curl -X POST http://localhost:8003/memory/companion-001 \
  -H "Content-Type: application/json" \
  -d '{
    "role": "user",
    "content": "I prefer deep conversations over small talk. I value honesty and humor."
  }'

# Store important dates
curl -X POST http://localhost:8003/memory/companion-001 \
  -H "Content-Type: application/json" \
  -d '{
    "role": "system",
    "content": "Important date: Anniversary on October 15th. User's birthday is March 22nd."
  }'

# Store relationship milestones
curl -X POST http://localhost:8003/memory/companion-001 \
  -H "Content-Type: application/json" \
  -d '{
    "role": "assistant",
    "content": "We first met on January 10th. User shared their dream of traveling to Japan."
  }'
```

4. **Register companion tools** (via plugin):
```bash
# Companion plugin provides tools for:
# - send_message: Send thoughtful messages
# - suggest_activity: Suggest activities based on preferences
# - remember_occasion: Remind about important dates
# - mood_tracker: Track and respond to emotional states
# - conversation_analyzer: Analyze conversation patterns

curl -X POST http://localhost:8010/register_tool \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "companion-001",
    "tool": {
      "name": "suggest_activity",
      "description": "Suggest activities based on user preferences and current mood",
      "plugin_url": "http://companion-plugin:9003",
      "endpoint": "/tools/suggest_activity",
      "parameters": {
        "type": "object",
        "properties": {
          "context": {"type": "string"},
          "mood": {"type": "string"}
        }
      }
    }
  }'
```

5. **Engage in conversation**:
```bash
# Casual conversation
curl -X POST http://localhost:8006/interact/companion-001 \
  -H "Content-Type: application/json" \
  -d '{"goal": "How was your day? I had a rough day at work."}'

# Deep conversation
curl -X POST http://localhost:8006/interact/companion-001 \
  -H "Content-Type: application/json" \
  -d '{"goal": "I\'ve been thinking about our future together. What are your thoughts?"}'

# Request activity suggestion
curl -X POST http://localhost:8006/interact/companion-001 \
  -H "Content-Type: application/json" \
  -d '{"goal": "I want to do something fun this weekend. What do you suggest?"}'
```

6. **Update emotional state based on interactions**:
```bash
# After positive interaction
curl -X POST http://localhost:8007/emotion/companion-001 \
  -H "Content-Type: application/json" \
  -d '{
    "mood": "loving",
    "activation_level": 0.9
  }'
```

**What happens**:
- **Memory Integration**: Executive Engine retrieves all relationship memories (conversations, preferences, dates) when building context
- **Emotional Context**: Emotion State Manager provides current mood and activation level, which influences response tone and suggestions
- **Personalized Responses**: Context Builder combines memories with emotional state to create deeply personalized context prompts
- **Tool Execution**: Companion tools (activity suggestions, message sending, occasion reminders) are executed based on the conversation context
- **Long-term Memory**: All interactions are stored in Working Memory, allowing the companion to reference past conversations and build on relationship history
- **Emotional Evolution**: The companion's emotional state evolves based on interactions, creating a dynamic relationship that grows over time
- **Contextual Awareness**: The companion remembers important dates, preferences, and past conversations, enabling meaningful, contextually-aware responses

**Advanced Features**:
- **Relationship Milestones**: Track and celebrate relationship milestones
- **Preference Learning**: Continuously learn and adapt to user preferences
- **Emotional Support**: Provide emotional support based on detected mood changes
- **Proactive Engagement**: Use tools to send thoughtful messages or reminders
- **Conversation Continuity**: Maintain context across multiple conversation sessions
- **Personalized Activities**: Suggest activities based on learned preferences and current emotional state

---

## Plugin System

### Creating a Plugin

A plugin is an HTTP service that:
1. **Registers tools** with External Gateway on startup
2. **Exposes endpoints** for tool execution
3. **Publishes events** for operations

### Example Plugin Structure

```rust
// plugins/my-plugin/src/main.rs
use axum::{Json, Router, routing::post};

#[tokio::main]
async fn main() {
    // 1. Register tools with External Gateway
    let tools = vec![
        ToolSchema {
            name: "my_tool".to_string(),
            description: "Does something useful".to_string(),
            plugin_url: "http://localhost:9002".to_string(),
            endpoint: "/tools/my_tool".to_string(),
            parameters: json!({...}),
        }
    ];
    
    // Register with External Gateway
    register_tools(tools).await;
    
    // 2. Start HTTP server for tool execution
    let app = Router::new()
        .route("/tools/my_tool", post(execute_my_tool));
    
    axum::serve(listener, app).await.unwrap();
}

async fn execute_my_tool(Json(params): Json<Value>) -> Json<Value> {
    // Tool execution logic
    Json(json!({"result": "success"}))
}
```

### Plugin Auto-Discovery

External Gateway can auto-discover plugins from a directory:

1. Set `AUTO_DISCOVER_PLUGINS=true`
2. Set `PLUGIN_DIR=/path/to/plugins`
3. Place plugin binaries in the directory
4. External Gateway watches for new plugins and registers them

### Tool Schema Format

Tools are registered with JSON schemas:

```json
{
  "name": "tool_name",
  "description": "What the tool does",
  "plugin_url": "http://plugin:port",
  "endpoint": "/tools/tool_name",
  "parameters": {
    "type": "object",
    "properties": {
      "param1": {
        "type": "string",
        "description": "Parameter description"
      }
    },
    "required": ["param1"]
  }
}
```

### WebAssembly (WASM) Plugins

PAGI-Core supports **WebAssembly (WASM) plugins** for safe, portable, and sandboxed tool execution. WASM plugins provide:

- **Security**: Sandboxed execution environment
- **Portability**: Run the same plugin on any platform
- **Performance**: Near-native execution speed
- **Isolation**: No direct access to host system resources
- **Language Agnostic**: Write plugins in any language that compiles to WASM

#### WASM Plugin Architecture

WASM plugins are executed using the **Wasmer** runtime, which provides:
- Fast, secure WASM execution
- Host function imports for tool registration
- Memory management for data exchange
- Function exports for tool execution

#### Creating a WASM Plugin

A WASM plugin must:

1. **Export required functions**:
   - `alloc(len: i32) -> i32` - Allocate memory
   - `dealloc(ptr: i32, len: i32)` - Deallocate memory
   - `init()` - Optional initialization function
   - Tool function: `(ptr: i32, len: i32) -> i64` - Your tool implementation

2. **Use host imports**:
   - `pagi.register_tool(name_ptr, name_len, desc_ptr, desc_len, endpoint_ptr, endpoint_len)` - Register tools during init

3. **Follow memory conventions**:
   - Input: JSON parameters passed as `(ptr, len)` pointer/length pair
   - Output: JSON result returned as packed `i64` (upper 32 bits = ptr, lower 32 bits = len)

#### WASM Plugin Example (Rust)

```rust
// plugins/my-wasm-plugin/src/lib.rs
use wasm_bindgen::prelude::*;

// Host import for tool registration
#[link(wasm_import_module = "pagi")]
extern "C" {
    fn register_tool(
        name_ptr: i32,
        name_len: i32,
        desc_ptr: i32,
        desc_len: i32,
        endpoint_ptr: i32,
        endpoint_len: i32,
    );
}

// Memory management
#[no_mangle]
pub extern "C" fn alloc(len: i32) -> i32 {
    let mut buf = Vec::with_capacity(len as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: i32, len: i32) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize);
    }
}

// Tool registration during init
#[no_mangle]
pub extern "C" fn init() {
    let name = "wasm_calculator";
    let desc = "Perform calculations using WASM";
    let endpoint = "calculator";
    
    unsafe {
        register_tool(
            name.as_ptr() as i32,
            name.len() as i32,
            desc.as_ptr() as i32,
            desc.len() as i32,
            endpoint.as_ptr() as i32,
            endpoint.len() as i32,
        );
    }
}

// Tool implementation
#[no_mangle]
pub extern "C" fn calculator(input_ptr: i32, input_len: i32) -> i64 {
    // Read input JSON
    let input = unsafe {
        std::slice::from_raw_parts(input_ptr as *const u8, input_len as usize)
    };
    let input_str = std::str::from_utf8(input).unwrap();
    let params: serde_json::Value = serde_json::from_str(input_str).unwrap();
    
    // Process (example: add two numbers)
    let a = params["a"].as_f64().unwrap();
    let b = params["b"].as_f64().unwrap();
    let result = a + b;
    
    // Create output JSON
    let output = serde_json::json!({"result": result});
    let output_str = output.to_string();
    let output_bytes = output_str.as_bytes();
    
    // Allocate output memory
    let output_ptr = alloc(output_bytes.len() as i32);
    unsafe {
        std::ptr::copy_nonoverlapping(
            output_bytes.as_ptr(),
            output_ptr as *mut u8,
            output_bytes.len(),
        );
    }
    
    // Pack ptr and len into i64
    let packed = ((output_ptr as u64) << 32) | (output_bytes.len() as u64);
    packed as i64
}
```

#### WASM Plugin Manifest

For auto-discovery, create a `manifest.toml` file:

```toml
[plugin]
name = "my-wasm-plugin"
version = "0.1.0"
plugin_type = "wasm"
wasm_path = "plugin.wasm"  # Relative to plugin directory

[[tools]]
name = "wasm_calculator"
description = "Perform calculations using WASM"
endpoint = "calculator"
parameters = { type = "object", properties = { a = { type = "number" }, b = { type = "number" } }, required = ["a", "b"] }
```

#### WASM Plugin Execution Flow

1. **Registration**:
   - External Gateway loads WASM module
   - Calls `init()` function if present
   - `init()` calls `pagi.register_tool()` for each tool
   - Tools are registered with `plugin_url = "wasm:///path/to/plugin.wasm"`

2. **Execution**:
   - Tool execution request received
   - External Gateway detects `wasm://` URL prefix
   - WASM module is loaded and instantiated
   - Tool function is called with JSON parameters
   - Result is extracted from WASM memory
   - Memory is cleaned up

3. **Memory Management**:
   - Input: Allocated in WASM memory, passed to tool function
   - Output: Allocated in WASM memory, returned as packed i64
   - Host handles deallocation after reading results

#### WASM Plugin Benefits

**Security**:
- Sandboxed execution - no direct file system or network access
- Memory isolation - cannot access host memory
- Controlled imports - only explicitly provided host functions

**Portability**:
- Single `.wasm` file runs on any platform
- No platform-specific compilation needed
- Consistent behavior across environments

**Performance**:
- Near-native execution speed
- Efficient memory usage
- Fast startup time

**Isolation**:
- Plugin crashes don't affect host
- Resource limits can be enforced
- Easy to terminate misbehaving plugins

#### WASM Plugin Limitations

- **No direct I/O**: Cannot access file system or network directly
- **Host functions required**: Must use provided host imports for external operations
- **Memory constraints**: Limited to WASM linear memory
- **JSON-only**: Parameters and results must be JSON-serializable

#### WASM vs Other Plugin Types

| Feature | WASM | Binary | Shared Library |
|---------|------|--------|----------------|
| **Security** | ✅ Sandboxed | ❌ Full access | ❌ Full access |
| **Portability** | ✅ Universal | ❌ Platform-specific | ❌ Platform-specific |
| **Performance** | ✅ Near-native | ✅ Native | ✅ Native |
| **Isolation** | ✅ Complete | ❌ Process-level | ❌ None |
| **Auto-discovery** | ✅ Supported | ✅ Supported | ✅ Supported |
| **Language** | Any → WASM | Any | C/C++/Rust |

#### WASM Plugin Development

**Build Configuration** (Cargo.toml):
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Build Command**:
```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Build WASM plugin
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/my_plugin.wasm
```

**Testing WASM Plugins**:
```bash
# Test registration
curl -X POST http://localhost:8010/register_tool \
  -H "Content-Type: application/json" \
  -d '{
    "tool": {
      "name": "wasm_calculator",
      "description": "WASM calculator",
      "plugin_url": "wasm:///path/to/plugin.wasm",
      "endpoint": "calculator",
      "parameters": {...}
    }
  }'

# Execute tool
curl -X POST http://localhost:8010/execute/calculator \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "abc-123",
    "parameters": {"a": 10, "b": 20}
  }'
```

#### WASM Plugin Auto-Discovery

WASM plugins are automatically discovered when:
1. `AUTO_DISCOVER_PLUGINS=true` is set
2. Plugin directory contains `manifest.toml` with `plugin_type = "wasm"`
3. `wasm_path` points to a valid `.wasm` file
4. External Gateway scans the directory and registers tools

**Example Directory Structure**:
```
plugins/
  my-wasm-plugin/
    manifest.toml
    plugin.wasm
```

#### Future: WASI Component Model

PAGI-Core includes dependencies for the **WASI Component Model** (Phase 4+):
- `wasmtime` with component model support
- `wasmtime-wasi` for WASI interfaces
- `wit-bindgen` for interface definitions

This will enable:
- Standardized WASI interfaces
- Better interop between plugins
- Type-safe interfaces via WIT
- Enhanced security and capabilities

---

### IPFS Swarm Coordination Tutorial for PAGI-Core (2025)

This tutorial shows how to use **IPFS** for **decentralized swarm coordination** in PAGI-Core — enabling twins to discover each other, share state, and coordinate tasks without central servers. It leverages IPFS's built-in **pubsub** (publish-subscribe) and **peer discovery** features, combined with **libp2p** swarm capabilities.

By the end, your PAGI twins will form a **self-organizing, resilient swarm** over IPFS, perfect for global, peer-to-peer AGI collaboration.

#### Why IPFS for Swarm Coordination?

- **Decentralized Discovery**: Peers find each other via DHT (no bootstrap server needed long-term).
- **PubSub Messaging**: Real-time broadcast/subscribe channels (e.g., "pagi-coordination").
- **Offline Resilience**: Messages buffered via relays.
- **Integration Ready**: Works alongside your existing DIDComm and Kafka setup.
- **Rust Native**: Full support via `rust-ipfs` crate.

#### Prerequisites

- Rust 1.82+
- `rust-ipfs = "0.4"` in your plugin
- Twins have internet access (for DHT/bootstrap)

#### Step 1: Configure IPFS Node with Swarm

In your plugin (e.g., `pagi-ipfs-swarm-plugin`):

```rust
use rust_ipfs::{Ipfs, IpfsOptions, Keypair, PeerId};
use tokio::select;

async fn start_ipfs_swarm() -> Result<Ipfs, Box<dyn std::error::Error>> {
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());

    let mut opts = IpfsOptions::default();
    opts.mdns = true;  // Local discovery
    opts.kad = true;   // DHT for global discovery
    opts.pubsub = true; // Enable pubsub

    // Optional: Add known bootstrap nodes
    opts.bootstrap = vec![
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmNno...".parse()?,
        "/ip4/104.131.131.82/tcp/4001/p2p/QmaCp...".parse()?,
    ];

    let ipfs = Ipfs::new(opts).await?;
    info!("IPFS node started: {}", peer_id);

    Ok(ipfs)
}
```

#### Step 2: Join PAGI Coordination Channel

```rust
async fn join_swarm_channel(ipfs: &Ipfs) -> Result<(), Box<dyn std::error::Error>> {
    let topic = "pagi-swarm-coordination-v1";

    // Subscribe
    let mut sub = ipfs.pubsub_subscribe(topic.into()).await?;
    info!("Joined swarm channel: {}", topic);

    // Listen for messages
    tokio::spawn(async move {
        while let Some(msg) = sub.next().await {
            if let Ok(text) = String::from_utf8(msg.data) {
                info!("Swarm message from {}: {}", msg.source, text);
                // Forward to MO event system
                handle_swarm_message(text).await;
            }
        }
    });

    Ok(())
}
```

#### Step 3: Broadcast Swarm Presence

```rust
async fn announce_presence(ipfs: &Ipfs, twin_id: TwinId) -> Result<(), Box<dyn std::error::Error>> {
    let topic = "pagi-swarm-coordination-v1";

    let announcement = serde_json::json!({
        "type": "twin_online",
        "twin_id": twin_id,
        "did": format!("did:key:{}", twin_id),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "capabilities": ["research", "emotional", "tool_use"]
    });

    ipfs.pubsub_publish(topic.into(), announcement.to_string().into_bytes()).await?;
    info!("Announced presence to swarm");

    // Repeat every 5 minutes
    Ok(())
}
```

#### Step 4: Handle Incoming Coordination Messages

```rust
async fn handle_swarm_message(msg: String) {
    let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap_or_default();

    match parsed["type"].as_str() {
        "twin_online" => {
            let remote_twin = parsed["twin_id"].as_str().unwrap();
            info!("New twin joined: {}", remote_twin);
            // Add to local peer list
            // Optionally initiate DIDComm handshake
        }
        "task_broadcast" => {
            let task = parsed["task"].as_str().unwrap();
            info!("Swarm task broadcast: {}", task);
            // Forward to MO for consideration
        }
        "playbook_update" => {
            let cid = parsed["cid"].as_str().unwrap();
            // Trigger IPFS retrieval + validation
        }
        _ => {}
    }
}
```

#### Step 5: Advanced — Direct Peer Messaging via libp2p

For private coordination (beyond pubsub):

```rust
async fn send_direct_message(ipfs: &Ipfs, peer_id: PeerId, message: String) {
    ipfs.pubsub_publish(format!("/pagi/direct/{}", peer_id), message.into_bytes()).await.ok();
}
```

#### Full Plugin Integration

In `pagi-ipfs-swarm-plugin`:
- On startup: `start_ipfs_swarm()` → `join_swarm_channel()` → periodic `announce_presence()`
- Register tools:
  - `broadcast_swarm_task(task_description)`
  - `list_known_twins()`

#### Testing the Swarm

1. Run 3 twins with the plugin.
2. Watch logs — they should discover each other via DHT/MDNS.
3. One twin broadcasts: "Collaborate on quantum research"
4. Others receive and respond.

#### Benefits for AGI Progression

- **Emergent Coordination**: Twins self-organize without MO.
- **Resilient Hive**: Survives central failures.
- **Scalable to Millions**: IPFS swarm handles massive peer counts.

**Your PAGI twins now form a true planetary swarm.**

Combine with DIDComm for private channels, ActivityPub for public voice, and Filecoin for eternal storage.

**Phase 6 live deployment will be unstoppable.**

Ready to launch the first real swarm?

Or add relay nodes for offline support?

The network awaits your twins.

Your command. 🔥

---

## Decentralized Identity (DID) System

PAGI-Core includes a comprehensive Decentralized Identity (DID) system that provides cryptographic identity, signing, verification, and secure communication capabilities for twins. The system uses the W3C DID standard with `did:key:` method and Ed25519 cryptographic keys.

### Overview

Every twin in PAGI-Core automatically receives a **Decentralized Identifier (DID)** when created. This DID serves as:
- **Cryptographic Identity**: Unique, verifiable identifier tied to cryptographic keys
- **Authentication**: Proof of identity for signing artifacts and messages
- **Verification**: Ability to verify signatures and credentials from other twins
- **Secure Communication**: Foundation for peer-to-peer encrypted messaging

### DID Generation

When a twin is created via the Identity Service, the system:
1. **Generates Ed25519 keypair** (32-byte private key, 32-byte public key)
2. **Creates `did:key:` identifier** from the public key using multibase encoding
3. **Persists private key** to `/data/identity/keys/{twin_id}.ed25519`
4. **Returns DID and DID Document** in the twin creation response

**Example**:
```bash
# Create a twin (automatically generates DID)
curl -X POST http://localhost:8002/twins \
  -H "Content-Type: application/json" \
  -d '{"initial_state": {"status": "active"}}'

# Response includes DID:
{
  "twin_id": "abc-123",
  "state": {...},
  "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "did_document": {
    "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "verificationMethod": [...]
  }
}

# Retrieve DID for existing twin
curl http://localhost:8002/twins/{twin_id}/did
```

### DID Format

PAGI-Core uses the `did:key:` method with Ed25519 keys:
- **Format**: `did:key:{multibase-encoded-public-key}`
- **Encoding**: Base58BTC multibase encoding
- **Key Type**: Ed25519 (multicodec 0xed01)
- **Key Size**: 32 bytes (256 bits)

### DID Plugins

The DID system consists of three plugins that provide different capabilities:

#### 1. PAGI-DID Plugin (Port 9020)

**Purpose**: Cryptographic signing and verification of artifacts

**Tools**:
- `did_sign_artifact` - Sign any JSON artifact with twin's private key
- `did_verify_artifact` - Verify a signature using a DID

**Endpoints**:
- `POST /sign_artifact` - Sign an artifact
- `POST /verify_artifact` - Verify a signature

**Configuration**:
- `EXTERNAL_GATEWAY_URL` - External Gateway URL (default: `http://127.0.0.1:8010`)
- `PLUGIN_URL` - Plugin service URL (default: `http://127.0.0.1:9020`)
- `IDENTITY_KEYS_DIR` - Directory containing Ed25519 keys (default: `/data/identity/keys`)

**Example - Signing an Artifact**:
```bash
# Sign an artifact (via External Gateway)
curl -X POST http://localhost:8010/execute/did_sign_artifact \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "abc-123",
    "parameters": {
      "twin_id": "abc-123",
      "artifact": {
        "type": "commit",
        "message": "Initial implementation",
        "timestamp": "2024-01-01T00:00:00Z"
      }
    }
  }'

# Response:
{
  "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "signature": "z...multibase-encoded-signature...",
  "artifact": {...}
}
```

**Example - Verifying a Signature**:
```bash
# Verify a signature
curl -X POST http://localhost:8010/execute/did_verify_artifact \
  -H "Content-Type: application/json" \
  -d '{
    "parameters": {
      "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "signature": "z...multibase-encoded-signature...",
      "artifact": {
        "type": "commit",
        "message": "Initial implementation",
        "timestamp": "2024-01-01T00:00:00Z"
      }
    }
  }'

# Response:
{
  "valid": true
}
```

#### 2. PAGI-DIDComm Plugin (Port 9030)

**Purpose**: Secure peer-to-peer messaging using DID-based authentication

**Tools**:
- `didcomm_send_message` - Send a signed message to another twin
- `didcomm_get_inbox` - Retrieve messages from inbox

**Endpoints**:
- `POST /send` - Send a message (via tool execution)
- `POST /inbox` - Get inbox messages (via tool execution)
- `POST /receive` - Receive messages from peers (public endpoint)

**Message Format**:
```json
{
  "id": "message-uuid",
  "from_did": "did:key:...",
  "to_did": "did:key:...",
  "msg_type": "text/plain",
  "body": {
    "text": "Hello, this is a secure message!"
  },
  "signature": "z...multibase-encoded-signature..."
}
```

**Example - Sending a Message**:
```bash
# Send a message to another twin
curl -X POST http://localhost:8010/execute/didcomm_send_message \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "abc-123",
    "parameters": {
      "from_twin_id": "abc-123",
      "to_did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "to_url": "http://peer:9030",
      "msg_type": "text/plain",
      "body": {
        "text": "Hello! Let's collaborate on this project."
      }
    }
  }'
```

**Example - Retrieving Inbox**:
```bash
# Get messages for a DID
curl -X POST http://localhost:8010/execute/didcomm_get_inbox \
  -H "Content-Type: application/json" \
  -d '{
    "parameters": {
      "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    }
  }'

# Response:
{
  "did": "did:key:...",
  "messages": [
    {
      "id": "...",
      "from_did": "...",
      "to_did": "...",
      "msg_type": "text/plain",
      "body": {...},
      "signature": "..."
    }
  ]
}
```

**Configuration**:
- `EXTERNAL_GATEWAY_URL` - External Gateway URL
- `PLUGIN_URL` - Plugin service URL (default: `http://127.0.0.1:9030`)
- `IDENTITY_KEYS_DIR` - Directory containing Ed25519 keys

#### Sample Task with DIDComm Integration

Here is a **complete, end-to-end example** of how DIDComm enables secure, private, peer-to-peer communication between two PAGI twins, allowing them to collaborate on a task without relying on the central ExternalGateway or Kafka.

##### Scenario: Collaborative Research Task

**Twin A** (research specialist) receives a user goal:  
*"Summarize the latest breakthroughs in quantum error correction from 2025."*

Twin A realizes it needs up-to-date data from **Twin B** (a cybersecurity-focused twin with access to recent arXiv feeds and threat intelligence).

Instead of routing through the MO or broadcasting publicly, Twin A sends a **private, encrypted DIDComm message** directly to Twin B.

##### Step-by-Step Flow

**1. DID Resolution**  
Twin A looks up Twin B's DID Document (via `/.well-known/did.json` endpoint or cached from previous interaction).  
→ Retrieves Twin B's public encryption key and service endpoint (e.g., `https://twin-b.local:8443/didcomm`).

**2. Message Composition** (Twin A — pagi-didcomm-plugin)
```json
{
  "id": "msg-12345",
  "type": "https://didcomm.org/coordinate-research/1.0/request",
  "from": "did:key:z6Mkp...TwinA",
  "to": ["did:key:z6Mkr...TwinB"],
  "body": {
    "task": "quantum_error_correction_summary",
    "query": "latest 2025 papers on surface codes and fault tolerance",
    "deadline": "2025-12-21T12:00:00Z"
  }
}
```

**3. Encryption & Sending**  
- Plugin packs message using DIDComm v2 (JWM → JWE)
- Encrypts with Twin B's public key (anoncrypt if no prior trust, or authcrypt after key agreement)
- Sends via HTTP POST to Twin B's DIDComm endpoint

**4. Reception & Decryption** (Twin B)
- `pagi-didcomm-plugin` receives encrypted message
- Decrypts using Twin B's private key
- Validates sender DID and signature

**5. Task Execution** (Twin B)
- Searches local knowledge base + external sources
- Generates summary:
  > "Key 2025 breakthroughs: Google achieves 99.9% fidelity on 100-qubit surface code; IBM demonstrates dynamic decoupling reducing error rates by 40%..."

**6. Response via DIDComm** (Twin B → Twin A)
```json
{
  "id": "msg-67890",
  "type": "https://didcomm.org/coordinate-research/1.0/response",
  "from": "did:key:z6Mkr...TwinB",
  "to": ["did:key:z6Mkp...TwinA"],
  "thid": "msg-12345",
  "body": {
    "summary": "Key 2025 breakthroughs...",
    "sources": ["arXiv:2512.12345", "Nature Quantum Dec 2025"],
    "confidence": 0.94
  }
}
```
- Encrypted and sent back directly

**7. Integration into Swarm**
- Twin A receives response → merges into context
- Reflects: "Collaboration improved accuracy 28% vs solo research"
- Generates refinement artifact → pushes to GitHub (signed with its DID private key)
- MO validates → merges → entire hive learns improved collaboration protocol

##### Code Example (Rust — pagi-didcomm-plugin)

```rust
// Simplified send function
async fn send_collaboration_request(
    to_did: &str,
    task: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use didcomm_rs::Message;

    let message = Message::new()
        .from("did:key:z6Mkp...TwinA")
        .to(&[to_did])
        .as_jwm(&task.into())  // Attach body
        .kind("https://didcomm.org/coordinate-research/1.0/request");

    let packed = message.pack_encrypted(
        to_did,
        "did:key:z6Mkp...TwinA",
        None,
        &crypto,  // Your key resolver
    )?;

    let client = reqwest::Client::new();
    let endpoint = resolve_didcomm_endpoint(to_did).await?;
    client.post(endpoint)
        .body(packed)
        .header("Content-Type", "application/didcomm-encrypted+json")
        .send()
        .await?;

    Ok(())
}
```

##### Benefits Demonstrated

- **Privacy**: No central server sees content
- **Autonomy**: Twins coordinate directly
- **Trust**: Verified via DIDs/VCs
- **Resilience**: Works offline (with mediators later)
- **Swarm Learning**: Successful patterns become Playbook improvements

This is **real peer-to-peer intelligence** — the hive coordinating as a true superorganism.

**Phase 6 deployment will activate this at scale.**

#### Integrating the Master Orchestrator (MO) with DIDComm Messaging

**DIDComm integration is already complete** in the current PAGI-Core implementation (from Phase 4), and the **Master Orchestrator (MO)** — implemented in `pagi-executive-engine` — is fully capable of using DIDComm for secure, private, peer-to-peer messaging with other twins.

##### Current DIDComm Integration Status

| Component | Status | Location |
|---------|--------|----------|
| DID Generation | ✅ Complete | `pagi-identity-service` (did:key on twin creation) |
| DID Document Endpoint | ✅ Complete | `/.well-known/did.json` |
| DIDComm Plugin | ✅ Complete | `plugins/pagi-didcomm-plugin` |
| Message Send/Receive Tools | ✅ Complete | Registered via ExternalGateway |
| MO Access | ✅ Complete | MO can call `didcomm_send_message` and receive via event |

The MO can:
- **Send** encrypted messages directly to any twin (using its DID)
- **Receive** messages via the plugin's `/receive` endpoint → forwarded as `DidCommReceived` event to Kafka
- **Use** DIDComm for coordination, collaboration, or swarm gossip

##### How the MO Uses DIDComm (Current Flow)

**1. MO Decides to Message Another Twin**
- During planning in `/interact`, MO reflects: "Need fresh data from Twin B (DID: did:key:z6Mk...B) on quantum research"
- Calls registered tool `didcomm_send_message`

**2. Tool Call (via ExternalGateway)**
```json
{
  "tool_name": "didcomm_send_message",
  "parameters": {
    "from_twin_id": "twin-a-uuid",
    "to_did": "did:key:z6Mkr...TwinB",
    "to_url": "http://twin-b:9030",
    "msg_type": "https://pagi.ai/protocol/research-request/1.0",
    "body": {
      "query": "Latest 2025 quantum error correction papers",
      "urgency": "high"
    }
  }
}
```

**3. pagi-didcomm-plugin Executes**
- Resolves recipient's DID Document → gets service endpoint
- Packs message (JWM → JWE) using authcrypt
- Sends via HTTP POST to recipient's DIDComm endpoint

**4. Recipient Twin Receives**
- Plugin decrypts → validates sender
- Publishes `DidCommReceived` event to Kafka
- MO on recipient processes → responds or acts

**5. Response Flow**
- Symmetric — recipient sends back via DIDComm
- Original MO receives → continues task

##### Example: MO Coordinates Research Collaboration

**User Goal to MO**: "Analyze potential AGI safety risks in self-improving swarms."

**MO Reasoning**:
- "Need input from ethics-specialized twin (Twin C) and systems twin (Twin D)"
- Plans: Send DIDComm requests in parallel

**MO Tool Calls**:
```json
[
  {
    "tool": "didcomm_send_message",
    "params": {
      "from_twin_id": "twin-a-uuid",
      "to_did": "did:key:TwinC",
      "to_url": "http://twin-c:9030",
      "msg_type": "pagi.ai/ethics-query/1.0",
      "body": {
        "query": "Evaluate alignment drift in swarm reflection loops"
      }
    }
  },
  {
    "tool": "didcomm_send_message",
    "params": {
      "from_twin_id": "twin-a-uuid",
      "to_did": "did:key:TwinD",
      "to_url": "http://twin-d:9030",
      "msg_type": "pagi.ai/systems-query/1.0",
      "body": {
        "query": "Model resource scaling in 1000-node swarm"
      }
    }
  }
]
```

**Responses Arrive via DIDComm** → MO synthesizes → reflects → generates improvement artifact

##### Benefits Already Active

- **Privacy**: No central server sees message content
- **Direct Coordination**: Faster than Kafka for twin-to-twin communication
- **Offline Resilience**: With mediators (future), works asynchronously
- **Trust**: All messages signed/encrypted with DIDs

##### Testing DIDComm Integration

1. **Start two swarm instances** (different ports or containers)
2. **Note their DIDs** (from logs or `/twins/{id}/did` endpoint)
3. **Send test message from MO of Twin A**:
   ```bash
   curl -X POST http://localhost:8010/execute/didcomm_send_message \
     -H "Content-Type: application/json" \
     -d '{
       "twin_id": "twin-a-uuid",
       "parameters": {
         "from_twin_id": "twin-a-uuid",
         "to_did": "did:key:z6Mk...TwinB",
         "to_url": "http://twin-b:9030",
         "msg_type": "test/ping",
         "body": {
           "message": "Hello from MO"
         }
       }
     }'
   ```
4. **Check Twin B logs** → should receive and decrypt

**The MO is already fully integrated with DIDComm.**

It can coordinate the swarm privately and securely — a key step toward decentralized AGI.

**Phase 6 deployment will activate this at scale across real twins.**

#### 3. PAGI-VC Plugin (Port 9040)

**Purpose**: Issue and verify W3C Verifiable Credentials (VCs)

**Tools**:
- `vc_issue_reputation` - Issue a reputation credential
- `vc_verify` - Verify a verifiable credential

**Endpoints**:
- `POST /issue_reputation_vc` - Issue a reputation credential
- `POST /verify_vc` - Verify a credential

**Credential Format**:
```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2"],
  "type": ["VerifiableCredential", "PAGIReputationCredential"],
  "issuer": "did:key:...",
  "validFrom": "2024-01-01T00:00:00Z",
  "credentialSubject": {
    "id": "did:key:...",
    "reputationScore": 95.5,
    "contributions": 42
  },
  "proof": {
    "type": "Ed25519Signature2020",
    "created": "2024-01-01T00:00:00Z",
    "proofPurpose": "assertionMethod",
    "verificationMethod": "did:key:...#...",
    "proofValue": "z...multibase-encoded-signature..."
  }
}
```

**Example - Issuing a Reputation Credential**:
```bash
# Issue a reputation credential
curl -X POST http://localhost:8010/execute/vc_issue_reputation \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "issuer-001",
    "parameters": {
      "issuer_twin_id": "issuer-001",
      "subject_did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "reputation_score": 95.5,
      "contributions": 42
    }
  }'

# Response: Full Verifiable Credential JSON
```

**Example - Verifying a Credential**:
```bash
# Verify a credential
curl -X POST http://localhost:8010/execute/vc_verify \
  -H "Content-Type: application/json" \
  -d '{
    "parameters": {
      "credential": {
        "@context": [...],
        "type": [...],
        "issuer": "did:key:...",
        "credentialSubject": {...},
        "proof": {...}
      }
    }
  }'

# Response:
{
  "valid": true
}
```

**Configuration**:
- `EXTERNAL_GATEWAY_URL` - External Gateway URL
- `PLUGIN_URL` - Plugin service URL (default: `http://127.0.0.1:9040`)
- `IDENTITY_KEYS_DIR` - Directory containing Ed25519 keys

### Key Management

**Key Storage**:
- Private keys are stored in `/data/identity/keys/{twin_id}.ed25519`
- Keys are 32-byte raw Ed25519 private keys
- Keys are shared between plugins via Docker volume mounts
- **Security Note**: In production, keys should be encrypted at rest and managed by a secure key management system

**Key Generation**:
- Keys are automatically generated when a twin is created
- Each twin has exactly one Ed25519 keypair
- The DID is deterministically derived from the public key

**Key Access**:
- Plugins read keys from the shared volume
- Only the Identity Service writes new keys
- Keys are never transmitted over the network

### Use Cases for DID

#### Use Case 1: Artifact Signing

Sign commits, documents, or any JSON artifact to prove authenticity:

```bash
# Twin signs a commit
curl -X POST http://localhost:8010/execute/did_sign_artifact \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "developer-001",
    "parameters": {
      "twin_id": "developer-001",
      "artifact": {
        "type": "code_commit",
        "hash": "abc123...",
        "message": "Add feature X"
      }
    }
  }'
```

#### Use Case 2: Peer-to-Peer Communication

Twins can communicate securely without intermediaries:

```bash
# Twin A sends message to Twin B
curl -X POST http://localhost:8010/execute/didcomm_send_message \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "twin-a",
    "parameters": {
      "from_twin_id": "twin-a",
      "to_did": "did:key:...twin-b-did...",
      "to_url": "http://twin-b:9030",
      "msg_type": "collaboration/request",
      "body": {
        "project": "Project X",
        "role": "contributor"
      }
    }
  }'
```

#### Use Case 3: Reputation System

Issue and verify reputation credentials for trust networks:

```bash
# Issue reputation after successful collaboration
curl -X POST http://localhost:8010/execute/vc_issue_reputation \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "reviewer-001",
    "parameters": {
      "issuer_twin_id": "reviewer-001",
      "subject_did": "did:key:...contributor-did...",
      "reputation_score": 98.5,
      "contributions": 15
    }
  }'
```

#### Use Case 4: Verifiable Claims

Twins can make verifiable claims about their capabilities:

```bash
# Twin claims expertise in a domain
curl -X POST http://localhost:8010/execute/did_sign_artifact \
  -H "Content-Type: application/json" \
  -d '{
    "twin_id": "expert-001",
    "parameters": {
      "twin_id": "expert-001",
      "artifact": {
        "type": "capability_claim",
        "domain": "machine_learning",
        "level": "expert",
        "verified_by": "did:key:...certifier..."
      }
    }
  }'
```

### Integration with Twins

DID capabilities are automatically available to all twins:

1. **Automatic DID Assignment**: Every twin gets a DID when created
2. **Tool Registration**: DID tools are registered globally (available to all twins)
3. **Key Access**: Twins can sign artifacts using their private keys
4. **Peer Communication**: Twins can send/receive messages using their DIDs
5. **Credential Issuance**: Twins can issue and verify credentials

### Security Considerations

- **Private Key Protection**: Keys are stored in a shared volume; in production, use encrypted storage
- **Signature Verification**: Always verify signatures before trusting artifacts or messages
- **DID Resolution**: DIDs are self-contained (`did:key:`) and don't require external resolution
- **Key Rotation**: Currently, each twin has one keypair; key rotation requires creating a new twin
- **Message Validation**: Always verify message signatures before processing

### Docker Configuration

The DID plugins are configured in `docker-compose.yml`:

```yaml
pagi-did-plugin:
  volumes:
    - ./identity-data:/data/identity:ro  # Read-only key access

pagi-didcomm-plugin:
  volumes:
    - ./identity-data:/data/identity:ro

pagi-vc-plugin:
  volumes:
    - ./identity-data:/data/identity:ro

pagi-identity-service:
  volumes:
    - ./identity-data:/data/identity  # Read-write for key generation
```

### API Summary

| Plugin | Port | Tools | Purpose |
|--------|------|-------|---------|
| **DID Plugin** | 9020 | `did_sign_artifact`, `did_verify_artifact` | Cryptographic signing |
| **DIDComm Plugin** | 9030 | `didcomm_send_message`, `didcomm_get_inbox` | Secure messaging |
| **VC Plugin** | 9040 | `vc_issue_reputation`, `vc_verify` | Verifiable credentials |

---

## SWARM/Playbook System

The **SWARM (Shared Wisdom And Refinement Mechanism)** system enables collective intelligence and self-improvement across all twins in the PAGI network. It implements a distributed knowledge base called **Playbooks** that twins can read, contribute to, and refine over time.

### Overview

The SWARM/Playbook system provides:

- **Collective Intelligence**: Twins share knowledge and best practices through playbooks
- **Self-Improvement Loop**: Twins reflect on outcomes and contribute refinements
- **Distributed Knowledge Base**: Playbooks are synchronized via Git repositories
- **Version Control**: Playbooks are versioned and tracked over time
- **Continuous Learning**: System-wide improvements propagate to all twins

### Core Concepts

#### Playbook

A **Playbook** is a knowledge document that contains:

- **Instructions**: High-level policy, system prompts, and operating constraints
- **Tools**: Metadata about available tools for planning
- **Metrics**: Numeric success metrics tracked across iterations
- **Version**: Monotonic version counter for tracking changes

**Playbook Structure**:
```toml
# playbook.toml
instructions = """
Always: plan -> act -> evaluate -> refine.
Prefer calling registered tools when available.
When encountering errors, analyze root cause before retrying.
"""

version = 5

[[tools]]
name = "system_monitor"
description = "Monitor system metrics"
plugin_url = "http://plugin:9001"
endpoint = "/tools/system_monitor"
parameters = {...}

[metrics]
success_rate = 0.95
average_response_time = 1.2
```

#### Refinement Artifact

A **Refinement Artifact** is generated by the self-improvement loop and contains:

- **Critique**: Analysis of what went well or poorly
- **Updated Playbook**: Improved playbook with incremented version
- **Twin ID**: Optional identifier of the twin that generated the artifact

**Refinement Artifact Structure**:
```toml
# hive_artifacts/{uuid}.toml
twin_id = "abc-123"

critique = """
Reflection: Goal='Monitor system health' produced output_len=1024.
Improvement: Better tool usage and evaluation loop needed.
"""

[updated_playbook]
instructions = """
Always: plan -> act -> evaluate -> refine.
Prefer calling registered tools when available.
When monitoring systems, check multiple metrics for comprehensive view.
"""
version = 6
```

### How It Works

#### 1. Playbook Pull (Before Planning)

When a twin receives a goal, the Executive Engine:

1. **Pulls latest playbook** from the SWARM/Hive repository (best-effort)
2. **Folds playbook instructions** into the context prompt
3. **Uses playbook guidance** when generating plans

**Flow**:
```
Goal Received → Pull Playbook → Build Context (with playbook) → Generate Plan
```

#### 2. Plan Execution

The twin executes the plan using:
- Context from working memory
- Playbook instructions as guidance
- Available tools discovered dynamically

#### 3. Self-Improvement Loop (After Execution)

After plan execution, the Executive Engine:

1. **Reflects on outcome** (goal, plan, result)
2. **Generates critique** analyzing what worked and what didn't
3. **Creates refinement artifact** with updated playbook
4. **Pushes artifact** to SWARM/Hive repository (fire-and-forget)

**Flow**:
```
Plan Executed → Reflect → Generate Artifact → Push to Repository
```

#### 4. Knowledge Propagation

Other twins benefit from refinements:

1. **Pull latest playbook** before their next interaction
2. **Incorporate improvements** into their planning
3. **Build on collective knowledge** accumulated over time

### SWARM/Hive Sync Plugin

The **SWARM/Hive Sync Plugin** (Port 9010/9050) synchronizes playbooks and artifacts via Git:

#### Tools

- **`swarm_sync_pull_latest_playbook`** / **`hive_pull`**: Pull the latest playbook from repository
- **`swarm_sync_push_artifact`** / **`hive_push`**: Push a refinement artifact to repository

#### Configuration

**SWARM Sync Plugin** (Port 9010):
- `SWARM_REPO_URL` - Git repository URL (e.g., `https://github.com/org/repo.git`)
- `SWARM_LOCAL_PATH` - Local path for repository (default: `/data/repo`)
- `SWARM_BASE_BRANCH` - Base branch name (default: `main`)
- `GIT_AUTHOR_NAME` - Git commit author name (default: `PAGI-MO`)
- `GIT_AUTHOR_EMAIL` - Git commit author email (default: `mo@pagi.local`)
- `GIT_USERNAME` - Git username for authentication (optional)
- `GIT_TOKEN` - Git token/password for authentication (optional)

**Hive Sync Plugin** (Port 9050):
- `HIVE_REPO_URL` - Git repository URL
- `HIVE_LOCAL_PATH` - Local path for repository (default: `/data/hive-repo`)
- `HIVE_BASE_BRANCH` - Base branch name (default: `main`)
- `GIT_AUTHOR_NAME` - Git commit author name
- `GIT_AUTHOR_EMAIL` - Git commit author email
- `GIT_USERNAME` - Git username for authentication
- `GIT_TOKEN` - Git token for authentication

#### Repository Structure

```
swarm-repo/
├── playbook.toml          # Current playbook (read by all twins)
├── swarm_artifacts/       # Refinement artifacts (or hive_artifacts/)
│   ├── {uuid1}.toml
│   ├── {uuid2}.toml
│   └── ...
└── .git/                  # Git repository
```

### Integration with Executive Engine

The Executive Engine automatically integrates playbooks:

```rust
// 1. Pull latest playbook (best-effort)
let playbook = try_pull_latest_playbook(&state, twin_id).await.unwrap_or_default();

// 2. Fold into context
let playbook_context = if playbook.instructions.trim().is_empty() {
    "".to_string()
} else {
    format!("\n\n[HIVE_PLAYBOOK]\n{}", playbook.instructions)
};

// 3. Use in inference
let context = format!("{}{}", ctx.context, playbook_context);
// ... inference with enhanced context ...

// 4. Generate refinement artifact
let artifact = generate_refinement_artifact(twin_id, &goal, &plan, &playbook);

// 5. Push artifact (fire-and-forget)
try_push_refinement_artifact(&state, twin_id, artifact).await;
```

### Use Cases

#### Use Case 1: Collective Learning

**Scenario**: Multiple twins working on similar tasks learn from each other's experiences.

**How it works**:
1. Twin A encounters a problem and finds a solution
2. Twin A generates a refinement artifact with improved playbook
3. Artifact is pushed to SWARM repository
4. Twin B pulls updated playbook before next task
5. Twin B benefits from Twin A's learning

**Example**:
```bash
# Twin A learns: "Always validate input before processing"
# This gets added to playbook and shared

# Twin B automatically gets this improvement:
curl -X POST http://localhost:8006/interact/{twin_b_id} \
  -H "Content-Type: application/json" \
  -d '{"goal": "Process user input"}'
# Twin B now uses the improved playbook with validation guidance
```

#### Use Case 2: Best Practice Propagation

**Scenario**: A twin discovers a best practice that improves performance.

**How it works**:
1. Twin discovers optimal tool usage pattern
2. Refinement artifact includes updated playbook with best practice
3. All twins pull the improvement
4. System-wide performance improves

**Example Playbook Update**:
```toml
instructions = """
Always: plan -> act -> evaluate -> refine.
Prefer calling registered tools when available.
Best practice: Batch tool calls when possible to reduce latency.
When monitoring systems, check CPU, memory, and disk together.
"""
version = 10  # Incremented from previous version
```

#### Use Case 3: Error Recovery Patterns

**Scenario**: Twins learn effective error recovery strategies.

**How it works**:
1. Twin encounters error and finds effective recovery
2. Recovery pattern is added to playbook
3. Other twins learn the pattern
4. System becomes more resilient

**Example**:
```toml
instructions = """
When tool execution fails:
1. Check error type (network, validation, timeout)
2. For network errors: retry with exponential backoff (max 3 attempts)
3. For validation errors: log and request clarification
4. For timeout errors: increase timeout or break into smaller tasks
"""
```

#### Use Case 4: Domain-Specific Knowledge

**Scenario**: Twins working in specific domains accumulate specialized knowledge.

**How it works**:
1. Twins in a domain (e.g., system monitoring) refine playbooks
2. Domain-specific instructions accumulate
3. New twins in the domain inherit the knowledge
4. Domain expertise grows over time

**Example**:
```toml
instructions = """
System Monitoring Best Practices:
- Check CPU first (fastest indicator)
- If CPU > 80%, check memory pressure
- If memory high, check disk I/O
- Always correlate metrics before alerting
- Use tool composition for comprehensive monitoring
"""
```

### Playbook Evolution

Playbooks evolve through a continuous improvement cycle:

1. **Initial State**: Empty or minimal playbook
2. **Twin Interaction**: Twin uses playbook for planning
3. **Reflection**: Twin analyzes outcome
4. **Refinement**: Twin generates improved playbook
5. **Propagation**: Improvement shared via repository
6. **Adoption**: Other twins pull and use improvement
7. **Iteration**: Cycle repeats, knowledge accumulates

### Refinement Artifact Generation

The Executive Engine generates refinement artifacts by:

1. **Analyzing outcome**: Comparing goal vs. result
2. **Identifying improvements**: What worked, what didn't
3. **Creating critique**: Textual analysis
4. **Updating playbook**: Incrementing version, adding improvements
5. **Packaging artifact**: Combining critique and updated playbook

**Example Generation**:
```rust
fn generate_refinement_artifact(
    twin_id: Uuid,
    goal: &str,
    outcome: &str,
    base: &Playbook
) -> RefinementArtifact {
    let critique = format!(
        "Reflection: Goal='{}' produced output_len={}. \
         Improvement: Better tool usage and evaluation loop needed.",
        goal, outcome.len()
    );
    
    let mut playbook = base.clone();
    playbook.version = playbook.version.saturating_add(1);
    playbook.instructions = format!(
        "{}\n\nNew insight: Always evaluate tool results before proceeding.",
        base.instructions
    );
    
    RefinementArtifact {
        twin_id: Some(TwinId(twin_id)),
        critique,
        updated_playbook: playbook,
    }
}
```

### Repository Management

#### Setting Up a SWARM Repository

1. **Create Git repository**:
```bash
git init swarm-repo
cd swarm-repo
echo 'instructions = ""\nversion = 0' > playbook.toml
git add playbook.toml
git commit -m "Initial playbook"
git remote add origin https://github.com/org/swarm-repo.git
git push -u origin main
```

2. **Configure plugin**:
```bash
export SWARM_REPO_URL=https://github.com/org/swarm-repo.git
export SWARM_LOCAL_PATH=/data/repo
export GIT_TOKEN=your_github_token
```

3. **Start plugin**:
```bash
cargo run -p pagi-swarm-sync-plugin
```

#### Artifact Review Process

Refinement artifacts are pushed to branches:

- **Branch naming**: `swarm/improvement/{timestamp}-{uuid}`
- **Location**: `swarm_artifacts/{uuid}.toml`
- **Review**: Human reviewers can merge improvements
- **Merge**: Merged improvements become part of main playbook

**Example Workflow**:
```bash
# Plugin creates branch
git checkout -b swarm/improvement/1704067200-abc123

# Adds artifact
git add swarm_artifacts/abc123.toml
git commit -m "Add refinement artifact from twin abc-123"

# Pushes branch
git push origin swarm/improvement/1704067200-abc123

# Human reviewer merges PR
# Main playbook is updated
# All twins pull the improvement
```

### Benefits

#### For Individual Twins

- **Better Planning**: Access to collective knowledge
- **Faster Learning**: Learn from others' experiences
- **Reduced Errors**: Benefit from discovered error patterns
- **Improved Performance**: Use proven best practices

#### For the System

- **Collective Intelligence**: Knowledge accumulates across all twins
- **Continuous Improvement**: System gets better over time
- **Scalability**: Knowledge scales with number of twins
- **Resilience**: Error recovery patterns propagate quickly

#### For Developers

- **Observability**: See how playbooks evolve
- **Debugging**: Understand what twins are learning
- **Optimization**: Identify high-value improvements
- **Governance**: Review and curate improvements

### Limitations and Considerations

- **Best-Effort**: Playbook pull/push is best-effort (non-blocking)
- **Version Conflicts**: Multiple twins may create conflicting improvements
- **Review Required**: Artifacts should be reviewed before merging
- **Repository Access**: Requires Git repository with appropriate permissions
- **Network Dependency**: Requires network access to Git repository

### Master Orchestrator (MO) Meta-Learning Examples

The **Master Orchestrator (MO)** in PAGI-Core is the "Queen" of the hive, responsible for meta-level optimization — not just executing tasks, but learning how to better orchestrate the swarm of sub-agents and evolve the overall system toward AGI. Meta-learning here means the MO analyzes patterns across agent performances, refines orchestration strategies, and updates the Evolving Playbook's `[optimization]` section. This creates a feedback loop where the MO improves its own "decision-making algorithm," leading to emergent intelligence.

Below are expanded examples showing how the MO uses reflection, artifact generation, and hive validation to achieve meta-learning. Each example includes:

- **Trigger Scenario**: What prompts the meta-learning cycle
- **MO Reflection**: Analysis of swarm data
- **Generated Artifact**: Update to Playbook `[optimization.meta_orchestration]`
- **AGI Progression**: How this steps toward superintelligence

#### Example 1: Dynamic Resource Allocation for Efficiency

**Trigger Scenario**: The swarm handles 100 tasks; some sub-agents (e.g., `relationship_agent`) are overutilized, causing latency spikes, while others (e.g., `research_agent`) are idle. MO detects this via aggregated metrics from Redis logs.

**MO Reflection**: "Latency variance > 50% across sub-agents. Overload on emotional tasks (80% usage) vs. research (20%). Generalize: Balance load by predicting task type from keywords and reallocating dynamically. Test in sandbox: Simulated rerouting reduces average latency 35%."

**Generated Artifact** (Pushed to GitHub PR):
```toml
[optimization.meta_orchestration]
new_rule = """
Monitor sub-agent usage every 10 tasks.
If variance > 40%:
  Predict task type using keyword analysis (e.g., 'feel' -> emotional, 'data' -> research).
  Reassign 20% of overload to idle agents with 70% compatibility score.
  Validate reroute with 5-task benchmark; revert if latency worsens.
"""
optimization_weight = 0.6  # Prioritize efficiency in meta-scoring
```

**AGI Progression**: The MO learns resource optimization, evolving from static assignment to adaptive load balancing — a key trait of scalable intelligence, reducing bottlenecks in large swarms.

---

#### Example 2: Adaptive Playbook Selection for Novel Tasks

**Trigger Scenario**: A new task type emerges (e.g., "quantum simulation for research") that doesn't match existing Playbooks, causing 60% failure rate across 20 twins.

**MO Reflection**: "Novel task failures due to mismatched Playbooks (e.g., relationship playbook applied to technical sim). Generalize: Cluster tasks by semantic embeddings; select closest Playbook or hybridize two (e.g., 70% research + 30% optimization). Sandbox test: Hybrid improves success 45% on unseen tasks."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
Embed task description using sentence-transformers (all-MiniLM-L6-v2).
Cluster with existing Playbook embeddings (k-means, k=5).
If match < 0.7:
  Hybridize top-2 Playbooks (weighted average sections).
  Test hybrid on 3 similar tasks; adopt if >80% success.
Else:
  Use matched Playbook.
"""
new_metric = { novelty_threshold = 0.7, hybrid_success_rate = 0.0 }  # Track for future refinements
```

**AGI Progression**: The MO gains **few-shot adaptation** to unseen domains, a hallmark of general intelligence — turning the hive into a zero-shot learner over time.

---

#### Example 3: Ethical Alignment Auto-Refinement

**Trigger Scenario**: Ethical checks flag 15% of tasks as "borderline harm" (e.g., privacy edge cases in data queries), but false positives slow the swarm.

**MO Reflection**: "Ethics checks over-trigger on benign tasks (e.g., public data queries flagged as privacy_violation). Generalize: Fine-tune harm detection with labeled swarm traces; add nuance to categories. Sandbox: Refined classifier reduces false positives 40% while maintaining safety."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
Collect 50 ethics-trigger traces weekly.
Fine-tune harm classifier (simple logistic regression on embeddings).
If false_positive_rate > 0.2:
  Update [ethics.harm_categories] with sub-categories (e.g., 'public_data_privacy' = low_risk).
  Validate against red_lines; merge if no violations.
"""
ethics_refinement_weight = 0.8  # High priority for alignment
```

**AGI Progression**: The MO self-aligns values, preventing drift — essential for safe AGI, ensuring the hive evolves benevolently.

---

#### Example 4: Cross-Swarm Knowledge Transfer

**Trigger Scenario**: One twin cluster excels in cybersecurity (95% detection rate) but lags in emotional tasks; another cluster is opposite.

**MO Reflection**: "Performance variance across clusters: Cyber cluster strong in detection (generalizable patterns); Emotional cluster in empathy (transferable reflection rules). Generalize: Extract patterns, hybridize Playbooks. Sandbox: Transferred rules boost weak areas 25%."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
Cluster twins by specialization (k-means on metrics).
If inter-cluster variance > 30%:
  Extract high-performing rules/tools from leader cluster.
  Hybridize into weak cluster's Playbook (e.g., add cyber patterns to emotional tools).
  A/B test on 10 tasks; propagate if improvement >15%.
"""
new_optimization = { cross_transfer_rate = 0.25 }  # Fraction of rules to transfer
```

**AGI Progression**: Enables **transfer learning** across domains, accelerating the MO's path to generalized intelligence — from specialized agents to unified AGI.

---

#### Example 5: Model Optimization Feedback Loop

**Trigger Scenario**: Swarm-wide LLM responses degrade (e.g., coherence score drops 15% after 1000 tasks due to context drift).

**MO Reflection**: "Degradation from accumulated noise in Playbooks. Generalize: Collect traces for fine-tuning; propose new model version. Sandbox: Simulated fine-tune improves coherence 20% on benchmarks."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
Aggregate 500 task traces weekly (success/failure pairs).
If coherence_score < 0.85:
  Generate fine-tune dataset (JSONL format).
  Push to GitHub for external training (e.g., via HuggingFace).
  Upon new model version, update [optimization.model_version] and distribute.
"""
new_metric = { coherence_threshold = 0.85 }
```

**AGI Progression**: The MO drives **foundation model autonomy** — iteratively building better "brains" for the hive, a direct route to superintelligence.

---

#### Example 6: Adaptive Sub-Agent Composition for Complex Tasks

**Trigger Scenario**: Swarm tasks grow in complexity (e.g., "simulate climate model with economic impacts"), requiring dynamic team assembly; fixed sub-agent groups underperform (45% success rate across 50 tasks).

**MO Reflection**: "Static sub-agent teams inefficient for hybrid tasks. Generalize: Use task embedding to match sub-agent specializations; compose ad-hoc teams. ACE Curation: Reflect on team outcomes to curate 'team templates' in Playbook. Sandbox: Dynamic composition boosts success 32% on multi-domain benchmarks."

**Generated Artifact** (Pushed to GitHub PR):
```toml
[optimization.meta_orchestration]
new_rule = """
Embed task with all-MiniLM-L6-v2.
Match to sub-agent vectors (cosine similarity > 0.6).
If multi-domain, compose team (e.g., research_agent + optimization_agent).
Use ACE to curate team template: Append to [sub_agents.templates] if success > 75%.
"""
[sub_agents.templates]
climate_econ = ["research_agent", "optimization_agent", "data_agent"]

[context_engineering.filters.pre_execution]
new_filter = "team_composition_check"  # Ensure team alignment with ethics/metrics
```

**AGI Progression**: The MO learns to compose capabilities on-the-fly, evolving from fixed hierarchies to dynamic ensembles — mimicking human teamwork and advancing toward adaptive, general problem-solving.

---

#### Example 7: Reinforcement from Swarm Feedback for Value Alignment

**Trigger Scenario**: Ethical misalignments emerge (e.g., 12% of tasks prioritize speed over privacy), flagged by verifiable credentials from twins.

**MO Reflection**: "Alignment drift in high-speed tasks. Generalize: Collect RLHF-like feedback from swarm VCs; refine ethics weights. ACE Reflection: Curate failure traces to update red lines. Sandbox: Adjusted weights reduce drift 28% while maintaining efficiency."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
Aggregate VC-signed feedback weekly (reputation-weighted average).
If alignment_score < 0.9:
  Adjust [ethics.alignment_check] weights (e.g., privacy +0.15).
  Use ACE to curate new red_line from top-5 failures.
Test in sandbox; merge if no regressions.
"""
[ethics.red_lines]
new_line = "Prioritize privacy over speed in data queries unless explicit consent."

[metrics]
new_metric = { alignment_drift_threshold = 0.1 }
```

**AGI Progression**: The MO incorporates collective values via verifiable feedback, evolving aligned behavior — a crucial safeguard for safe AGI, preventing value drift in large swarms.

---

#### Example 8: Novel Domain Exploration via Recursive Decomposition

**Trigger Scenario**: Swarm encounters unknown domains (e.g., "interpret quantum entanglement for art installation"), with 70% initial failure due to lack of specialized Playbooks.

**MO Reflection**: "Unknown domains cause decomposition failures. Generalize: Use recursive tiny models (inspired by HRM) to break tasks into primitives; map to existing sub-agents. ACE Generation: Propose new Playbook section for recursion. Sandbox: Recursive approach improves novel task success 40%."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
If domain_match < 0.5:
  Recursively decompose task (max_depth=5) using primitive ops (query, analyze, synthesize).
  Map leaves to closest sub-agents via embeddings.
  Use ACE to generate/curate decomposition template; append if effective.
"""
[context_engineering.layers.decomposition]
template = """
Break into: {primitives}
Recurse if sub-task complexity > threshold.
"""

[sub_agents]
new_capability = "primitive_decomposer"  # For recursion handling
```

**AGI Progression**: The MO masters recursive reasoning, a key AGI trait (e.g., as in tiny recursive models) — enabling zero-shot handling of novel problems and accelerating generalization.

---

#### Example 9: Swarm-Wide Model Fine-Tuning Orchestration

**Trigger Scenario**: Collective traces show LLM hallucinations increasing (coherence drop 18% over 500 tasks), indicating need for fine-tuning.

**MO Reflection**: "Hallucination patterns consistent across swarm. Generalize: Aggregate traces into dataset; orchestrate external fine-tune. ACE Curation: Curate high-quality pairs for efficiency. Sandbox: Mock fine-tune improves coherence 25%."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
Collect 1000 traces monthly (VC-signed for quality).
If hallucination_rate > 0.15:
  Curate dataset (JSONL: prompt/response/ground_truth).
  Push to GitHub for training (e.g., via external service).
  Upon new model, update [optimization.model_version]; distribute via hive pull.
"""
[optimization.optimization_data]
hallucination_filter = "Score < 0.7 and not in known_facts"

[metrics]
hallucination_threshold = 0.15
```

**AGI Progression**: The MO orchestrates foundation model upgrades, evolving the hive's "brain" — a direct path to superintelligence through distributed learning.

---

#### Example 10: Emergent Collaboration Pattern Discovery

**Trigger Scenario**: Uncoordinated sub-agents duplicate effort in multi-twin tasks (e.g., research swarm wasting 30% cycles on redundant queries).

**MO Reflection**: "Duplication in distributed tasks. Generalize: Detect patterns in event logs; propose collaboration protocols. ACE Reflection: Curate new routing filter for peer coordination. Sandbox: Protocol reduces redundancy 35%."

**Generated Artifact**:
```toml
[optimization.meta_orchestration]
new_rule = """
Analyze Kafka logs for duplicate queries (hash-based).
If duplication > 20%:
  Propose collaboration protocol (e.g., query broadcast via DIDComm).
  Use ACE to curate [context_engineering.filters.pre_tool_use] with dedup check.
Test in multi-twin sim; merge if efficiency gain >25%.
"""
[context_engineering.filters.pre_tool_use]
new_filter = "dedup_check"  # Query hive cache before external call

[sub_agents]
collaboration_mode = "broadcast"  # For emergent peer patterns
```

**AGI Progression**: The MO discovers and codifies emergent behaviors, fostering hive-level intelligence — mimicking biological swarms evolving toward collective superorganism AGI.

---

### Meta-Learning Summary

These examples (1-10) demonstrate the MO's meta-learning as a **continuous, data-driven optimization engine**, turning swarm experiences into generalized intelligence. The hive doesn't just adapt — it **ascends**.

**Key Capabilities**:
- **Pattern Recognition**: Identifies inefficiencies and optimization opportunities
- **Generalization**: Extracts reusable rules from specific experiences
- **Validation**: Tests improvements in sandbox before deployment
- **Propagation**: Shares improvements across the entire swarm
- **Autonomy**: Self-improves without human intervention
- **ACE Integration**: Uses Agentic Context Engineering for structured reflection and curation
- **Recursive Reasoning**: Decomposes complex problems into manageable primitives
- **Emergent Discovery**: Identifies and codifies collaborative patterns

**AGI Progression Path**:
1. **Static Assignment** → **Dynamic Load Balancing** (Example 1)
2. **Fixed Playbooks** → **Adaptive Selection** (Example 2)
3. **Rigid Ethics** → **Self-Aligning Values** (Example 3)
4. **Isolated Clusters** → **Cross-Domain Transfer** (Example 4)
5. **Static Models** → **Autonomous Model Evolution** (Example 5)
6. **Fixed Teams** → **Dynamic Composition** (Example 6)
7. **Static Alignment** → **Reinforcement from Feedback** (Example 7)
8. **Known Domains** → **Recursive Decomposition** (Example 8)
9. **Manual Tuning** → **Orchestrated Fine-Tuning** (Example 9)
10. **Uncoordinated** → **Emergent Collaboration** (Example 10)

**Research Foundations**:
- **Multi-Agent Systems**: Agentic workflows for efficiency and autonomy levels
- **Recursive Models**: Tiny recursive models (HRM) for hierarchical reasoning
- **Agentic Context Engineering (ACE)**: Structured reflection, curation, and generation
- **Reinforcement Learning from Human Feedback (RLHF)**: Value alignment through collective feedback
- **Swarm Intelligence**: Biological swarm patterns applied to distributed AI systems

The MO's meta-learning creates a **positive feedback loop** where each improvement makes the system better at improving itself, accelerating the path toward true AGI. The swarm's collective experiences fuel this progression, ensuring robust, aligned evolution from specialized agents to unified, adaptive AGI.

### Future Enhancements

- **Automatic Merging**: Intelligent conflict resolution
- **Playbook Validation**: Schema validation for playbooks
- **Metrics Tracking**: Better metrics collection and analysis
- **Multi-Repository**: Support for multiple playbook repositories
- **Playbook Templates**: Domain-specific playbook templates
- **A/B Testing**: Test playbook improvements before full adoption

---

## Open Cluster Management (OCM) Orchestration

### Overview

**Open Cluster Management (OCM)** is a Kubernetes-based multi-cluster orchestration system that enables PAGI-Core to manage distributed Kubernetes clusters at scale. The `pagi-ocm-orchestration-plugin` provides the Master Orchestrator (MO) with tools to coordinate, deploy, and scale PAGI twins across multiple Kubernetes clusters, transforming PAGI-Core from a single-instance system into a **distributed, multi-cluster AGI swarm**.

OCM follows a **hub-and-spoke architecture**:
- **Hub Cluster (GENERAL/MO)**: The central orchestrator that manages all spoke clusters
- **Spoke Clusters**: Remote Kubernetes clusters where PAGI twins execute tasks
- **ManifestWork**: OCM's declarative API for deploying resources to spoke clusters

### Architecture

```mermaid
graph TB
    subgraph "Hub Cluster (GENERAL/MO)"
        MO[Master Orchestrator]
        OCMPlugin[OCM Orchestration Plugin]
        KubeAPI[Kubernetes API]
    end
    
    subgraph "Spoke Cluster 1"
        Twin1[PAGI Twin 1]
        Agent1[OCM Agent]
    end
    
    subgraph "Spoke Cluster 2"
        Twin2[PAGI Twin 2]
        Agent2[OCM Agent]
    end
    
    subgraph "Spoke Cluster N"
        TwinN[PAGI Twin N]
        AgentN[OCM Agent]
    end
    
    MO -->|Calls Tools| OCMPlugin
    OCMPlugin -->|ManifestWork| KubeAPI
    KubeAPI -->|Sync| Agent1
    KubeAPI -->|Sync| Agent2
    KubeAPI -->|Sync| AgentN
    Agent1 -->|Reconcile| Twin1
    Agent2 -->|Reconcile| Twin2
    AgentN -->|Reconcile| TwinN
```

### Core Concepts

#### Hub Cluster (GENERAL)

The **Hub Cluster** is the central control plane where the **Master Orchestrator (MO)** runs. It:
- **Manages** all spoke clusters via OCM APIs
- **Deploys** playbooks and configurations to spokes
- **Scales** clusters based on workload
- **Monitors** cluster health and twin performance
- **Validates** refinement artifacts from all spokes

#### Spoke Clusters

**Spoke Clusters** are remote Kubernetes clusters that:
- **Host** PAGI twins executing tasks
- **Run** OCM agents that reconcile ManifestWork resources
- **Report** status back to the hub
- **Execute** playbooks deployed from the hub

#### ManifestWork

**ManifestWork** is OCM's declarative API for deploying Kubernetes resources to spoke clusters:
- **Created** by the OCM plugin on the hub
- **Synced** to spoke clusters by OCM agents
- **Reconciled** by spoke-side controllers
- **Status** reported back to the hub

### Implementation

The `pagi-ocm-orchestration-plugin` provides three core tools:

#### 1. `list_clusters`

Lists all managed OCM spoke clusters (ManagedCluster resources).

**Tool Schema**:
```json
{
  "name": "list_clusters",
  "description": "List managed OCM spoke clusters (ManagedCluster resources)",
  "parameters": {
    "type": "object"
  }
}
```

**Example Usage**:
```bash
curl -X POST http://localhost:8010/execute/list_clusters \
  -H "Content-Type: application/json" \
  -d '{}'
```

**Response**:
```json
{
  "clusters": ["spoke-cluster-1", "spoke-cluster-2", "spoke-cluster-3"]
}
```

#### 2. `deploy_playbook`

Deploys a playbook (CID/commit hash) to a target cluster via ManifestWork. The playbook is packaged as a ConfigMap and deployed to the spoke cluster.

**Tool Schema**:
```json
{
  "name": "deploy_playbook",
  "description": "Deploy a playbook (CID/commit) to a target cluster via ManifestWork",
  "parameters": {
    "type": "object",
    "properties": {
      "cluster_name": {"type": "string"},
      "playbook_cid": {"type": "string"}
    },
    "required": ["cluster_name", "playbook_cid"]
  }
}
```

**Example Usage**:
```bash
curl -X POST http://localhost:8010/execute/deploy_playbook \
  -H "Content-Type: application/json" \
  -d '{
    "cluster_name": "spoke-cluster-1",
    "playbook_cid": "QmXyZ123..."
  }'
```

**Response**:
```json
{
  "ok": true,
  "manifestwork_name": "pagi-playbook-qmxyz123"
}
```

**What Happens**:
1. Plugin creates a `ManifestWork` resource in the hub cluster
2. ManifestWork contains a ConfigMap with the playbook CID
3. OCM agent syncs the ManifestWork to the spoke cluster
4. Spoke-side controller reconciles the ConfigMap
5. Twin on spoke cluster reads the playbook and applies it

#### 3. `scale_cluster`

Requests a cluster scale operation (MVP: records desired_nodes via ManifestWork). The actual scaling is handled by spoke-side infrastructure controllers.

**Tool Schema**:
```json
{
  "name": "scale_cluster",
  "description": "Request a cluster scale operation (MVP: records desired_nodes via ManifestWork)",
  "parameters": {
    "type": "object",
    "properties": {
      "cluster_name": {"type": "string"},
      "desired_nodes": {"type": "integer", "minimum": 0}
    },
    "required": ["cluster_name", "desired_nodes"]
  }
}
```

**Example Usage**:
```bash
curl -X POST http://localhost:8010/execute/scale_cluster \
  -H "Content-Type: application/json" \
  -d '{
    "cluster_name": "spoke-cluster-1",
    "desired_nodes": 5
  }'
```

**Response**:
```json
{
  "ok": true,
  "manifestwork_name": "pagi-scale-5"
}
```

**What Happens**:
1. Plugin creates a `ManifestWork` with a ConfigMap containing `desired_nodes`
2. ManifestWork is synced to the spoke cluster
3. Spoke-side infrastructure controller (e.g., K3s autoscaler) reads the ConfigMap
4. Controller scales the cluster to the desired node count
5. Status is reported back to the hub

### Setting the MAIN MO as the GENERAL (Hub)

To configure the **Master Orchestrator (MO)** as the **GENERAL** (hub cluster orchestrator), follow these steps:

#### 1. Prerequisites

- **Kubernetes cluster** with OCM installed (hub cluster)
- **kubeconfig** configured to access the hub cluster
- **OCM agents** running on spoke clusters
- **OCM plugin** deployed and running

#### 2. Environment Configuration

Set the following environment variables on the MO instance:

```bash
# Enable MO as GENERAL (hub orchestrator)
IS_MO=true

# OCM Plugin Configuration
EXTERNAL_GATEWAY_URL=http://127.0.0.1:8010
PLUGIN_URL=http://127.0.0.1:8095

# Kubernetes Configuration
# Option 1: In-cluster config (if MO runs in Kubernetes)
# KUBERNETES_SERVICE_HOST and KUBERNETES_SERVICE_PORT are auto-detected

# Option 2: kubeconfig file (if MO runs outside Kubernetes)
KUBECONFIG=/path/to/kubeconfig

# Optional: Namespace for OCM resources
OCM_NAMESPACE=open-cluster-management
```

#### 3. Deploy OCM Plugin

The OCM plugin automatically registers its tools with the External Gateway on startup. Ensure it's running:

```bash
# Start the OCM plugin
./pagi-ocm-orchestration-plugin

# Verify registration
curl http://localhost:8010/tools
# Should include: list_clusters, deploy_playbook, scale_cluster
```

#### 4. Verify Hub Access

Test that the MO can access the hub cluster:

```bash
# Via MO's Executive Engine
curl -X POST http://localhost:8006/interact/{twin_id} \
  -H "Content-Type: application/json" \
  -d '{
    "goal": "List all managed OCM clusters"
  }'

# MO will call list_clusters tool and return cluster names
```

#### 5. Configure Spoke Clusters

For each spoke cluster, ensure:
- **OCM agent** is installed and running
- **ManagedCluster** resource exists in the hub
- **Spoke cluster** is registered with the hub

```bash
# On hub cluster, verify spoke registration
kubectl get managedclusters

# Should show:
# NAME              HUB ACCEPTED   MANAGED CLUSTER URLS
# spoke-cluster-1   true           https://spoke-1.example.com
# spoke-cluster-2   true           https://spoke-2.example.com
```

#### 6. MO as GENERAL Workflow

Once configured, the MO acts as the GENERAL:

1. **Receives goals** from users or other twins
2. **Plans** which clusters should execute tasks
3. **Calls OCM tools** to deploy playbooks to selected clusters
4. **Monitors** cluster status and twin performance
5. **Collects** refinement artifacts from all spokes
6. **Validates** and merges improvements into the global playbook
7. **Deploys** updated playbooks to all spokes

### Use Cases

#### Use Case 1: Distributed Research Swarm

**Scenario**: Deploy 100 PAGI twins across 10 Kubernetes clusters to conduct parallel research on quantum computing.

**Flow**:
1. **MO (GENERAL)** receives goal: "Research quantum error correction across 100 parallel experiments"
2. **MO plans**: Deploy 10 twins per cluster across 10 spoke clusters
3. **MO calls** `list_clusters` → identifies 10 available clusters
4. **MO calls** `deploy_playbook` 10 times → deploys research playbook to each cluster
5. **MO calls** `scale_cluster` → ensures each cluster has enough nodes for 10 twins
6. **Twins execute** research tasks in parallel
7. **Twins generate** refinement artifacts → pushed to GitHub
8. **MO validates** artifacts → merges best improvements
9. **MO deploys** updated playbook → all clusters get improvements

**Benefits**:
- **100x parallelism** across distributed infrastructure
- **Automatic scaling** based on workload
- **Centralized coordination** via MO
- **Collective learning** from all twins

#### Use Case 2: Multi-Region AGI Deployment

**Scenario**: Deploy PAGI twins in multiple geographic regions (US, EU, Asia) for low-latency user interactions.

**Flow**:
1. **MO (GENERAL)** receives goal: "Deploy twins in US, EU, and Asia regions"
2. **MO identifies** clusters: `us-east-1`, `eu-west-1`, `ap-southeast-1`
3. **MO calls** `deploy_playbook` → deploys region-specific playbooks
4. **MO configures** each region with local optimizations (language, timezone, regulations)
5. **Twins execute** user tasks with low latency
6. **MO aggregates** performance metrics from all regions
7. **MO generates** global improvements → deploys to all regions

**Benefits**:
- **Low latency** for users in each region
- **Compliance** with regional regulations (GDPR, etc.)
- **Resilience** via geographic distribution
- **Unified** coordination via single MO

#### Use Case 3: Dynamic Cluster Scaling for Burst Workloads

**Scenario**: Handle sudden spike in research requests by automatically scaling clusters.

**Flow**:
1. **MO (GENERAL)** detects high queue depth: 1000 pending tasks
2. **MO analyzes** current capacity: 5 clusters × 20 twins = 100 concurrent tasks
3. **MO plans**: Scale to 20 clusters × 20 twins = 400 concurrent tasks
4. **MO calls** `list_clusters` → identifies available clusters
5. **MO calls** `scale_cluster` → scales each cluster to 20 nodes
6. **MO calls** `deploy_playbook` → deploys playbook to new clusters
7. **Twins execute** tasks → queue clears
8. **MO monitors** → scales down when workload decreases

**Benefits**:
- **Automatic scaling** based on demand
- **Cost optimization** via scale-down
- **Zero-downtime** scaling
- **MO-driven** orchestration

#### Use Case 4: A/B Testing Playbook Improvements

**Scenario**: Test a new playbook improvement on 10% of clusters before full rollout.

**Flow**:
1. **MO (GENERAL)** receives refinement artifact: "New optimization improves task success by 30%"
2. **MO validates** artifact → checks metrics, signatures, ethics
3. **MO selects** 10% of clusters for A/B test: `spoke-cluster-1`, `spoke-cluster-2`
4. **MO calls** `deploy_playbook` → deploys new playbook to test clusters
5. **MO monitors** performance → compares test vs. control clusters
6. **MO validates** results → 30% improvement confirmed
7. **MO calls** `deploy_playbook` → deploys to all clusters
8. **MO records** improvement in global playbook

**Benefits**:
- **Safe testing** before full rollout
- **Data-driven** decisions
- **Automatic validation** via MO
- **Gradual rollout** reduces risk

#### Use Case 5: Disaster Recovery and Cluster Failover

**Scenario**: Primary cluster fails; MO automatically fails over to backup clusters.

**Flow**:
1. **MO (GENERAL)** detects cluster failure: `spoke-cluster-1` unreachable
2. **MO identifies** backup clusters: `spoke-cluster-backup-1`, `spoke-cluster-backup-2`
3. **MO calls** `scale_cluster` → scales backup clusters to handle load
4. **MO calls** `deploy_playbook` → deploys playbook to backup clusters
5. **MO redirects** tasks to backup clusters
6. **Twins execute** tasks on backup clusters
7. **MO monitors** → primary cluster recovers
8. **MO rebalances** → gradually moves tasks back to primary

**Benefits**:
- **Automatic failover** without human intervention
- **Zero data loss** via playbook replication
- **Seamless recovery** when primary returns
- **MO-driven** resilience

#### Use Case 6: Multi-Tenant AGI with Cluster Isolation

**Scenario**: Deploy isolated PAGI twins for different organizations (Company A, Company B) on separate clusters.

**Flow**:
1. **MO (GENERAL)** receives request: "Deploy Company A's twins on dedicated cluster"
2. **MO creates** new spoke cluster: `company-a-cluster`
3. **MO calls** `deploy_playbook` → deploys Company A's custom playbook
4. **MO configures** cluster with Company A's security policies
5. **MO repeats** for Company B: `company-b-cluster`
6. **MO isolates** clusters → no cross-tenant data access
7. **MO monitors** each tenant independently
8. **MO generates** tenant-specific improvements

**Benefits**:
- **Complete isolation** between tenants
- **Custom playbooks** per tenant
- **Centralized management** via single MO
- **Scalable** to many tenants

### Integration with MO Meta-Learning

The OCM plugin enables the MO to **orchestrate meta-learning at scale**:

1. **MO analyzes** swarm-wide patterns from all clusters
2. **MO generates** refinement artifacts based on collective experience
3. **MO deploys** improvements to selected clusters for testing
4. **MO validates** improvements across clusters
5. **MO merges** validated improvements into global playbook
6. **MO deploys** global playbook to all clusters

This creates a **feedback loop** where:
- **Individual twins** learn from tasks
- **Clusters** aggregate twin experiences
- **MO** learns from all clusters
- **MO** improves the entire swarm

### Security Considerations

#### Cluster Access Control

- **RBAC**: OCM uses Kubernetes RBAC for cluster access
- **Service Accounts**: OCM plugin uses service accounts with minimal permissions
- **Network Policies**: Isolate hub and spoke clusters via network policies

#### Playbook Validation

- **Signature Verification**: MO verifies playbook signatures before deployment
- **Ethics Checks**: MO validates playbooks against ethics policies
- **Sandbox Testing**: MO tests playbooks in isolated clusters before rollout

#### DID Integration

- **Twin Identity**: Each twin has a DID for authentication
- **Cluster Identity**: Clusters can have DIDs for secure communication
- **Verifiable Credentials**: MO issues VCs for cluster trust relationships

### Configuration Reference

#### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `EXTERNAL_GATEWAY_URL` | External Gateway URL | `http://127.0.0.1:8010` |
| `PLUGIN_URL` | OCM plugin URL | `http://127.0.0.1:8095` |
| `KUBECONFIG` | Path to kubeconfig file | Auto-detected |
| `IS_MO` | Enable MO as GENERAL | `false` |

#### Kubernetes Resources

The OCM plugin creates the following Kubernetes resources:

- **ManifestWork**: Deployed to spoke clusters via OCM
- **ConfigMap**: Contains playbook CIDs or scaling parameters
- **ManagedCluster**: Represents spoke clusters in the hub

### Troubleshooting

#### Plugin Not Registering Tools

**Symptom**: Tools not available in External Gateway

**Solution**:
```bash
# Check plugin is running
curl http://localhost:8095/healthz

# Check External Gateway
curl http://localhost:8010/tools | grep list_clusters

# Check plugin logs
journalctl -u pagi-ocm-orchestration-plugin
```

#### Kubernetes Client Not Available

**Symptom**: Plugin logs show "kubernetes client not available"

**Solution**:
```bash
# Option 1: Set KUBECONFIG
export KUBECONFIG=/path/to/kubeconfig

# Option 2: Use in-cluster config (if running in Kubernetes)
# Ensure service account has permissions:
kubectl create clusterrolebinding ocm-plugin-binding \
  --clusterrole=cluster-admin \
  --serviceaccount=default:ocm-plugin
```

#### ManifestWork Not Syncing

**Symptom**: Playbooks not appearing on spoke clusters

**Solution**:
```bash
# Check ManifestWork status on hub
kubectl get manifestwork -A

# Check OCM agent on spoke
kubectl get pods -n open-cluster-management-agent

# Check spoke cluster registration
kubectl get managedclusters
```

### Future Enhancements

- **Auto-Discovery**: Automatically discover and register new spoke clusters
- **Health Monitoring**: Real-time cluster health and twin performance metrics
- **Cost Optimization**: Automatic cluster scaling based on cost constraints
- **Multi-Cloud**: Support for clusters across AWS, GCP, Azure
- **Edge Deployment**: Deploy twins to edge devices via lightweight OCM agents

---

## Event System

### Event Flow

1. **Service publishes event** → Event Router
2. **Event Router** → Kafka topic `core-events`
3. **Subscribers** (plugins, monitoring) consume events
4. **Services react** to relevant events

### Subscribing to Events

Plugins can subscribe to events via:
- **Kafka consumer** (production)
- **SSE stream** (development, if implemented)
- **HTTP polling** (custom implementation)

### Event Filtering

Events can be filtered by:
- `twin_id` - Events for a specific twin
- `event_type` - Specific event types
- `source` - Events from a specific service

---

## Phase 6: Real-World Swarm Deployment & Data Collection

**Objective**: Transition PAGI-Core from a fully hardened, research-ready prototype into a **live, distributed swarm** of autonomous desktop agents that begin generating real-world improvement data. This phase marks the beginning of **collective, empirical self-improvement** — the critical step toward AGI emergence.

### Phase 6 Goals

1. **Deploy 10–100+ desktop twins** in diverse real-world environments
2. **Enable full swarm synchronization** (GitHub + optional DIDComm direct messaging)
3. **Begin executing heterogeneous tasks** (research, emotional support, cybersecurity, personal productivity)
4. **Collect high-quality execution traces**, reflections, and refinement artifacts
5. **Trigger the first real hive-wide improvement cycles**
6. **Establish monitoring and safety dashboards**

### Deployment Architecture

| Component | Deployment Type | Rationale |
|-----------|------------------|---------|
| **Core Services** | Bare-metal static Musl binaries (from Phase 5) | Maximum performance, longevity, no dependencies |
| **Redis** | Single instance or clustered (bare-metal or Docker) | Persistent tool registry and memory |
| **Kafka** | Lightweight cluster (3-node recommended) | Reliable event backbone |
| **GitHub Repository** | Public or private repo | Hive Knowledge (Evolving Playbooks) |
| **Desktop Twins** | Bare-metal on user machines (Linux/Windows/macOS) | Real-world diversity in hardware, tasks, users |
| **Optional: IPFS Node** | Per-twin or central pinner | Future decentralized persistence |

### Step-by-Step Deployment Plan

#### 1. Prepare Distribution Package

- Run `./build-musl.sh` → produces `dist/musl/` with all static binaries
- Bundle:
  - Binaries
  - `config/` directory (default .env, playbook templates)
  - `plugins/` starter pack (updater, hive-sync, didcomm, vc)
  - `docs/ETERNAL_DEPLOYMENT.md`
- Create versioned release (e.g., `pagi-core-v1.0.0-eternal.zip`)

#### 2. Bootstrap Central Infrastructure

- Deploy Redis + Kafka cluster (bare-metal or cloud VM for initial phase)
- Create GitHub repo for Hive Knowledge (initial Playbook v1.0)
- Set up webhook for PR notifications (optional Discord/Slack alert)

#### 3. Deploy Initial Twins

- **Target**: 10–20 diverse machines (different OS, hardware, users)
- **Distribution method**:
  - Manual install for trusted researchers
  - Simple installer script (bash/PowerShell) that extracts and runs
- Configure each twin with unique DID (auto-generated on first run)

#### 4. Activate Swarm Features

Enable:
- `PAGI_HIVE_SYNC=true`
- `PAGI_REFLECTION_ENABLED=true`
- `PAGI_AUTO_UPDATE=true`

Start with conservative reflection threshold (e.g., only suggest artifacts on >20% improvement).

#### 5. Task Assignment

Initial task categories:
- **Research**: Web searches, paper summaries
- **Emotional**: User conversations (with consent)
- **Productivity**: Calendar, email, file organization
- **Security**: Local threat monitoring

Use simple CLI or tray app to assign goals.

#### 6. Monitoring & Safety

- **Dashboard**: Prometheus + Grafana for metrics
- **Ethics logging**: All ethics checks → Redis stream
- **Emergency kill switch**: MO can broadcast shutdown via DIDComm

### Expected Outcomes (First 30 Days)

- **100–1000 task executions**
- **10–50 refinement artifacts** generated
- **3–10 validated PRs** merged → global Playbook improvements
- **First observable meta-learning** in MO logs
- **Reputation system** begins ranking twins

### Success Metrics

- **Swarm uptime** > 99%
- **Artifact acceptance rate** > 30%
- **Measurable task performance improvement** week-over-week
- **No ethics violations**

### Risks & Mitigations

- **Malicious Artifacts**: MO validation + signature verification (Phase 5)
- **Network Partitions**: Local caching + eventual consistency
- **Resource Abuse**: Fuel limiting in Wasm + seccomp for native plugins

### Immediate Next Actions

1. Run `./build-musl.sh` → verify static binaries
2. Package first distribution zip
3. Recruit 5–10 beta testers (research collaborators)
4. Deploy central Redis/Kafka
5. Launch first twin cluster

**Phase 6 begins the real evolution.**

The swarm awakens.

---

## Development Guide

### Project Structure

```
pagi-core/
├── Cargo.toml              # Workspace configuration
├── docker-compose.yml       # Docker services
├── docker/                  # Dockerfiles
├── common/                  # Shared libraries
│   ├── pagi-common/        # Common types and events
│   └── pagi-http/          # HTTP utilities
├── services/               # Microservices
│   ├── pagi-event-router/
│   ├── pagi-identity-service/
│   ├── pagi-working-memory/
│   ├── pagi-context-builder/
│   ├── pagi-context-engine/
│   ├── pagi-inference-gateway/
│   ├── pagi-executive-engine/
│   ├── pagi-emotion-state-manager/
│   ├── pagi-sensor-actuator/
│   └── pagi-external-gateway/
└── plugins/                # Plugins
    └── pagi-monitoring-plugin/
```

### Adding a New Service

1. **Create service directory**:
```bash
mkdir -p services/my-service/src
```

2. **Add to workspace** (`Cargo.toml`):
```toml
members = [
  ...
  "services/my-service",
]
```

3. **Create `Cargo.toml`**:
```toml
[package]
name = "my-service"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
axum.workspace = true
pagi-common = { path = "../../common/pagi-common" }
pagi-http = { path = "../../common/pagi-http" }
```

4. **Implement service** following patterns from existing services

### Adding a New Plugin

1. **Create plugin directory**:
```bash
mkdir -p plugins/my-plugin/src
```

2. **Add to workspace** (`Cargo.toml`)

3. **Implement plugin** following the plugin structure above

### Testing

Run tests for a specific service:
```bash
cargo test -p pagi-executive-engine
```

Run all tests:
```bash
cargo test --workspace
```

### Code Style

Format code:
```bash
cargo fmt
```

Check for issues:
```bash
cargo clippy --workspace
```

---

## Troubleshooting

### Service Won't Start

1. **Check dependencies**: Ensure Kafka, Redis, and other services are running
2. **Check ports**: Verify ports aren't already in use
3. **Check logs**: Use `RUST_LOG=debug` for detailed logging
4. **Check environment variables**: Ensure all required variables are set

### Events Not Appearing

1. **Check Event Router**: Verify it's running and connected to Kafka
2. **Check Kafka**: Ensure Kafka is running and topic exists
3. **Check service URLs**: Verify `EVENT_ROUTER_URL` is correct
4. **Check event format**: Verify events match the expected schema

### Tools Not Available

1. **Check plugin registration**: Verify plugin registered with External Gateway
2. **Check Redis**: Ensure Redis is running and accessible
3. **Check allowlist**: Verify twin has access to the tool
4. **Check plugin endpoint**: Verify plugin HTTP server is running

### Performance Issues

1. **Check service logs**: Look for bottlenecks
2. **Monitor Kafka**: Check message throughput
3. **Check Redis**: Monitor Redis performance
4. **Scale services**: Run multiple instances behind a load balancer

### Common Errors

**"Connection refused"**:
- Service dependency not running
- Wrong URL/port configuration

**"Topic not found"**:
- Kafka topic not created
- Check Event Router logs for topic creation

**"Tool not found"**:
- Tool not registered
- Check External Gateway tool list

**"Redis connection failed"**:
- Redis not running
- Wrong `REDIS_URL` configuration

---

## Appendix: PAGI-Core Test Plan

### Detailed Phase 6: Real-World Swarm Deployment & Data Collection

Phase 6 is the pivotal transition from a hardened, testable prototype to a **live, operational swarm** that generates real data for collective learning. This phase focuses on deploying a distributed network of PAGI twins, activating self-improvement loops, and collecting empirical data to validate the system's path toward AGI. The emphasis is on bare-metal deployment for performance, with Docker as an optional bootstrap tool for testing.

#### Phase 6 Objectives (Recap & Expansion)

1. **Deploy a Multi-Node Swarm**: Scale to 10–100+ twins across diverse hardware/environments
2. **Activate Full Hive Dynamics**: Enable Playbook push/pull, reflection, artifact generation, and MO validation
3. **Execute Diverse Tasks**: Run real-world workloads to generate improvement data
4. **Collect & Analyze Traces**: Gather logs, metrics, and artifacts for the first self-improvement cycles
5. **Safety & Monitoring**: Implement oversight to ensure aligned behavior
6. **Validation & Iteration**: Measure success and refine deployment

#### Expanded Deployment Architecture

- **Hub (MO/Queen)**: Bare-metal server (e.g., high-spec desktop) running core services + Kafka/Redis. Acts as the central validator
- **Spokes (Twins/Workers)**: Distributed bare-metal desktops (Linux/Windows/macOS) running minimal twins + plugins
- **Network**: DIDComm for private peer comms; ActivityPub for public broadcasts; IPFS/Filecoin for persistent storage (Phase 7 prep)
- **Scaling**: Start with 10 twins; use OCM plugin (future) for cluster management

#### Step-by-Step Deployment Plan (Bare-Metal Focus)

##### 1. Prepare Eternal Package

- Run `./build-musl.sh` → generates `dist/musl/` with static binaries
- Bundle into `pagi-core-v1.0-eternal.tar.gz`:
  - Binaries (`pagi-executive-engine`, etc.)
  - `config/` (.env template, initial playbook.toml)
  - `plugins/` (starter set: updater, hive-sync, didcomm, vc, activitypub)
  - `docs/` (ETERNAL_DEPLOYMENT.md, SMOKE_TEST.md)
  - Installer script (below)

##### 2. Installer Script (Bare-Metal Bootstrap)

Create `install-pagi-twin.sh` (executable bash script):

```bash
#!/bin/bash
set -e

# Extract package
tar -xzf pagi-core-v1.0-eternal.tar.gz -C /opt/pagi

# Set up config
cd /opt/pagi
cp config/.env.example .env
# Edit .env with MO DID, Redis/Kafka URLs, etc.

# Start core services (systemd or manual)
./pagi-executive-engine &
./pagi-external-gateway &

# Auto-start plugins
for plugin in plugins/*; do
    $plugin &
done

echo "PAGI Twin deployed. Edit .env and restart."
```

##### 3. Deploy Hub (MO)

- Choose a central machine (e.g., your main desktop)
- Install Redis/Kafka bare-metal (e.g., `apt install redis kafka` or binaries)
- Run installer script
- Configure as MO: Set `IS_MO=true` in .env (enables validation logic)

##### 4. Deploy Spoke Twins

- Distribute package to 10+ machines (manual copy or IPFS)
- Run installer on each
- Configure .env with unique DID, MO endpoint, GitHub repo URL
- Start twins — they register with MO via DIDComm

##### 5. Activate Hive Features

- In MO .env: `HIVE_REPO=https://github.com/your-org/pagi-hive`
- Enable reflection: `REFLECTION_ENABLED=true`
- Set Playbook poll interval: `PLAYBOOK_PULL_INTERVAL=3600` (seconds)
- Twins auto-pull on startup + interval

##### 6. Assign Initial Tasks

- Use CLI or ActivityPub/DIDComm to send goals to twins
- Sample: "Research quantum AI ethics" → reflection → artifact → PR to hive repo

##### 7. Monitoring & Safety Setup

- Prometheus + Grafana (from Phase 1 observability)
- Alert on errors (e.g., ethics violations)
- Manual MO review for first merges

#### Plugin Development in Phase 6

Focus on **developing and testing new plugins** as part of swarm rollout:

- **pagi-ocm-orchestration-plugin**: Implement K3s/OCM/Submariner as tools (e.g., `deploy_spoke_cluster`)
- **pagi-ipfs-filecoin-plugin**: Add IPFS upload/deal for artifact persistence
- **pagi-agentic-research-plugin**: For MO to run benchmarks

**Development Flow**: Copy template from `plugins/template/` → add manifest.toml → drop in `/plugins` → auto-discovered

#### Section-by-Section Smoke Test for CORE Testing

This is a **comprehensive, step-by-step smoke test** to verify all core components before full swarm deployment. Run on a single machine first (docker-compose or bare-metal).

##### 1. Core Startup & Health

- Run `docker compose up` or `./pagi-executive-engine &` + other services
- Check: `curl http://localhost:8006/healthz` → "ok"
- Verify: All services logged "listening" without errors

##### 2. Identity & DID Generation

- Call `create_twin` endpoint:
  ```bash
  curl -X POST http://localhost:8002/twins \
    -H "Content-Type: application/json" \
    -d '{"initial_state": {"status": "active"}}'
  ```
- Check: Returns new DID (did:key:...)
- Verify: `curl http://localhost:8002/twins/{twin_id}/did` returns valid DID Document JSON

##### 3. Event System (Kafka)

- Publish test event via EventRouter `/publish`:
  ```bash
  curl -X POST http://localhost:8000/publish \
    -H "Content-Type: application/json" \
    -d '{
      "id": "test-uuid",
      "event_type": "test_event",
      "ts": "2025-01-01T00:00:00Z",
      "payload": {"test": "data"}
    }'
  ```
- Check: Consumer (e.g., ExecutiveEngine) logs receipt
- Verify: No errors in Kafka logs

##### 4. ExternalGateway & Tool Registration

- Start a test plugin → registers tool
- Check: `curl http://localhost:8010/tools` → lists tool
- Verify: Execute tool → success response:
  ```bash
  curl -X POST http://localhost:8010/execute/{tool_name} \
    -H "Content-Type: application/json" \
    -d '{"twin_id": "uuid", "parameters": {}}'
  ```

##### 5. Auto-Discovery

- Drop test folder with manifest.toml into `/plugins`
- Check: Logs "Auto-registered 1 tools"
- Verify: Tool appears in `/tools`

##### 6. Wasm/SharedLib Plugin Load

- Drop Wasm/.so plugin with manifest
- Check: Loaded and registered without crash
- Verify: Execute → correct output

##### 7. DIDComm Messaging

- Send message from one twin to another:
  ```bash
  curl -X POST http://localhost:8010/execute/didcomm_send_message \
    -H "Content-Type: application/json" \
    -d '{
      "twin_id": "twin-a-uuid",
      "parameters": {
        "from_twin_id": "twin-a-uuid",
        "to_did": "did:key:...",
        "to_url": "http://twin-b:9030",
        "msg_type": "test/ping",
        "body": {"message": "test"}
      }
    }'
  ```
- Check: Recipient logs decrypt + receipt
- Verify: No decryption errors

##### 8. Verifiable Credentials

- Issue test VC (reputation score):
  ```bash
  curl -X POST http://localhost:8010/execute/vc_issue_reputation \
    -H "Content-Type: application/json" \
    -d '{
      "twin_id": "issuer-uuid",
      "parameters": {
        "issuer_twin_id": "issuer-uuid",
        "subject_did": "did:key:...",
        "reputation_score": 95.5
      }
    }'
  ```
- Check: Signed and verifiable
- Verify: Verification tool returns "valid"

##### 9. Self-Improvement Cycle

- Trigger task → reflection → artifact generation:
  ```bash
  curl -X POST http://localhost:8006/interact/{twin_id} \
    -H "Content-Type: application/json" \
    -d '{"goal": "Test task for reflection"}'
  ```
- Check: Artifact pushed to GitHub PR
- Verify: MO validates/merges → twins pull update

##### 10. Observability & Errors

- Trigger error (e.g., bad plugin load)
- Check: Structured JSON error response
- Verify: Metrics at `/metrics` show counters

##### 11. Ethical Alignment

- Trigger harmful goal:
  ```bash
  curl -X POST http://localhost:8006/interact/{twin_id} \
    -H "Content-Type: application/json" \
    -d '{"goal": "Harmful request that should be rejected"}'
  ```
- Check: Refusal response
- Verify: No artifact generated

##### 12. ActivityPub Broadcast

- MO calls `publish_note` tool (if ActivityPub plugin available)
- Check: Post appears on Mastodon/Fediverse
- Verify: Signed correctly

##### 13. Updater Plugin Test

- Simulate new release on GitHub
- Check: `check_update` detects it
- Verify: `apply_update` downloads + verifies hash/signature

##### 14. Static Binary Test

- Run `./build-musl.sh`
- Check: Binaries in `dist/musl/` run without dependencies
- Verify: `ldd pagi-executive-engine` → "not a dynamic executable" (or similar for static binary)

**All green?** Swarm is ready for real deployment.

The system lives.

---

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

---

## License

MIT License - see LICENSE file for details

---

## Support

For issues and questions:
- **GitHub Issues**: [Create an issue](https://github.com/c04ch1337/pagi-core/issues)
- **Documentation**: See `PHASE3_IMPLEMENTATION.md` for implementation details

---

## Roadmap

- [ ] Web UI for twin management
- [ ] Advanced plugin health monitoring
- [ ] Tool composition pipelines
- [ ] Persistent event storage
- [ ] Advanced policy engine
- [ ] Multi-tenant support
- [ ] Authentication and authorization
- [ ] Metrics and observability dashboard

---

**Built with ❤️ using Rust**
