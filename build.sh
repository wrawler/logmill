#!/usr/bin/env bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' 

echo -e "${BLUE}=== LogMill Build Script ===${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo is not installed.${NC}"
    echo -e "To install Rust and Cargo, run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo -e "Compiling the project in highly-optimized release mode..."
cargo build --release

BINARY_NAME="logmill" 
TARGET_PATH="target/release/$BINARY_NAME"

if [ -f "$TARGET_PATH" ]; then
    echo -e "${GREEN}Build successful!${NC}"
    echo -e "The compiled binary is available locally at: ./target/release/${BINARY_NAME}\n"
else
    echo -e "${RED}Build failed or binary not found.${NC}"
    exit 1
fi

echo -e "${YELLOW}Would you like to install this system-wide to /usr/local/bin?${NC}"
echo -e "This allows you to run 'logmill' from any directory."
read -p "Install globally? (requires sudo) [y/N]: " -r
echo # Move to a new line

if [[ $REPLY =~ ^[Yy]([Ee][Ss])?$ ]]; then
    echo -e "\nRequesting sudo privileges..."

    # Copy the binary to the system path
    sudo cp "$TARGET_PATH" "/usr/local/bin/$BINARY_NAME"
    
    # Verify it was placed correctly
    if [ -f "/usr/local/bin/$BINARY_NAME" ]; then
        echo -e "${GREEN}Successfully installed to /usr/local/bin/${BINARY_NAME}${NC}"
        echo -e "You can now type 'logmill' in any terminal."
    else
        echo -e "${RED} Failed to install.${NC}"
    fi
else
    echo -e "\nSkipped global installation."
fi