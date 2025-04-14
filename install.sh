#!/usr/bin/env bash

# inspired from: https://github.com/chaqchase/lla/blob/main/install.sh
# this script is for Linux / macOS
#
# use with command:
#
# ```bash
# curl -fsSL https://dria.co/launcher | bash
#
# # or the direct link
# curl -fsSL https://raw.githubusercontent.com/firstbatchxyz/dkn-compute-launcher/refs/heads/master/install.ps1 | bash
# ```
#
# installs the binary under ~/.dria/bin and adds it to path if a known shell is detected.


# exit on error
set -e

################# LOGGERS #################

NC='\033[0m' 
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'

print_step() {
    echo -e "${BLUE}==>${NC} $1"
}
print_success() {
    echo -e "${GREEN}==>${NC} $1"
}
print_error() {
    echo -e "${RED}==>${NC} $1"
}

################## LOGIC ##################

# detects the platform and returns the respective asset name
# e.g. dkn-compute-launcher-linux-amd64
#
# this can be used with version to obtain the download URL like:
# https://github.com/firstbatchxyz/dkn-compute-launcher/releases/download/v0.1.0/dkn-compute-launcher-linux-amd64
get_release_name() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)     OS="linux" ;;
        Darwin)    OS="macOS" ;;
        *)
            print_error "Unsupported operating system: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64)  ARCH="amd64" ;;
        aarch64) ARCH="arm64" ;;
        arm64)   ARCH="arm64" ;;
        *)
            print_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    RELEASE_NAME="dkn-compute-launcher-${OS}-${ARCH}"
}

get_latest_version() {
    # this retuns a release object with a `tag_name` field that contains the `tag` as appears in GitHub release
    LATEST_RELEASE_URL="https://api.github.com/repos/firstbatchxyz/dkn-compute-launcher/releases/latest"
    # we cURL that and extract the `tag_name` field
    VERSION=$(curl -s $LATEST_RELEASE_URL | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        print_error "Failed to fetch latest version"
        exit 1
    fi
}

download_binary() {
    # FIXME: for some reason sometimes the `get_latest_version` sets VERSION to `:`, still not sure how
    if [ "$VERSION" = ":" ]; then
        print_step "Downloading Dria Compute Launcher (latest) for ${OS}-${ARCH}..."
        DOWNLOAD_URL="https://github.com/firstbatchxyz/dkn-compute-launcher/releases/latest/download/${RELEASE_NAME}"
    else
        print_step "Downloading Dria Compute Launcher ${VERSION} for ${OS}-${ARCH}..."
        DOWNLOAD_URL="https://github.com/firstbatchxyz/dkn-compute-launcher/releases/download/${VERSION}/${RELEASE_NAME}"
    fi
    
    
    print_step "Downloading from $DOWNLOAD_URL"
    TMP_DIR=$(mktemp -d)
    curl -f -L "$DOWNLOAD_URL" -o "${TMP_DIR}/dkn-compute-launcher"
    
    if [ $? -ne 0 ]; then
        print_error "Failed to download launcher"
        rm -rf "$TMP_DIR"
        exit 1
    fi

    print_success "Downloaded launcher to ${TMP_DIR}"
}

# move launcher binary to $HOME/.dria/bin for global access
install_binary() {
    DRIA_INSTALL_DIR="$HOME/.dria/bin"
    
    # extract to target path, and make it executable
    print_step "Extracting binary to ${DRIA_INSTALL_DIR}"
    mkdir -p "$DRIA_INSTALL_DIR"
    mv "${TMP_DIR}/dkn-compute-launcher" "${DRIA_INSTALL_DIR}/"
    rm -rf "$TMP_DIR"
    chmod +x "${DRIA_INSTALL_DIR}/dkn-compute-launcher"

    # detect current shell
    SHELL_NAME=$(basename "$SHELL")
    print_step "Detected shell: $SHELL_NAME"

    # check shell and corresponding config files
    case "$SHELL_NAME" in
      "fish")
        CONFIG_FILE="$HOME/.config/fish/config.fish"
        ;;
      "zsh")
        CONFIG_FILE="$HOME/.zshrc"
        ;;
      "bash")
        # check for both .bash_profile and .bashrc
        if [ -f "$HOME/.bash_profile" ]; then
          CONFIG_FILE="$HOME/.bash_profile"
        else
          CONFIG_FILE="$HOME/.bashrc"
        fi
        ;;
      *)
        print_step "You are using $SHELL_NAME shell."
        print_step "Please manually add this line to your shell config file:"
        print_step "export PATH=\"${DRIA_INSTALL_DIR}:\$PATH\""
        return
        ;;
    esac

    # if config file exists, add PATH if not already present
    if [ -f "$CONFIG_FILE" ]; then
      if grep -q "export PATH=\"${DRIA_INSTALL_DIR}:\$PATH\"" "$CONFIG_FILE"; then
        print_step "Dria Compute Launcher path exists in $CONFIG_FILE"
        return
      fi
      print_step "Adding Dria Compute Launcher path to $CONFIG_FILE"
      echo "" >> "$CONFIG_FILE"
      echo '# added by Dria Compute Launcher' >> "$CONFIG_FILE"
      echo "export PATH=\"${DRIA_INSTALL_DIR}:\$PATH\"" >> "$CONFIG_FILE"
    else
      print_step "Creating config file for your shell: $CONFIG_FILE"
      touch "$CONFIG_FILE"
      echo '# added by Dria Compute Launcher' >> "$CONFIG_FILE"
      echo "export PATH=\"${DRIA_INSTALL_DIR}:\$PATH\"" >> "$CONFIG_FILE"
    fi
}

# WSL has some issues, we prefer that users run the Windows build instead
detect_wsl() {
    if [ -f "/proc/version" ] && grep -qi microsoft /proc/version; then
        print_error "WSL detected; please use Windows terminal (cmd.exe) instead and follow the steps below:"
        print_error "  (1) Install the launcher: powershell -c \"irm https://dria.co/launcher.ps1 | iex\""
        print_error "  (2) Restart your terminal"
        print_error "  (3) Start the node: dkn-compute-launcher.exe start"
        exit 1
    fi
}

main() {
    print_step "Installing Dria Compute Launcher to $(pwd)"
    if ! command -v curl >/dev/null 2>&1; then
        print_error "curl is required but not installed"
        exit 1
    fi
    
    detect_wsl
    get_release_name
    get_latest_version
    download_binary
    install_binary

    print_success "Dria Compute Launcher has been installed successfully!"
    print_success "Please RESTART your terminal, and then:"
    print_success "  \"dkn-compute-launcher help\" to see available commands,"
    print_success "  \"dkn-compute-launcher start\" to start a node!"
}

main
