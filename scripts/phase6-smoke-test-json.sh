#!/bin/bash

# PAGI-Core Phase 6 Smoke Test with JSON Output
# Machine-readable output for Cursor IDE Agent automation

set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost}"
TIMEOUT="${TIMEOUT:-15}"
JSON_OUTPUT="${JSON_OUTPUT:-phase6-smoke-test-results.json}"

# Service ports
IDENTITY_PORT=8002
EXECUTIVE_PORT=8006
EXTERNAL_GATEWAY_PORT=8010
DIDCOMM_PLUGIN_PORT=9030
HIVE_SYNC_PORT=9050
SWARM_SYNC_PORT=9010

# Test results
declare -a TEST_RESULTS=()
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# JSON helpers
json_start() {
    echo "{" > "$JSON_OUTPUT"
    echo "  \"phase\": \"Phase 6 - Real-World Swarm Deployment\"," >> "$JSON_OUTPUT"
    echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$JSON_OUTPUT"
    echo "  \"base_url\": \"$BASE_URL\"," >> "$JSON_OUTPUT"
    echo "  \"phases\": {" >> "$JSON_OUTPUT"
}

json_end() {
    echo "  }," >> "$JSON_OUTPUT"
    echo "  \"summary\": {" >> "$JSON_OUTPUT"
    echo "    \"total\": $TOTAL_TESTS," >> "$JSON_OUTPUT"
    echo "    \"passed\": $PASSED_TESTS," >> "$JSON_OUTPUT"
    echo "    \"failed\": $FAILED_TESTS," >> "$JSON_OUTPUT"
    echo "    \"skipped\": $SKIPPED_TESTS" >> "$JSON_OUTPUT"
    echo "  }" >> "$JSON_OUTPUT"
    echo "}" >> "$JSON_OUTPUT"
}

add_test_result() {
    local phase="$1"
    local name="$2"
    local status="$3"  # pass, fail, skip
    local message="${4:-}"
    local remediation="${5:-}"
    
    local comma=""
    if [[ ${#TEST_RESULTS[@]} -gt 0 ]]; then
        comma=","
    fi
    
    echo "$comma{" >> "$JSON_OUTPUT"
    echo "        \"name\": \"$name\"," >> "$JSON_OUTPUT"
    echo "        \"status\": \"$status\"," >> "$JSON_OUTPUT"
    [[ -n "$message" ]] && echo "        \"message\": \"$message\"," >> "$JSON_OUTPUT"
    [[ -n "$remediation" ]] && echo "        \"remediation\": \"$remediation\"," >> "$JSON_OUTPUT"
    echo "        \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"" >> "$JSON_OUTPUT"
    echo "      }" >> "$JSON_OUTPUT"
    
    TEST_RESULTS+=("$phase:$name:$status")
    
    case "$status" in
        pass) ((PASSED_TESTS++)) || true ;;
        fail) ((FAILED_TESTS++)) || true ;;
        skip) ((SKIPPED_TESTS++)) || true ;;
    esac
    ((TOTAL_TESTS++)) || true
}

