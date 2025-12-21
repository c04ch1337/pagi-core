#!/bin/bash

# PAGI-Core Smoke Test with JSON Output
# Machine-readable output for Cursor IDE Agent automation

set -euo pipefail

# Configuration
BASE_URL="${BASE_URL:-http://localhost}"
TIMEOUT="${TIMEOUT:-10}"
JSON_OUTPUT="${JSON_OUTPUT:-smoke-test-results.json}"

# Service ports
declare -A SERVICE_PORTS=(
    ["event-router"]="8000"
    ["identity-service"]="8002"
    ["working-memory"]="8003"
    ["context-builder"]="8004"
    ["inference-gateway"]="8005"
    ["executive-engine"]="8006"
    ["emotion-state-manager"]="8007"
    ["sensor-actuator"]="8008"
    ["external-gateway"]="8010"
)

declare -A PLUGIN_PORTS=(
    ["did-plugin"]="9020"
    ["didcomm-plugin"]="9030"
    ["vc-plugin"]="9040"
    ["hive-sync-plugin"]="9050"
    ["swarm-sync-plugin"]="9010"
    ["activitypub-plugin"]="9070"
    ["ipfs-plugin"]="9080"
    ["filecoin-plugin"]="9090"
    ["ocm-orchestration-plugin"]="8095"
    ["updater-plugin"]="9060"
)

# Test results storage
declare -a TEST_RESULTS=()

# JSON helper functions
json_start() {
    echo "{"
    echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"base_url\": \"$BASE_URL\","
    echo "  \"tests\": ["
}

json_end() {
    echo "  ],"
    echo "  \"summary\": {"
    echo "    \"total\": $TOTAL_TESTS,"
    echo "    \"passed\": $PASSED_TESTS,"
    echo "    \"failed\": $FAILED_TESTS,"
    echo "    \"skipped\": $SKIPPED_TESTS"
    echo "  }"
    echo "}"
}

add_test_result() {
    local name="$1"
    local status="$2"  # pass, fail, skip
    local message="${3:-}"
    local remediation="${4:-}"
    
    local comma=""
    if [[ ${#TEST_RESULTS[@]} -gt 0 ]]; then
        comma=","
    fi
    
    echo "$comma{" >> "$JSON_OUTPUT"
    echo "      \"name\": \"$name\"," >> "$JSON_OUTPUT"
    echo "      \"status\": \"$status\"," >> "$JSON_OUTPUT"
    [[ -n "$message" ]] && echo "      \"message\": \"$message\"," >> "$JSON_OUTPUT"
    [[ -n "$remediation" ]] && echo "      \"remediation\": \"$remediation\"," >> "$JSON_OUTPUT"
    echo "      \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"" >> "$JSON_OUTPUT"
    echo "    }" >> "$JSON_OUTPUT"
    
    TEST_RESULTS+=("$name:$status")
    
    case "$status" in
        pass) ((PASSED_TESTS++)) || true ;;
        fail) ((FAILED_TESTS++)) || true ;;
        skip) ((SKIPPED_TESTS++)) || true ;;
    esac
    ((TOTAL_TESTS++)) || true
}

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# HTTP test helper
test_http_json() {
    local name="$1"
    local method="${2:-GET}"
    local url="$3"
    local expected_status="${4:-200}"
    local data="${5:-}"
    
    local response
    response=$(curl -s -w '\n%{http_code}' -X "$method" \
        ${data:+-d "$data"} \
        -H "Content-Type: application/json" \
        --max-time "$TIMEOUT" \
        "$url" 2>&1) || {
        add_test_result "$name" "fail" "Request failed or timeout" \
            "Check if service is running: curl $url"
        return 1
    }
    
    local status_code
    status_code=$(echo "$response" | tail -n1)
    local body
    body=$(echo "$response" | sed '$d')
    
    if [[ "$status_code" == "$expected_status" ]]; then
        add_test_result "$name" "pass" "HTTP $status_code"
        return 0
    else
        add_test_result "$name" "fail" "Expected HTTP $expected_status, got $status_code" \
            "Check service logs and verify endpoint is correct"
        return 1
    fi
}

# Health check
test_health_json() {
    local service="$1"
    local port="${SERVICE_PORTS[$service]:-}"
    
    if [[ -z "$port" ]]; then
        add_test_result "Health: $service" "skip" "Port not configured"
        return 0
    fi
    
    test_http_json "Health: $service" "GET" "$BASE_URL:$port/healthz" "200"
}

# Plugin health check
test_plugin_health_json() {
    local plugin="$1"
    local port="${PLUGIN_PORTS[$plugin]:-}"
    
    if [[ -z "$port" ]]; then
        add_test_result "Plugin Health: $plugin" "skip" "Port not configured"
        return 0
    fi
    
    test_http_json "Plugin Health: $plugin" "GET" "$BASE_URL:$port/healthz" "200"
}

# Main execution
main() {
    # Initialize JSON output
    > "$JSON_OUTPUT"
    json_start >> "$JSON_OUTPUT"
    
    # Test infrastructure
    log "Testing infrastructure..."
    
    # Redis
    if redis-cli -h localhost -p 6379 ping &> /dev/null; then
        add_test_result "Infrastructure: Redis" "pass" "Connection successful"
    else
        add_test_result "Infrastructure: Redis" "fail" "Connection failed" \
            "Start Redis: docker-compose up -d redis"
    fi
    
    # Kafka
    if timeout 2 bash -c "echo > /dev/tcp/localhost/29092" 2>/dev/null; then
        add_test_result "Infrastructure: Kafka" "pass" "Port 29092 is open"
    else
        add_test_result "Infrastructure: Kafka" "fail" "Port 29092 not accessible" \
            "Start Kafka: docker-compose up -d kafka zookeeper"
    fi
    
    # Test core services
    for service in "${!SERVICE_PORTS[@]}"; do
        test_health_json "$service"
    done
    
    # Test plugins
    for plugin in "${!PLUGIN_PORTS[@]}"; do
        test_plugin_health_json "$plugin"
    done
    
    # Test External Gateway tools endpoint
    test_http_json "External Gateway: List tools" "GET" \
        "$BASE_URL:${SERVICE_PORTS[external-gateway]}/tools" "200"
    
    # Finalize JSON
    json_end >> "$JSON_OUTPUT"
    
    # Print summary
    echo ""
    echo "Test Results:"
    echo "  Total:  $TOTAL_TESTS"
    echo "  Passed: $PASSED_TESTS"
    echo "  Failed: $FAILED_TESTS"
    echo "  Skipped: $SKIPPED_TESTS"
    echo ""
    echo "JSON output saved to: $JSON_OUTPUT"
    
    # Exit with appropriate code
    if [[ $FAILED_TESTS -gt 0 ]]; then
        exit 1
    else
        exit 0
    fi
}

# Helper function for logging (can be removed for pure JSON)
log() {
    echo "[$(date +%H:%M:%S)] $1" >&2
}

main "$@"

