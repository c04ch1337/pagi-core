# PAGI-Core Smoke Test Scripts

Comprehensive smoke test suite for PAGI-Core services and plugins, designed for automation and Cursor IDE Agent integration.

## Scripts Overview

### Core Smoke Tests

1. **`smoke-test.sh`** - Main smoke test for all services and plugins
2. **`smoke-test-json.sh`** - JSON output version for automation
3. **`smoke-test-remediate.sh`** - Auto-remediation for common issues

### Phase 6 Smoke Tests (Swarm Deployment)

4. **`phase6-smoke-test.sh`** - Phase 6 specific tests for real-world swarm deployment
5. **`phase6-smoke-test-json.sh`** - JSON output version for Phase 6
6. **`phase6-remediate.sh`** - Auto-remediation for Phase 6 issues

## Scripts Overview

### 1. `smoke-test.sh` - Main Smoke Test Script

Comprehensive test suite that validates all core services and plugin integrations.

**Features:**
- Tests all core services (health checks, functionality)
- Tests all plugin integrations (registration, tool execution)
- Structured logging with pass/fail indicators
- Remediation hints for failed tests
- Color-coded output for easy reading
- Detailed error messages with suggested fixes

**Usage:**
```bash
# Basic usage
./scripts/smoke-test.sh

# With custom base URL
BASE_URL=http://localhost:8080 ./scripts/smoke-test.sh

# With verbose output
VERBOSE=true ./scripts/smoke-test.sh

# With custom timeout
TIMEOUT=30 ./scripts/smoke-test.sh
```

**Environment Variables:**
- `BASE_URL` - Base URL for services (default: `http://localhost`)
- `TIMEOUT` - HTTP request timeout in seconds (default: `10`)
- `VERBOSE` - Enable verbose output (default: `false`)
- `LOG_FILE` - Custom log file path (default: `smoke-test-YYYYMMDD-HHMMSS.log`)

**Output:**
- Console output with color-coded results
- Detailed log file with all test results
- Exit code: `0` if all tests pass, `1` if any test fails

### 2. `smoke-test-json.sh` - JSON Output Version

Machine-readable version of the smoke test with JSON output for programmatic processing.

**Features:**
- JSON output for easy parsing
- Same test coverage as main script
- Structured results with timestamps
- Remediation hints in JSON format

**Usage:**
```bash
# Basic usage
./scripts/smoke-test-json.sh

# Custom output file
JSON_OUTPUT=results.json ./scripts/smoke-test-json.sh
```

**Output Format:**
```json
{
  "timestamp": "2025-12-21T12:00:00Z",
  "base_url": "http://localhost",
  "tests": [
    {
      "name": "Health: event-router",
      "status": "pass",
      "message": "HTTP 200",
      "timestamp": "2025-12-21T12:00:01Z"
    },
    {
      "name": "Health: identity-service",
      "status": "fail",
      "message": "Expected HTTP 200, got 503",
      "remediation": "Check service logs and verify endpoint is correct",
      "timestamp": "2025-12-21T12:00:02Z"
    }
  ],
  "summary": {
    "total": 25,
    "passed": 23,
    "failed": 2,
    "skipped": 0
  }
}
```

### 3. `smoke-test-remediate.sh` - Auto-Remediation Script

Automatically fixes common issues detected by the smoke test.

**Features:**
- Starts infrastructure services (Redis, Kafka, Zookeeper)
- Starts core services
- Starts plugins
- Restarts failed services
- Views service logs
- Health checks with automatic retry

**Usage:**
```bash
# Remediate everything
./scripts/smoke-test-remediate.sh all

# Remediate infrastructure only
./scripts/smoke-test-remediate.sh infrastructure

# Remediate core services only
./scripts/smoke-test-remediate.sh core

# Remediate plugins only
./scripts/smoke-test-remediate.sh plugins

# View logs for a service
./scripts/smoke-test-remediate.sh logs pagi-event-router

# Restart a specific service
./scripts/smoke-test-remediate.sh restart pagi-external-gateway
```

## Test Coverage

### Infrastructure Tests
- ✅ Redis connectivity
- ✅ Kafka availability (port check)

### Core Service Tests
- ✅ Event Router (port 8000)
- ✅ Identity Service (port 8002)
- ✅ Working Memory (port 8003)
- ✅ Context Builder (port 8004)
- ✅ Inference Gateway (port 8005)
- ✅ Executive Engine (port 8006)
- ✅ Emotion State Manager (port 8007)
- ✅ Sensor Actuator (port 8008)
- ✅ External Gateway (port 8010)

### Core Service Functionality Tests
- ✅ Create twin (Identity Service)
- ✅ Get DID (Identity Service)
- ✅ Store memory fragment (Working Memory)
- ✅ Retrieve memory (Working Memory)
- ✅ Build context (Context Builder)
- ✅ Get emotion state (Emotion State Manager)
- ✅ Generate plan (Executive Engine)

### Plugin Tests
- ✅ DID Plugin (port 9020)
- ✅ DIDComm Plugin (port 9030)
- ✅ VC Plugin (port 9040)
- ✅ Hive Sync Plugin (port 9050)
- ✅ Swarm Sync Plugin (port 9010)
- ✅ ActivityPub Plugin (port 9070)
- ✅ IPFS Plugin (port 9080)
- ✅ Filecoin Plugin (port 9090)
- ✅ OCM Orchestration Plugin (port 8095)
- ✅ Updater Plugin (port 9060)

### Plugin Integration Tests
- ✅ Tool registration verification
- ✅ Tool execution (where applicable)

