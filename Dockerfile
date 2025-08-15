# VX0 Network Daemon - Multi-stage Docker build
FROM rust:1.76-slim-bookworm AS builder

# Install additional dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy all files (simpler approach for CI reliability)
COPY . .

# Build for release
ENV CARGO_NET_RETRY=10
ENV RUSTFLAGS="-C target-cpu=generic"
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create vx0net user and group
RUN groupadd -r vx0net && useradd -r -g vx0net vx0net

# Create necessary directories
RUN mkdir -p /app/config /app/certs /app/logs /app/data \
    && chown -R vx0net:vx0net /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/vx0net /usr/local/bin/vx0net

# Copy configuration templates
COPY config/ /app/config/
COPY bootstrap-registry.json /app/

# Set ownership
RUN chown -R vx0net:vx0net /app

# Switch to non-root user
USER vx0net

# Set working directory
WORKDIR /app

# Expose ports
# BGP routing
EXPOSE 1179/tcp
# IKE/IPSec security  
EXPOSE 4500/udp
# DNS server
EXPOSE 5353/udp
# Service discovery
EXPOSE 8080/tcp
# Metrics/monitoring
EXPOSE 9090/tcp

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

# Default command
CMD ["vx0net", "start", "--foreground"]
