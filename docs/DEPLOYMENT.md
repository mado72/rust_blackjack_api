# Deployment Guide - Blackjack Multi-Player API

**Version:** 1.0  
**Last Updated:** January 15, 2026  
**API Version:** 0.1.0

## Overview

This guide provides comprehensive instructions for deploying the Blackjack Multi-Player API to production environments. The API is built with Rust using Axum web framework and can be deployed using Docker or as a standalone binary.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Configuration](#configuration)
- [Deployment Options](#deployment-options)
  - [Docker Deployment](#docker-deployment)
  - [Standalone Binary](#standalone-binary)
  - [Docker Compose](#docker-compose)
- [Reverse Proxy Setup](#reverse-proxy-setup)
- [Health Checks](#health-checks)
- [Monitoring & Logging](#monitoring--logging)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **OS**: Linux (Ubuntu 22.04+ recommended), macOS, or Windows Server
- **Memory**: Minimum 512MB RAM, 1GB+ recommended
- **Storage**: 100MB for application, additional for logs
- **Network**: Port 8080 (configurable) accessible

### Software Requirements

**For Docker Deployment:**
- Docker 24.0+
- Docker Compose 2.0+ (optional)

**For Binary Deployment:**
- Rust 1.70+ (for building)
- OpenSSL/LibSSL development libraries

## Configuration

### Environment Variables

Create a `.env` file in the deployment directory:

```bash
# Server Configuration
BLACKJACK_SERVER_HOST=0.0.0.0
BLACKJACK_SERVER_PORT=8080

# JWT Authentication
BLACKJACK_JWT_SECRET=your-secure-random-secret-here-minimum-32-characters
BLACKJACK_JWT_EXPIRATION_HOURS=24

# CORS Configuration
BLACKJACK_CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://www.yourdomain.com

# Rate Limiting
BLACKJACK_RATE_LIMIT_REQUESTS_PER_MINUTE=60

# Logging
RUST_LOG=info,blackjack_api=debug,blackjack_service=debug
```

### Generating a Secure JWT Secret

```bash
# Linux/macOS
openssl rand -base64 32

# Windows PowerShell
[Convert]::ToBase64String((1..32 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
```

### Configuration File (config.toml)

The API uses `crates/blackjack-api/config.toml` as the default configuration. Environment variables override file settings.

**Production configuration template:**

```toml
[server]
host = "0.0.0.0"
port = 8080

[jwt]
secret = "OVERRIDE_WITH_ENV_VAR"
expiration_hours = 24

[rate_limit]
requests_per_minute = 60
cleanup_interval_seconds = 300

[cors]
allowed_origins = ["https://yourdomain.com"]

[api]
version = "v1"
version_deprecation_months = 6
```

## Deployment Options

### Docker Deployment

#### 1. Build Docker Image

```bash
# Clone repository
git clone https://github.com/mado72/rust_blackjack_api.git
cd rust_blackjack_api

# Build Docker image
docker build -t blackjack-api:latest .
```

The Dockerfile uses multi-stage build for optimal image size (~100MB final image).

#### 2. Run Container

```bash
# Run with environment variables
docker run -d \
  --name blackjack-api \
  --restart unless-stopped \
  -p 8080:8080 \
  -e BLACKJACK_JWT_SECRET="your-secure-secret" \
  -e RUST_LOG="info" \
  blackjack-api:latest

# Run with .env file
docker run -d \
  --name blackjack-api \
  --restart unless-stopped \
  -p 8080:8080 \
  --env-file .env \
  blackjack-api:latest
```

#### 3. Verify Deployment

```bash
# Check container status
docker ps | grep blackjack-api

# Check logs
docker logs blackjack-api

# Test health endpoint
curl http://localhost:8080/health
```

### Standalone Binary

#### 1. Build Release Binary

```bash
# Build optimized release binary
cargo build --release --bin blackjack-api

# Binary location
ls -lh target/release/blackjack-api
```

#### 2. Deploy Binary

```bash
# Copy binary to server
scp target/release/blackjack-api user@server:/opt/blackjack/

# Copy configuration
scp crates/blackjack-api/config.toml user@server:/opt/blackjack/

# SSH to server and set permissions
ssh user@server
chmod +x /opt/blackjack/blackjack-api
```

#### 3. Create Systemd Service (Linux)

Create `/etc/systemd/system/blackjack-api.service`:

```ini
[Unit]
Description=Blackjack Multi-Player API
After=network.target

[Service]
Type=simple
User=blackjack
Group=blackjack
WorkingDirectory=/opt/blackjack
ExecStart=/opt/blackjack/blackjack-api
Restart=always
RestartSec=10

# Environment variables
Environment="BLACKJACK_JWT_SECRET=your-secure-secret"
Environment="RUST_LOG=info"

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/blackjack

[Install]
WantedBy=multi-user.target
```

Enable and start service:

```bash
# Create service user
sudo useradd -r -s /bin/false blackjack

# Set ownership
sudo chown -R blackjack:blackjack /opt/blackjack

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable blackjack-api
sudo systemctl start blackjack-api

# Check status
sudo systemctl status blackjack-api

# View logs
sudo journalctl -u blackjack-api -f
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  blackjack-api:
    build: .
    container_name: blackjack-api
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - BLACKJACK_JWT_SECRET=${BLACKJACK_JWT_SECRET}
      - BLACKJACK_SERVER_HOST=0.0.0.0
      - BLACKJACK_SERVER_PORT=8080
      - RUST_LOG=info,blackjack_api=debug
    env_file:
      - .env
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
    labels:
      - "com.blackjack.version=0.1.0"

  # Optional: Nginx reverse proxy
  nginx:
    image: nginx:alpine
    container_name: blackjack-nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - blackjack-api
```

Deploy with Docker Compose:

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## Reverse Proxy Setup

### Nginx Configuration

Create `nginx.conf`:

```nginx
upstream blackjack_api {
    server blackjack-api:8080;
    # For standalone: server 127.0.0.1:8080;
}

server {
    listen 80;
    server_name api.yourdomain.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    # SSL Configuration
    ssl_certificate /etc/nginx/ssl/fullchain.pem;
    ssl_certificate_key /etc/nginx/ssl/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    # Proxy Configuration
    location / {
        proxy_pass http://blackjack_api;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check endpoint (no authentication required)
    location /health {
        proxy_pass http://blackjack_api/health;
        access_log off;
    }
}
```

### Traefik Configuration (Docker)

Add labels to `docker-compose.yml`:

```yaml
services:
  blackjack-api:
    # ... other config ...
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.blackjack.rule=Host(`api.yourdomain.com`)"
      - "traefik.http.routers.blackjack.entrypoints=websecure"
      - "traefik.http.routers.blackjack.tls.certresolver=letsencrypt"
      - "traefik.http.services.blackjack.loadbalancer.server.port=8080"
```

## Health Checks

### Endpoints

- **Basic Health**: `GET /health`
- **Readiness Check**: `GET /health/ready`

### Kubernetes Liveness/Readiness Probes

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 30
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 2
```

### Monitoring Script

```bash
#!/bin/bash
# health-check.sh

API_URL="${API_URL:-http://localhost:8080}"
MAX_RETRIES=3

for i in $(seq 1 $MAX_RETRIES); do
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/health")
  
  if [ "$HTTP_CODE" -eq 200 ]; then
    echo "✅ API is healthy (HTTP $HTTP_CODE)"
    exit 0
  else
    echo "❌ API health check failed (HTTP $HTTP_CODE) - Attempt $i/$MAX_RETRIES"
    sleep 5
  fi
done

echo "❌ API is unhealthy after $MAX_RETRIES attempts"
exit 1
```

## Monitoring & Logging

### Structured Logging

The API uses `tracing` for structured logging. Configure log levels:

```bash
# Production: Info level for API, Debug for application
RUST_LOG=info,blackjack_api=debug,blackjack_service=debug

# Development: Trace everything
RUST_LOG=trace

# Minimal: Errors only
RUST_LOG=error
```

### Log Aggregation

#### With Docker + Fluentd

```yaml
# docker-compose.yml
services:
  blackjack-api:
    logging:
      driver: "fluentd"
      options:
        fluentd-address: localhost:24224
        tag: blackjack.api
```

#### With Systemd Journal

```bash
# View logs
journalctl -u blackjack-api -f --output=json-pretty

# Export logs
journalctl -u blackjack-api --since today > /var/log/blackjack-api.log
```

### Metrics (Future Enhancement)

The API has placeholders for Prometheus metrics. Once implemented:

```yaml
# Prometheus scrape config
scrape_configs:
  - job_name: 'blackjack-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

## Security Considerations

### HTTPS/TLS

**⚠️ CRITICAL**: Always use HTTPS in production!

1. Obtain SSL certificate (Let's Encrypt recommended)
2. Configure reverse proxy for TLS termination
3. Redirect all HTTP to HTTPS
4. Use strong TLS versions (1.2+) and ciphers

### JWT Secret Security

- **Generate strong secret**: Minimum 32 characters, cryptographically random
- **Environment variables only**: Never commit secrets to version control
- **Rotate regularly**: Change JWT secret periodically (invalidates all tokens)
- **Use secrets management**: HashiCorp Vault, AWS Secrets Manager, etc.

### Rate Limiting

The API implements rate limiting at the application level:

- Default: 60 requests/minute per player
- Configure: `BLACKJACK_RATE_LIMIT_REQUESTS_PER_MINUTE`
- Additional layer: Use Nginx/Traefik rate limiting

### CORS Configuration

Restrict CORS to your frontend domains only:

```bash
BLACKJACK_CORS_ALLOWED_ORIGINS=https://app.yourdomain.com,https://www.yourdomain.com
```

### Firewall Rules

```bash
# Allow only necessary ports
sudo ufw allow 80/tcp    # HTTP (redirects to HTTPS)
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 22/tcp    # SSH (restrict to specific IPs)
sudo ufw enable
```

### Docker Security

```bash
# Run as non-root user
docker run --user 1000:1000 ...

# Read-only root filesystem
docker run --read-only ...

# Drop capabilities
docker run --cap-drop=ALL ...
```

## Troubleshooting

### Common Issues

#### 1. "Address already in use" Error

```bash
# Find process using port 8080
sudo lsof -i :8080
# or
sudo netstat -tulpn | grep 8080

# Kill process
kill -9 <PID>
```

#### 2. JWT Authentication Fails

Check:
- JWT secret matches between API and client
- Token hasn't expired
- Clock synchronization between servers
- Authorization header format: `Bearer <token>`

```bash
# Test login endpoint
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"TestPass123!"}'
```

#### 3. Health Check Returns 503

- Check application logs: `docker logs blackjack-api`
- Verify configuration is valid
- Ensure port 8080 is accessible
- Check resource availability (memory, CPU)

#### 4. High Memory Usage

```bash
# Check container stats
docker stats blackjack-api

# Check process memory
ps aux | grep blackjack-api
```

The API uses in-memory storage. Memory scales with:
- Number of active games
- Players per game
- Card history length

### Debug Mode

Enable trace logging:

```bash
RUST_LOG=trace cargo run --bin blackjack-api
# or
docker run -e RUST_LOG=trace blackjack-api:latest
```

### Testing Deployment

```bash
#!/bin/bash
# test-deployment.sh

API_URL="${1:-http://localhost:8080}"

echo "Testing Blackjack API at $API_URL"

# Test 1: Health Check
echo -n "Health check... "
curl -sf "$API_URL/health" > /dev/null && echo "✅" || echo "❌"

# Test 2: Register User
echo -n "User registration... "
REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"TestPass123!"}')
echo $REGISTER_RESPONSE | grep -q "user_id" && echo "✅" || echo "❌"

# Test 3: Login
echo -n "User login... "
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"TestPass123!"}')
TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo $LOGIN_RESPONSE | grep -q "token" && echo "✅" || echo "❌"

# Test 4: Create Game (Authenticated)
echo -n "Create game (authenticated)... "
GAME_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/games" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"enrollment_timeout_seconds":600}')
echo $GAME_RESPONSE | grep -q "game_id" && echo "✅" || echo "❌"

echo ""
echo "Deployment test complete!"
```

## Backup & Recovery

### In-Memory State (Current)

**Important**: Current version uses in-memory storage. Game state is lost on restart.

**Mitigation strategies:**
1. Use session affinity/sticky sessions in load balancer
2. Implement periodic state snapshots to disk (future)
3. Migrate to SQLite/PostgreSQL persistence (Milestone 8+)

### Future: Database Backups

Once SQLite is implemented:

```bash
# Backup SQLite database
sqlite3 /opt/blackjack/data/games.db ".backup /opt/blackjack/backups/games-$(date +%Y%m%d).db"

# Restore from backup
sqlite3 /opt/blackjack/data/games.db < /opt/blackjack/backups/games-20260115.db
```

## Performance Tuning

### Rust Build Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### System Limits

```bash
# Increase file descriptors
ulimit -n 65536

# For systemd service, add to [Service]:
LimitNOFILE=65536
```

### Docker Resource Limits

```yaml
# docker-compose.yml
services:
  blackjack-api:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
```

## Support & Maintenance

### Updating the API

```bash
# Pull latest code
git pull origin main

# Rebuild Docker image
docker build -t blackjack-api:latest .

# Stop old container
docker stop blackjack-api
docker rm blackjack-api

# Start new container
docker run -d --name blackjack-api ... blackjack-api:latest
```

### Rolling Updates (Zero Downtime)

```bash
# Start new instance on different port
docker run -d --name blackjack-api-new -p 8081:8080 ... blackjack-api:latest

# Update load balancer to point to 8081
# Wait for health checks to pass

# Stop old instance
docker stop blackjack-api

# Rename new instance
docker rename blackjack-api-new blackjack-api
```

---

## Quick Start Checklist

- [ ] Generate secure JWT secret
- [ ] Create `.env` file with all required variables
- [ ] Build Docker image or compile release binary
- [ ] Configure reverse proxy with HTTPS
- [ ] Set up health checks
- [ ] Configure firewall rules
- [ ] Test deployment with test script
- [ ] Set up log aggregation
- [ ] Configure monitoring (optional)
- [ ] Document runbook for your team

---

**For questions or issues, please open an issue on GitHub: https://github.com/mado72/rust_blackjack_api/issues**

