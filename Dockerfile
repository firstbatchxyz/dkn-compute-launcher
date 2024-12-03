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
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Download the zip file
ADD https://github.com/firstbatchxyz/dkn-compute-launcher/releases/download/v0.0.17/dkn-compute-launcher-linux-amd64.zip /tmp/dkn-compute-launcher.zip

# Extract the zip file
RUN unzip /tmp/dkn-compute-launcher.zip -d /tmp/ && \
    rm /tmp/dkn-compute-launcher.zip && \
    cd /tmp && \
    mv dkn-compute-node /opt/dkn-compute-node

# Change working directory
WORKDIR /opt/dkn-compute-node

# Add startup script
COPY start.sh /opt/dkn-compute-node/start.sh

# Make the launcher and the script executable
RUN chmod +x ./dkn-compute-launcher ./start.sh

# Set the entrypoint to the startup script
ENTRYPOINT ["./start.sh"]
