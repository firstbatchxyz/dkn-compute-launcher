#!/bin/bash

# Generate a random 32-byte hexadecimal
random_hex=$(openssl rand -hex 32)
DKN_WALLET_SECRET_KEY=${DKN_WALLET_SECRET_KEY:-$random_hex}
echo "Random 32-byte hexadecimal (or provided key): $DKN_WALLET_SECRET_KEY"

# Function to log error and exit
log_error_and_exit() {
  echo "Error: $1 is not set. Exiting." >&2
  exit 1
}

DKN_P2P_LISTEN_ADDR=${DKN_P2P_LISTEN_ADDR:-/ip4/0.0.0.0/tcp/4001}

# Check for required environment variables
[ -z "$DKN_MODELS" ] && log_error_and_exit "DKN_MODELS"
[ -z "$DKN_WALLET_SECRET_KEY" ] && log_error_and_exit "DKN_WALLET_SECRET_KEY"
[ -z "$RUST_LOG" ] && log_error_and_exit "RUST_LOG"

# Populate the .env file
cat <<EOL > .env
DKN_MODELS=$DKN_MODELS
DKN_P2P_LISTEN_ADDR=$DKN_P2P_LISTEN_ADDR
RUST_LOG=$RUST_LOG
DKN_ADMIN_PUBLIC_KEY=0208ef5e65a9c656a6f92fb2c770d5d5e2ecffe02a6aade19207f75110be6ae658
DKN_WALLET_SECRET_KEY=$DKN_WALLET_SECRET_KEY
OLLAMA_AUTO_PULL=true
OLLAMA_PORT=11434
OLLAMA_HOST=http://host.docker.internal
DKN_NETWORK=pro
EOL

[ -n "$DKN_COMPUTE_VERSION" ] && echo "DKN_COMPUTE_VERSION=$DKN_COMPUTE_VERSION" >> .env
[ -n "$OPENAI_API_KEY" ] && echo "OPENAI_API_KEY=$OPENAI_API_KEY" >> .env
[ -n "$JINA_API_KEY" ] && echo "JINA_API_KEY=$JINA_API_KEY" >> .env
[ -n "$SERPER_API_KEY" ] && echo "SERPER_API_KEY=$SERPER_API_KEY" >> .env
[ -n "$OPENROUTER_API_KEY" ] && echo "OPENROUTER_API_KEY=$OPENROUTER_API_KEY" >> .env
[ -n "$GEMINI_API_KEY" ] && echo "GEMINI_API_KEY=$GEMINI_API_KEY" >> .env

# Function to generate random logs
generate_logs() {
  while true; do
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    log_level=$(shuf -e INFO WARN ERROR DEBUG -n 1)
    log_message="[$timestamp] [$log_level] Random log entry $(openssl rand -hex 8)"
    echo "$log_message" >> random_logs.log
    sleep $((RANDOM % 10 + 1)) # Random delay between 1-10 seconds
  done
}

# Start the log generation in the background
generate_logs &

# Run the launcher
exec ./dkn-compute-launcher
