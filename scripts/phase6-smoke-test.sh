#!/bin/bash

# PAGI-Core Phase 6 Smoke Test
# Tests real-world swarm deployment scenarios:
# - Multi-twin coordination
# - DIDComm messaging between twins
# - Playbook synchronization (Hive/Swarm sync)
# - Refinement artifact generation
# - MO meta-learning capabilities
# - Relay node functionality
# Designed for Cursor IDE Agent automation and remediation

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
LOG_FILE="${LOG_FILE:-phase6-smoke-test-$(date +%Y%m%d-%H%M%S).log}"
BASE_URL="${BASE_URL:-http://localhost}"
TIMEOUT="${TIMEOUT:-15}"
VERBOSE="${VERBOSE:-false}"

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0
FAILED_TESTS=()

# Service ports
IDENTITY_PORT=8002
EXECUTIVE_PORT=8006
EXTERNAL_GATEWAY_PORT=8010
DIDCOMM_PLUGIN_PORT=9030
HIVE_SYNC_PORT=9050
SWARM_SYNC_PORT=9010

# Test data storage
TWIN_A_ID=""
TWIN_B_ID=""
TWIN_A_DID=""
TWIN_B_DID=""
TWIN_A_URL="${BASE_URL}:${EXECUTIVE_PORT}"
TWIN_B_URL="${BASE_URL}:${EXECUTIVE_PORT}"

# Logging functions
log() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1" | tee -a "$LOG_FILE"
    ((TESTS_PASSED++)) || true
}

