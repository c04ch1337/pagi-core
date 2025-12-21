#!/bin/bash

# PAGI-Core Smoke Test Auto-Remediation Script
# Automatically fixes common issues detected by smoke-test.sh
# Designed for Cursor IDE Agent automation

set -euo pipefail

LOG_FILE="${LOG_FILE:-smoke-test-remediation-$(date +%Y%m%d-%H%M%S).log}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Check if Docker Compose is available
check_docker_compose() {
    if command -v docker-compose &> /dev/null || docker compose version &> /dev/null; then
        return 0
    else
        log_error "Docker Compose not found. Please install Docker and Docker Compose."
        return 1
    fi
}

# Start infrastructure services
remediate_infrastructure() {
    log "Remediating infrastructure services..."
    
    if ! check_docker_compose; then
        return 1
    fi
    
    # Start Redis
    log "Starting Redis..."
    if docker-compose up -d redis 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Redis started"
        sleep 2
    else
        log_error "Failed to start Redis"
        return 1
    fi
    
    # Start Zookeeper and Kafka
    log "Starting Zookeeper and Kafka..."
    if docker-compose up -d zookeeper kafka 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Zookeeper and Kafka started"
        log "Waiting for Kafka to be ready..."
        sleep 10
    else
        log_error "Failed to start Zookeeper/Kafka"
        return 1
    fi
    
    # Wait for services to be healthy
    log "Waiting for infrastructure to be healthy..."
    local max_attempts=30
    local attempt=0
    
    while [[ $attempt -lt $max_attempts ]]; do
        if redis-cli -h localhost -p 6379 ping &> /dev/null && \
           timeout 2 bash -c "echo > /dev/tcp/localhost/29092" 2>/dev/null; then
            log_success "Infrastructure is healthy"
            return 0
        fi
        ((attempt++)) || true
        sleep 1
    done
    
    log_error "Infrastructure did not become healthy within timeout"
    return 1
}

# Start core services
remediate_core_services() {
    log "Remediating core services..."
    
    if ! check_docker_compose; then
        return 1
    fi
    
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
    if docker-compose up -d "${services[@]}" 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Core services started"
        log "Waiting for services to be ready..."
        sleep 5
    else
        log_error "Failed to start core services"
        return 1
    fi
}

# Start plugins
remediate_plugins() {
    log "Remediating plugins..."
    
    if ! check_docker_compose; then
        return 1
    fi
    
    local plugins=(
        "pagi-did-plugin"
        "pagi-didcomm-plugin"
        "pagi-vc-plugin"
        "pagi-hive-sync-plugin"
        "pagi-swarm-sync-plugin"
        "pagi-activitypub-plugin"
    )
    
    log "Starting plugins..."
    if docker-compose up -d "${plugins[@]}" 2>&1 | tee -a "$LOG_FILE"; then
        log_success "Plugins started"
        log "Waiting for plugins to register..."
        sleep 5
    else
        log_error "Failed to start plugins"
        return 1
    fi
}

# Check service health
check_service_health() {
    local service="$1"
    local port="$2"
    local max_attempts=30
    local attempt=0
    
    log "Checking health of $service on port $port..."
    
    while [[ $attempt -lt $max_attempts ]]; do
        if curl -s --max-time 2 "http://localhost:$port/healthz" &> /dev/null; then
            log_success "$service is healthy"
            return 0
        fi
        ((attempt++)) || true
        sleep 1
    done
    
    log_error "$service did not become healthy within timeout"
    return 1
}

# Restart failed service
restart_service() {
    local service="$1"
    
    log "Restarting $service..."
    if docker-compose restart "$service" 2>&1 | tee -a "$LOG_FILE"; then
        log_success "$service restarted"
        sleep 3
        return 0
    else
        log_error "Failed to restart $service"
        return 1
    fi
}

# View service logs
view_logs() {
    local service="$1"
    local lines="${2:-50}"
    
    log "Recent logs for $service (last $lines lines):"
    docker-compose logs --tail="$lines" "$service" 2>&1 | tee -a "$LOG_FILE"
}

# Main remediation function
main() {
    local action="${1:-all}"
    
    log "========================================="
    log "PAGI-Core Auto-Remediation Script"
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
            remediate_plugins
            ;;
        all)
            remediate_infrastructure || exit 1
            remediate_core_services || exit 1
            remediate_plugins || exit 1
            
            log ""
            log "========================================="
            log "Verifying Services"
            log "========================================="
            
            # Check key services
            check_service_health "pagi-event-router" "8000" || restart_service "pagi-event-router"
            check_service_health "pagi-external-gateway" "8010" || restart_service "pagi-external-gateway"
            check_service_health "pagi-identity-service" "8002" || restart_service "pagi-identity-service"
            
            log ""
            log_success "Remediation complete!"
            log "Run smoke-test.sh to verify all services"
            ;;
        logs)
            local service="${2:-}"
            if [[ -z "$service" ]]; then
                log_error "Please specify a service name"
                log "Usage: $0 logs <service-name>"
                exit 1
            fi
            view_logs "$service"
            ;;
        restart)
            local service="${2:-}"
            if [[ -z "$service" ]]; then
                log_error "Please specify a service name"
                log "Usage: $0 restart <service-name>"
                exit 1
            fi
            restart_service "$service"
            ;;
        *)
            echo "Usage: $0 {all|infrastructure|core|plugins|logs <service>|restart <service>}"
            exit 1
            ;;
    esac
}

main "$@"