start_phase() {
    local phase_name="$1"
    local phase_key="$2"
    
    local comma=""
    if [[ ${#TEST_RESULTS[@]} -gt 0 ]]; then
        comma=","
    fi
    
    echo "$comma\"$phase_key\": [" >> "$JSON_OUTPUT"
}

end_phase() {
    echo "    ]" >> "$JSON_OUTPUT"
}

# HTTP test
test_http_json() {
    local phase="$1"
    local name="$2"
    local method="${3:-GET}"
    local url="$4"
    local expected_status="${5:-200}"
    local data="${6:-}"
    
    local response
    response=$(curl -s -w '\n%{http_code}' -X "$method" \
        ${data:+-d "$data"} \
        -H "Content-Type: application/json" \
        --max-time "$TIMEOUT" \
        "$url" 2>&1) || {
        add_test_result "$phase" "$name" "fail" "Request failed or timeout" \
            "Check if service is running: curl $url"
        return 1
    }
    
    local status_code
    status_code=$(echo "$response" | tail -n1)
    local body
    body=$(echo "$response" | sed '$d')
    
    if [[ "$status_code" == "$expected_status" ]]; then
        add_test_result "$phase" "$name" "pass" "HTTP $status_code"
        return 0
    else
        add_test_result "$phase" "$name" "fail" \
            "Expected HTTP $expected_status, got $status_code" \
            "Check service logs and verify endpoint"
        return 1
    fi
}

# Main execution
main() {
    > "$JSON_OUTPUT"
    json_start
    
    # Phase 1: Infrastructure
    start_phase "Phase 1" "infrastructure"
    
    if redis-cli -h localhost -p 6379 ping &> /dev/null; then
        add_test_result "Phase 1" "Redis" "pass" "Connection successful"
    else
        add_test_result "Phase 1" "Redis" "fail" "Connection failed" \
            "Start Redis: docker-compose up -d redis"
    fi
    
    if timeout 2 bash -c "echo > /dev/tcp/localhost/29092" 2>/dev/null; then
        add_test_result "Phase 1" "Kafka" "pass" "Port accessible"
    else
        add_test_result "Phase 1" "Kafka" "fail" "Port not accessible" \
            "Start Kafka: docker-compose up -d kafka zookeeper"
    fi
    
    test_http_json "Phase 1" "Event Router" "GET" "$BASE_URL:8000/healthz" "200"
    test_http_json "Phase 1" "Identity Service" "GET" "$BASE_URL:$IDENTITY_PORT/healthz" "200"
    test_http_json "Phase 1" "External Gateway" "GET" "$BASE_URL:$EXTERNAL_GATEWAY_PORT/healthz" "200"
    test_http_json "Phase 1" "Executive Engine" "GET" "$BASE_URL:$EXECUTIVE_PORT/healthz" "200"
    
    end_phase
    
    # Phase 2: Twin Setup
    start_phase "Phase 2" "twin_setup"
    
    local twin_a_data='{"initial_state": {"status": "active", "name": "TwinA"}}'
    local twin_a_response
    twin_a_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "$twin_a_data" --max-time "$TIMEOUT" \
        "$BASE_URL:$IDENTITY_PORT/twins" 2>&1) || {
        add_test_result "Phase 2" "Create Twin A" "fail" "Request failed" \
            "Check Identity Service"
        end_phase
        json_end
        exit 1
    }
    
    local twin_a_id
    twin_a_id=$(echo "$twin_a_response" | grep -o '"twin_id":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")
    if [[ -n "$twin_a_id" ]]; then
        add_test_result "Phase 2" "Create Twin A" "pass" "Twin ID: $twin_a_id"
    else
        add_test_result "Phase 2" "Create Twin A" "fail" "Could not extract twin ID"
        end_phase
        json_end
        exit 1
    }
    
    # Similar for Twin B...
    local twin_b_data='{"initial_state": {"status": "active", "name": "TwinB"}}'
    local twin_b_response
    twin_b_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "$twin_b_data" --max-time "$TIMEOUT" \
        "$BASE_URL:$IDENTITY_PORT/twins" 2>&1) || {
        add_test_result "Phase 2" "Create Twin B" "fail" "Request failed"
        end_phase
        json_end
        exit 1
    }
    
    local twin_b_id
    twin_b_id=$(echo "$twin_b_response" | grep -o '"twin_id":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")
    if [[ -n "$twin_b_id" ]]; then
        add_test_result "Phase 2" "Create Twin B" "pass" "Twin ID: $twin_b_id"
    else
        add_test_result "Phase 2" "Create Twin B" "fail" "Could not extract twin ID"
    fi
    
    end_phase
    
    # Phase 3: DIDComm
    start_phase "Phase 3" "didcomm_messaging"
    test_http_json "Phase 3" "DIDComm Plugin Health" "GET" "$BASE_URL:$DIDCOMM_PLUGIN_PORT/healthz" "200"
    end_phase
    
    # Phase 4: Playbook Sync
    start_phase "Phase 4" "playbook_sync"
    test_http_json "Phase 4" "Hive Sync Plugin" "GET" "$BASE_URL:$HIVE_SYNC_PORT/healthz" "200"
    test_http_json "Phase 4" "Swarm Sync Plugin" "GET" "$BASE_URL:$SWARM_SYNC_PORT/healthz" "200"
    end_phase
    
    # Finalize
    json_end
    
    echo ""
    echo "Phase 6 Test Results:"
    echo "  Total:  $TOTAL_TESTS"
    echo "  Passed: $PASSED_TESTS"
    echo "  Failed: $FAILED_TESTS"
    echo "  Skipped: $SKIPPED_TESTS"
    echo ""
    echo "JSON output: $JSON_OUTPUT"
    
    if [[ $FAILED_TESTS -gt 0 ]]; then
        exit 1
    else
        exit 0
    fi
}

main "$@"