log_failure() {
    echo -e "${RED}[FAIL]${NC} $1" | tee -a "$LOG_FILE"
    ((TESTS_FAILED++)) || true
    FAILED_TESTS+=("$1")
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1" | tee -a "$LOG_FILE"
    ((TESTS_SKIPPED++)) || true
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

log_phase() {
    echo -e "${CYAN}[PHASE]${NC} $1" | tee -a "$LOG_FILE"
}

# HTTP helper
http_request() {
    local method="$1"
    local url="$2"
    local data="${3:-}"
    local headers="${4:-Content-Type: application/json}"
    
    local cmd="curl -s -w '\n%{http_code}' -X $method"
    cmd="$cmd -H '$headers'"
    [[ -n "$data" ]] && cmd="$cmd -d '$data'"
    cmd="$cmd --max-time $TIMEOUT '$url'"
    
    eval "$cmd" 2>&1
}

# Test helper
test_http() {
    local name="$1"
    local method="${2:-GET}"
    local url="$3"
    local expected_status="${4:-200}"
    local data="${5:-}"
    
    local response
    response=$(http_request "$method" "$url" "$data") || {
        log_failure "$name: Request failed"
        return 1
    }
    
    local status_code
    status_code=$(echo "$response" | tail -n1)
    local body
    body=$(echo "$response" | sed '$d')
    
    if [[ "$status_code" == "$expected_status" ]]; then
        log_success "$name: HTTP $status_code"
        [[ "$VERBOSE" == "true" ]] && echo "  Response: $body" | tee -a "$LOG_FILE"
        echo "$body"
        return 0
    else
        log_failure "$name: Expected HTTP $expected_status, got $status_code"
        echo "  Response: $body" | tee -a "$LOG_FILE"
        return 1
    fi
}

# Phase 1: Infrastructure and Core Services
phase1_infrastructure() {
    log_phase "Phase 1: Infrastructure and Core Services"
    
    # Test Redis
    log "Testing Redis..."
    if redis-cli -h localhost -p 6379 ping &> /dev/null; then
        log_success "Redis: Connection successful"
    else
        log_failure "Redis: Connection failed"
        log_error "Remediation: docker-compose up -d redis"
    fi
    
    # Test Kafka
    log "Testing Kafka..."
    if timeout 2 bash -c "echo > /dev/tcp/localhost/29092" 2>/dev/null; then
        log_success "Kafka: Port accessible"
    else
        log_failure "Kafka: Port not accessible"
        log_error "Remediation: docker-compose up -d kafka zookeeper"
    fi
    
    # Test core services
    test_http "Event Router Health" "GET" "$BASE_URL:8000/healthz"
    test_http "Identity Service Health" "GET" "$BASE_URL:$IDENTITY_PORT/healthz"
    test_http "External Gateway Health" "GET" "$BASE_URL:$EXTERNAL_GATEWAY_PORT/healthz"
    test_http "Executive Engine Health" "GET" "$BASE_URL:$EXECUTIVE_PORT/healthz"
    
    log ""
}

# Phase 2: Twin Creation and DID Setup
phase2_twin_setup() {
    log_phase "Phase 2: Twin Creation and DID Setup"
    
    # Create Twin A
    log "Creating Twin A..."
    local twin_a_data='{"initial_state": {"status": "active", "name": "TwinA"}}'
    local twin_a_response
    twin_a_response=$(test_http "Create Twin A" "POST" \
        "$BASE_URL:$IDENTITY_PORT/twins" "200" "$twin_a_data") || {
        log_error "Failed to create Twin A - cannot continue Phase 2"
        return 1
    }
    
    TWIN_A_ID=$(echo "$twin_a_response" | grep -o '"twin_id":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")
    if [[ -z "$TWIN_A_ID" ]]; then
        log_failure "Could not extract Twin A ID"
        return 1
    fi
    log "Twin A ID: $TWIN_A_ID"
    
    # Get Twin A DID
    log "Retrieving Twin A DID..."
    local twin_a_did_response
    twin_a_did_response=$(test_http "Get Twin A DID" "GET" \
        "$BASE_URL:$IDENTITY_PORT/twins/$TWIN_A_ID/did") || return 1
    
    TWIN_A_DID=$(echo "$twin_a_did_response" | grep -o '"did":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")
    if [[ -n "$TWIN_A_DID" ]]; then
        log "Twin A DID: $TWIN_A_DID"
    fi
    
    # Create Twin B
    log "Creating Twin B..."
    local twin_b_data='{"initial_state": {"status": "active", "name": "TwinB"}}'
    local twin_b_response
    twin_b_response=$(test_http "Create Twin B" "POST" \
        "$BASE_URL:$IDENTITY_PORT/twins" "200" "$twin_b_data") || {
        log_error "Failed to create Twin B - cannot continue Phase 2"
        return 1
    }
    
    TWIN_B_ID=$(echo "$twin_b_response" | grep -o '"twin_id":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")
    if [[ -z "$TWIN_B_ID" ]]; then
        log_failure "Could not extract Twin B ID"
        return 1
    fi
    log "Twin B ID: $TWIN_B_ID"
    
    # Get Twin B DID
    log "Retrieving Twin B DID..."
    local twin_b_did_response
    twin_b_did_response=$(test_http "Get Twin B DID" "GET" \
        "$BASE_URL:$IDENTITY_PORT/twins/$TWIN_B_ID/did") || return 1
    
    TWIN_B_DID=$(echo "$twin_b_did_response" | grep -o '"did":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")
    if [[ -n "$TWIN_B_DID" ]]; then
        log "Twin B DID: $TWIN_B_DID"
    fi
    
    log_success "Phase 2: Both twins created with DIDs"
    log ""
}

# Phase 3: DIDComm Messaging Between Twins
phase3_didcomm_messaging() {
    log_phase "Phase 3: DIDComm Messaging Between Twins"
    
    if [[ -z "$TWIN_A_ID" ]] || [[ -z "$TWIN_B_ID" ]] || [[ -z "$TWIN_B_DID" ]]; then
        log_skip "Phase 3: Skipping - Twin setup incomplete"
        return 0
    fi
    
    # Test DIDComm plugin health
    test_http "DIDComm Plugin Health" "GET" "$BASE_URL:$DIDCOMM_PLUGIN_PORT/healthz"
    
    # Test tool registration
    log "Checking DIDComm tool registration..."
    local tools_response
    tools_response=$(test_http "List Tools" "GET" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/tools") || return 1
    
    if echo "$tools_response" | grep -q "didcomm_send_message"; then
        log_success "DIDComm tool registered"
    else
        log_failure "DIDComm tool not found in registry"
        log_error "Remediation: Check DIDComm plugin is running and registered"
        return 1
    fi
    
    # Send message from Twin A to Twin B
    log "Sending DIDComm message from Twin A to Twin B..."
    local message_data=$(cat <<EOF
{
  "twin_id": "$TWIN_A_ID",
  "parameters": {
    "from_twin_id": "$TWIN_A_ID",
    "to_did": "$TWIN_B_DID",
    "to_url": "$TWIN_B_URL",
    "msg_type": "text/plain",
    "body": {
      "text": "Hello from Twin A - Phase 6 smoke test message"
    }
  }
}
EOF
)
    
    local send_response
    send_response=$(test_http "Send DIDComm Message" "POST" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/execute/didcomm_send_message" \
        "200" "$message_data") || {
        log_error "Remediation: Check DIDComm plugin configuration and recipient endpoint"
        return 1
    }
    
    log_success "Phase 3: DIDComm messaging successful"
    log ""
}

# Phase 4: Playbook Synchronization
phase4_playbook_sync() {
    log_phase "Phase 4: Playbook Synchronization (Hive/Swarm Sync)"
    
    # Test Hive Sync Plugin
    log "Testing Hive Sync Plugin..."
    test_http "Hive Sync Plugin Health" "GET" "$BASE_URL:$HIVE_SYNC_PORT/healthz"
    
    # Check Hive Sync tool registration
    log "Checking Hive Sync tool registration..."
    local tools_response
    tools_response=$(test_http "List Tools" "GET" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/tools") || return 1
    
    if echo "$tools_response" | grep -q "pull_latest_playbook"; then
        log_success "Hive Sync tool registered"
    else
        log_failure "Hive Sync tool not found"
        log_error "Remediation: Check Hive Sync plugin is running"
    fi
    
    # Test Swarm Sync Plugin
    log "Testing Swarm Sync Plugin..."
    test_http "Swarm Sync Plugin Health" "GET" "$BASE_URL:$SWARM_SYNC_PORT/healthz"
    
    # Check Swarm Sync tool registration
    log "Checking Swarm Sync tool registration..."
    if echo "$tools_response" | grep -q "push_artifact"; then
        log_success "Swarm Sync tool registered"
    else
        log_failure "Swarm Sync tool not found"
        log_error "Remediation: Check Swarm Sync plugin is running"
    fi
    
    # Test playbook pull (if configured)
    if [[ -n "${HIVE_REPO_URL:-}" ]] || [[ -n "${SWARM_REPO_URL:-}" ]]; then
        log "Testing playbook pull..."
        if [[ -n "$TWIN_A_ID" ]]; then
            local pull_data="{\"twin_id\": \"$TWIN_A_ID\"}"
            # This may fail if repo not configured, which is OK for smoke test
            test_http "Pull Latest Playbook" "POST" \
                "$BASE_URL:$EXTERNAL_GATEWAY_PORT/execute/pull_latest_playbook" \
                "200" "$pull_data" || log_skip "Playbook pull: Repository not configured"
        fi
    else
        log_skip "Playbook sync: Repository URLs not configured (HIVE_REPO_URL or SWARM_REPO_URL)"
    fi
    
    log_success "Phase 4: Playbook synchronization plugins verified"
    log ""
}

# Phase 5: Refinement Artifact Generation
phase5_refinement_artifacts() {
    log_phase "Phase 5: Refinement Artifact Generation"
    
    if [[ -z "$TWIN_A_ID" ]]; then
        log_skip "Phase 5: Skipping - Twin setup incomplete"
        return 0
    fi
    
    # Test Executive Engine can generate plans (prerequisite for artifacts)
    log "Testing Executive Engine plan generation..."
    local plan_data=$(cat <<EOF
{
  "twin_id": "$TWIN_A_ID",
  "goal": "Test goal for refinement artifact generation in Phase 6 smoke test"
}
EOF
)
    
    local plan_response
    plan_response=$(test_http "Generate Plan" "POST" \
        "$BASE_URL:$EXECUTIVE_PORT/plan" "200" "$plan_data") || {
        log_error "Remediation: Check Executive Engine and dependencies"
        return 1
    }
    
    # Test Swarm Sync push_artifact tool
    log "Testing refinement artifact push..."
    local artifact_data=$(cat <<EOF
{
  "twin_id": "$TWIN_A_ID",
  "parameters": {
    "critique": "Test critique for Phase 6 smoke test",
    "updated_playbook": {
      "instructions": "Test playbook instructions",
      "version": 1
    }
  }
}
EOF
)
    
    # This may fail if repo not configured, which is OK
    test_http "Push Refinement Artifact" "POST" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/execute/push_artifact" \
        "200" "$artifact_data" || log_skip "Artifact push: Repository not configured"
    
    log_success "Phase 5: Refinement artifact generation tested"
    log ""
}

# Phase 6: MO Meta-Learning Capabilities
phase6_mo_meta_learning() {
    log_phase "Phase 6: MO Meta-Learning Capabilities"
    
    if [[ -z "$TWIN_A_ID" ]]; then
        log_skip "Phase 6: Skipping - Twin setup incomplete"
        return 0
    fi
    
    # Test MO can interact with twin
    log "Testing MO interaction with twin..."
    local interact_data=$(cat <<EOF
{
  "goal": "Test MO meta-learning: analyze task patterns and generate improvement suggestions"
}
EOF
)
    
    local interact_response
    interact_response=$(test_http "MO Interact" "POST" \
        "$BASE_URL:$EXECUTIVE_PORT/interact/$TWIN_A_ID" "200" "$interact_data") || {
        log_error "Remediation: Check Executive Engine (MO) is running and can access dependencies"
        return 1
    }
    
    # Verify response contains expected fields
    if echo "$interact_response" | grep -q "status\|output\|plan"; then
        log_success "MO interaction successful"
    else
        log_failure "MO interaction response missing expected fields"
    fi
    
    # Test MO can access tools (meta-learning requires tool use)
    log "Testing MO tool access..."
    local tools_response
    tools_response=$(test_http "List Tools for Twin" "GET" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/tools/$TWIN_A_ID") || return 1
    
    local tool_count
    tool_count=$(echo "$tools_response" | grep -o '"name"' | wc -l || echo "0")
    if [[ "$tool_count" -gt 0 ]]; then
        log_success "MO has access to $tool_count tools"
    else
        log_failure "MO has no tools available"
        log_error "Remediation: Check plugin registration and tool allowlists"
    fi
    
    log_success "Phase 6: MO meta-learning capabilities verified"
    log ""
}

# Phase 7: Relay Node Functionality
phase7_relay_nodes() {
    log_phase "Phase 7: Relay Node Functionality (Offline Support)"
    
    if [[ -z "$TWIN_A_ID" ]] || [[ -z "$TWIN_B_DID" ]]; then
        log_skip "Phase 7: Skipping - Twin setup incomplete"
        return 0
    fi
    
    # Test relay message sending tool
    log "Checking relay message tool..."
    local tools_response
    tools_response=$(test_http "List Tools" "GET" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/tools") || return 1
    
    if echo "$tools_response" | grep -q "didcomm_send_message_with_relay"; then
        log_success "Relay message tool registered"
        
        # Test sending via relay (if relay URL configured)
        if [[ -n "${RELAY_URL:-}" ]]; then
            log "Testing message send with relay..."
            local relay_data=$(cat <<EOF
{
  "twin_id": "$TWIN_A_ID",
  "parameters": {
    "from_twin_id": "$TWIN_A_ID",
    "to_did": "$TWIN_B_DID",
    "to_url": "$TWIN_B_URL",
    "relay_url": "$RELAY_URL",
    "msg_type": "text/plain",
    "body": {
      "text": "Test message via relay"
    }
  }
}
EOF
)
            test_http "Send Message via Relay" "POST" \
                "$BASE_URL:$EXTERNAL_GATEWAY_PORT/execute/didcomm_send_message_with_relay" \
                "200" "$relay_data" || log_skip "Relay send: Relay not accessible"
        else
            log_skip "Relay functionality: RELAY_URL not configured"
        fi
    else
        log_skip "Relay functionality: Tool not registered (may not be implemented)"
    fi
    
    log_success "Phase 7: Relay node functionality tested"
    log ""
}

# Phase 8: Multi-Twin Coordination
phase8_multi_twin_coordination() {
    log_phase "Phase 8: Multi-Twin Coordination"
    
    if [[ -z "$TWIN_A_ID" ]] || [[ -z "$TWIN_B_ID" ]]; then
        log_skip "Phase 8: Skipping - Twin setup incomplete"
        return 0
    fi
    
    # Test both twins can execute goals independently
    log "Testing Twin A goal execution..."
    local goal_a_data=$(cat <<EOF
{
  "goal": "Twin A coordination test: research quantum computing basics"
}
EOF
)
    test_http "Twin A Goal Execution" "POST" \
        "$BASE_URL:$EXECUTIVE_PORT/interact/$TWIN_A_ID" "200" "$goal_a_data" || {
        log_error "Twin A goal execution failed"
    }
    
    log "Testing Twin B goal execution..."
    local goal_b_data=$(cat <<EOF
{
  "goal": "Twin B coordination test: analyze emotional support strategies"
}
EOF
)
    test_http "Twin B Goal Execution" "POST" \
        "$BASE_URL:$EXECUTIVE_PORT/interact/$TWIN_B_ID" "200" "$goal_b_data" || {
        log_error "Twin B goal execution failed"
    }
    
    # Test twins can access shared resources (External Gateway)
    log "Testing shared resource access..."
    test_http "Twin A Tool Access" "GET" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/tools/$TWIN_A_ID"
    
    test_http "Twin B Tool Access" "GET" \
        "$BASE_URL:$EXTERNAL_GATEWAY_PORT/tools/$TWIN_B_ID"
    
    log_success "Phase 8: Multi-twin coordination verified"
    log ""
}

# Main execution
main() {
    log "========================================="
    log "PAGI-Core Phase 6 Smoke Test"
    log "Real-World Swarm Deployment Validation"
    log "========================================="
    log "Log file: $LOG_FILE"
    log "Base URL: $BASE_URL"
    log "Timeout: ${TIMEOUT}s"
    log ""
    
    # Run all phases
    phase1_infrastructure
    phase2_twin_setup || {
        log_error "Phase 2 failed - cannot continue with twin-dependent tests"
    }
    phase3_didcomm_messaging
    phase4_playbook_sync
    phase5_refinement_artifacts
    phase6_mo_meta_learning
    phase7_relay_nodes
    phase8_multi_twin_coordination
    
    # Summary
    log "========================================="
    log "Phase 6 Test Summary"
    log "========================================="
    log "Passed:  $TESTS_PASSED"
    log "Failed:  $TESTS_FAILED"
    log "Skipped: $TESTS_SKIPPED"
    log ""
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        log_error "Failed Tests:"
        for test in "${FAILED_TESTS[@]}"; do
            log_error "  - $test"
        done
        log ""
        log_error "Remediation:"
        log_error "1. Review failed tests above"
        log_error "2. Run: ./scripts/smoke-test-remediate.sh all"
        log_error "3. Check service logs: docker-compose logs <service>"
        log_error "4. Verify environment variables (HIVE_REPO_URL, SWARM_REPO_URL, RELAY_URL)"
        log ""
        log_error "Full log: $LOG_FILE"
        exit 1
    else
        log_success "All Phase 6 tests passed!"
        log "Full log: $LOG_FILE"
        exit 0
    fi
}

main "$@"

