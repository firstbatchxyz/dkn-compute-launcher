# Base image
FROM ubuntu:20.04

# Set environment variables to avoid interaction during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install necessary utilities
RUN apt-get update && apt-get install -y \
    wget \
    unzip \
    strace \
    lsof \
    openssl \
    curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Download and extract the launcher
ADD https://github.com/firstbatchxyz/dkn-compute-launcher/releases/download/v0.0.17/dkn-compute-launcher-linux-amd64.zip /tmp/dkn-compute-launcher.zip
RUN unzip /tmp/dkn-compute-launcher.zip -d /tmp/ && \
    rm /tmp/dkn-compute-launcher.zip && \
    mv /tmp/dkn-compute-node /opt/dkn-compute-node

# Change working directory
WORKDIR /opt/dkn-compute-node

# Add the `run_processes.sh` script
COPY run_processes.sh /opt/dkn-compute-node/run_processes.sh

# Make the script and the launcher executable
RUN chmod +x ./dkn-compute-launcher ./run_processes.sh && \
    chown -R root:root /opt/dkn-compute-node

# Add a health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:11434/health || exit 1

# Set the entrypoint to the `run_processes.sh` script
ENTRYPOINT ["./run_processes.sh"]