## Cursor IDE Agent Integration

These scripts are designed for Cursor IDE Agent automation:

### Running Tests
```bash
# Agent can run the smoke test
./scripts/smoke-test.sh

# Parse JSON results
./scripts/smoke-test-json.sh | jq '.summary'
```

### Auto-Remediation
```bash
# Agent can automatically fix issues
./scripts/smoke-test-remediate.sh all

# Then re-run tests
./scripts/smoke-test.sh
```

### Example Agent Workflow
1. Run smoke test: `./scripts/smoke-test.sh`
2. If failures detected, run remediation: `./scripts/smoke-test-remediate.sh all`
3. Re-run smoke test to verify fixes
4. Parse JSON output for detailed analysis

## Troubleshooting

### Services Not Starting
```bash
# Check Docker Compose status
docker-compose ps

# View logs
docker-compose logs <service-name>

# Restart specific service
./scripts/smoke-test-remediate.sh restart <service-name>
```

### Port Conflicts
If ports are already in use:
1. Check what's using the port: `lsof -i :8000`
2. Stop conflicting services
3. Or modify `docker-compose.yml` to use different ports

### Network Issues
If services can't reach each other:
1. Verify Docker network: `docker network ls`
2. Check service dependencies in `docker-compose.yml`
3. Ensure all required services are running

## Continuous Integration

These scripts can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run Smoke Tests
  run: |
    docker-compose up -d
    sleep 30  # Wait for services to start
    ./scripts/smoke-test.sh
    
- name: Generate JSON Report
  run: |
    ./scripts/smoke-test-json.sh
    cat smoke-test-results.json
```

## Phase 6 Smoke Test (Swarm Deployment)

The Phase 6 smoke test validates real-world swarm deployment scenarios specific to Phase 6 requirements.

### Phase 6 Test Coverage

**Phase 1: Infrastructure and Core Services**
- Redis and Kafka connectivity
- Core service health checks

**Phase 2: Twin Creation and DID Setup**
- Create multiple twins (Twin A, Twin B)
- Retrieve DIDs for each twin
- Verify DID document structure

**Phase 3: DIDComm Messaging Between Twins**
- DIDComm plugin health
- Tool registration verification
- Send encrypted messages between twins
- Verify message delivery

**Phase 4: Playbook Synchronization**
- Hive Sync plugin functionality
- Swarm Sync plugin functionality
- Playbook pull operations
- Tool registration verification

**Phase 5: Refinement Artifact Generation**
- Executive Engine plan generation
- Refinement artifact creation
- Artifact push to repository (if configured)

**Phase 6: MO Meta-Learning Capabilities**
- MO interaction with twins
- Tool access verification
- Meta-learning workflow validation

**Phase 7: Relay Node Functionality**
- Relay message tool registration
- Offline message delivery via relay
- Relay polling functionality

**Phase 8: Multi-Twin Coordination**
- Independent twin goal execution
- Shared resource access
- Concurrent twin operations

### Usage

```bash
# Run Phase 6 smoke test
./scripts/phase6-smoke-test.sh

# With custom configuration
BASE_URL=http://localhost:8080 ./scripts/phase6-smoke-test.sh

# JSON output for automation
./scripts/phase6-smoke-test-json.sh

# Auto-remediate Phase 6 issues
./scripts/phase6-remediate.sh all

# Remediate specific components
./scripts/phase6-remediate.sh infrastructure
./scripts/phase6-remediate.sh core
./scripts/phase6-remediate.sh plugins
./scripts/phase6-remediate.sh env  # Check environment variables
```

### Phase 6 Environment Variables

Optional but recommended for full Phase 6 functionality:

- `HIVE_REPO_URL` - GitHub repository URL for Hive sync
- `SWARM_REPO_URL` - GitHub repository URL for Swarm sync
- `RELAY_URL` - Relay node URL for offline messaging

```bash
export HIVE_REPO_URL=https://github.com/your-org/pagi-hive
export SWARM_REPO_URL=https://github.com/your-org/pagi-swarm
export RELAY_URL=http://relay-node:9030
```

### Phase 6 Remediation

The Phase 6 remediation script automatically:
- Starts infrastructure (Redis, Kafka, Zookeeper)
- Starts all core services
- Starts Phase 6 specific plugins (DID, DIDComm, VC, Hive Sync, Swarm Sync, ActivityPub)
- Verifies service health
- Checks environment variable configuration
- Restarts failed services

### Cursor IDE Agent Integration

**Automated Workflow:**
```bash
# 1. Run Phase 6 smoke test
./scripts/phase6-smoke-test.sh

# 2. If failures, auto-remediate
./scripts/phase6-remediate.sh all

# 3. Re-run to verify
./scripts/phase6-smoke-test.sh

# 4. Parse JSON results
./scripts/phase6-smoke-test-json.sh | jq '.summary'
```

**JSON Output Structure:**
```json
{
  "phase": "Phase 6 - Real-World Swarm Deployment",
  "timestamp": "2025-12-21T12:00:00Z",
  "phases": {
    "infrastructure": [...],
    "twin_setup": [...],
    "didcomm_messaging": [...],
    "playbook_sync": [...],
    ...
  },
  "summary": {
    "total": 25,
    "passed": 23,
    "failed": 2,
    "skipped": 0
  }
}
```

## Contributing

When adding new services or plugins:
1. Add port mapping to `SERVICE_PORTS` or `PLUGIN_PORTS` arrays
2. Add health check test
3. Add functionality tests if applicable
4. Update this README with new test coverage

## License

Part of PAGI-Core project. See main LICENSE file.

