#!/bin/bash

# PAGI-Core Phase 6 Auto-Remediation Script
# Automatically fixes Phase 6 swarm deployment issues
# Designed for Cursor IDE Agent automation

set -euo pipefail

LOG_FILE="${LOG_FILE:-phase6-remediation-$(date +%Y%m%d-%H%M%S).log}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[FIXED]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

log_phase() {
    echo -e "${CYAN}[PHASE]${NC} $1" | tee -a "$LOG_FILE"
}

# Check Docker Compose
check_docker_compose() {
    if command -v docker-compose &> /dev/null || docker compose version &> /dev/null; then
        return 0
    else
        log_error "Docker Compose not found"
        return 1
    fi
}

# Remediate infrastructure
remediate_infrastructure() {
    log_phase "Remediating Infrastructure"
    
    if ! check_docker_compose; then
        return 1
    fi
    
    log "Starting Redis..."
    docker-compose up -d redis 2>&1 | tee -a "$LOG_FILE" && log_success "Redis started" || {
        log_error "Failed to start Redis"
        return 1
    }
    sleep 2
    
    log "Starting Zookeeper and Kafka..."
    docker-compose up -d zookeeper kafka 2>&1 | tee -a "$LOG_FILE" && log_success "Kafka started" || {
        log_error "Failed to start Kafka"
        return 1
    }
    sleep 10
    
    # Verify
    local max_attempts=30
    local attempt=0
    while [[ $attempt -lt $max_attempts ]]; do
        if redis-cli -h localhost -p 6379 ping &> /dev/null && \
           timeout 2 bash -c "echo > /dev/tcp/localhost/29092" 2>/dev/null; then
            log_success "Infrastructure healthy"
            return 0
        fi
        ((attempt++)) || true
        sleep 1
    done
    
    log_error "Infrastructure did not become healthy"
    return 1
}

# Remediate core services
remediate_core_services() {
    log_phase "Remediating Core Services"
    
    local services=(
        "pagi-event-router"
        "pagi-identity-service"
        "pagi-working-memory"
        "pagi-context-builder"
        "pagi-inference-gateway"
        "pagi-emotion-state-manager"
        "pagi-sensor-actuator"
        "pagi-external-gateway"
        "pagi-executive-engine"
    )
    
    log "Starting core services..."
    docker-compose up -d "${services[@]}" 2>&1 | tee -a "$LOG_FILE" && {
        log_success "Core services started"
        sleep 5
    } || {
        log_error "Failed to start core services"
        return 1
    }
}

# Remediate Phase 6 specific plugins
remediate_phase6_plugins() {
    log_phase "Remediating Phase 6 Plugins"
    
    local plugins=(
        "pagi-did-plugin"
        "pagi-didcomm-plugin"
        "pagi-vc-plugin"
        "pagi-hive-sync-plugin"
        "pagi-swarm-sync-plugin"
        "pagi-activitypub-plugin"
    )
    
    log "Starting Phase 6 plugins..."
    docker-compose up -d "${plugins[@]}" 2>&1 | tee -a "$LOG_FILE" && {
        log_success "Phase 6 plugins started"
        log "Waiting for plugin registration..."
        sleep 5
    } || {
        log_error "Failed to start plugins"
        return 1
    }
}

# Verify service health
verify_service() {
    local service="$1"
    local port="$2"
    local max_attempts=30
    local attempt=0
    
    log "Verifying $service on port $port..."
    
    while [[ $attempt -lt $max_attempts ]]; do
        if curl -s --max-time 2 "http://localhost:$port/healthz" &> /dev/null; then
            log_success "$service is healthy"
            return 0
        fi
        ((attempt++)) || true
        sleep 1
    done
    
    log_error "$service did not become healthy"
    return 1
}

# Restart service
restart_service() {
    local service="$1"
    
    log "Restarting $service..."
    docker-compose restart "$service" 2>&1 | tee -a "$LOG_FILE" && {
        log_success "$service restarted"
        sleep 3
        return 0
    } || {
        log_error "Failed to restart $service"
        return 1
    }
}

# Check environment variables
check_env_vars() {
    log_phase "Checking Phase 6 Environment Variables"
    
    local missing_vars=()
    
    # Optional but recommended for Phase 6
    [[ -z "${HIVE_REPO_URL:-}" ]] && missing_vars+=("HIVE_REPO_URL (optional)")
    [[ -z "${SWARM_REPO_URL:-}" ]] && missing_vars+=("SWARM_REPO_URL (optional)")
    [[ -z "${RELAY_URL:-}" ]] && missing_vars+=("RELAY_URL (optional)")
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        log_warn "Optional environment variables not set:"
        for var in "${missing_vars[@]}"; do
            log_warn "  - $var"
        done
        log_warn "These are optional but enable full Phase 6 functionality"
    else
        log_success "All Phase 6 environment variables configured"
    fi
}

# Main remediation
main() {
    local action="${1:-all}"
    
    log "========================================="
    log "PAGI-Core Phase 6 Auto-Remediation"
    log "========================================="
    log "Log file: $LOG_FILE"
    log ""
    
    case "$action" in
        infrastructure)
            remediate_infrastructure
            ;;
        core)
            remediate_core_services
            ;;
        plugins)
            remediate_phase6_plugins
            ;;
        env)
            check_env_vars
            ;;
        all)
            remediate_infrastructure || exit 1
            remediate_core_services || exit 1
            remediate_phase6_plugins || exit 1
            check_env_vars
            
            log ""
            log_phase "Verifying Critical Services"
            
            verify_service "pagi-event-router" "8000" || restart_service "pagi-event-router"
            verify_service "pagi-identity-service" "8002" || restart_service "pagi-identity-service"
            verify_service "pagi-external-gateway" "8010" || restart_service "pagi-external-gateway"
            verify_service "pagi-executive-engine" "8006" || restart_service "pagi-executive-engine"
            verify_service "pagi-didcomm-plugin" "9030" || restart_service "pagi-didcomm-plugin"
            
            log ""
            log_success "Phase 6 remediation complete!"
            log "Run ./scripts/phase6-smoke-test.sh to verify"
            ;;
        restart)
            local service="${2:-}"
            if [[ -z "$service" ]]; then
                log_error "Please specify a service name"
                exit 1
            fi
            restart_service "$service"
            ;;
        *)
            echo "Usage: $0 {all|infrastructure|core|plugins|env|restart <service>}"
            exit 1
            ;;
    esac
}

main "$@"

