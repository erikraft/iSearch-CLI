#!/usr/bin/env bash

# ==============================================================================
# iSearch CLI™ - Termux Installation Script
# Official Author: ErikrafT
# Copyright © 2026 ErikrafT
# ==============================================================================

set -euo pipefail

# --- CONFIGURATION SECTION ---
# Easily switch between GitHub Releases and a future custom download domain.
USE_CUSTOM_DOMAIN=false
DOWNLOAD_DOMAIN="https://download.erikraft.com"
GITHUB_ORG_REPO="erikraft/iSearch-CLI"

# ==============================================================================
# Colors
# ==============================================================================

# TrueColor (24-bit)
BLUE='\033[38;2;66;133;244m'
RED='\033[38;2;219;68;55m'
YELLOW='\033[38;2;244;180;0m'
GREEN='\033[38;2;15;157;88m'

PURPLE='\033[38;2;171;71;188m'
CYAN='\033[38;2;38;198;218m'
WHITE='\033[38;2;245;245;245m'
GRAY='\033[38;2;160;160;160m'
NC='\033[0m'

# Fallback para terminais sem TrueColor
if [[ "${COLORTERM:-}" != *truecolor* ]]; then
    BLUE='\033[34m'
    RED='\033[31m'
    YELLOW='\033[33m'
    GREEN='\033[32m'

    PURPLE='\033[35m'
    CYAN='\033[36m'
    WHITE='\033[97m'
    GRAY='\033[90m'
    NC='\033[0m'
fi

print_banner() {
    printf '%b\n' "${BLUE}  _ ___                  _       ${RED}___ _    ${YELLOW}___${NC}"
    printf '%b\n' "${BLUE} (_) __| ___ __ _ _ _ __| |_    ${RED}/ __| |  ${YELLOW}|_ _|${NC}"
    printf '%b\n' "${BLUE} | \\__ \\/ -_) _\` | '_/ _| ' \\  ${GREEN}| (__| |__ ${YELLOW}| |${NC}"
    printf '%b\n' "${BLUE} |_|___/\\___\\__,_|_| \\__|_||_|  ${GREEN}\\___|____|${RED}___|${NC}"
    printf '\n'

    printf '%b\n' "${PURPLE}                     iSearch CLI™${NC}"
    printf '%b\n' "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    printf '%b\n' " ${WHITE}Author${GRAY}    : ${GREEN}ErikrafT${NC}"
    printf '%b\n' " ${WHITE}Copyright${GRAY} : ${GREEN}© 2026 ErikrafT${NC}"
    printf '%b\n' " ${WHITE}Search${GRAY}    : ${BLUE}https://search.erikraft.com${NC} ${YELLOW}(Classic Website)${NC}"
    printf '%b\n' " ${WHITE}Download${GRAY}  : ${BLUE}https://download.erikraft.com${NC}"
    printf '%b\n' " ${WHITE}GitHub${GRAY}    : ${BLUE}https://github.com/erikraft/iSearch-CLI${NC}"
    printf '%b\n\n' "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

log_info() {
    printf '%b\n' "${BLUE}[INFO]${NC} $1" >&2
}

log_success() {
    printf '%b\n' "${GREEN}[SUCCESS]${NC} $1" >&2
}

log_warn() {
    printf '%b\n' "${YELLOW}[WARN]${NC} $1" >&2
}

log_error() {
    printf '%b\n' "${RED}[ERROR]${NC} $1" >&2
}

# Ensure script is running inside Termux and not as root
check_environment() {
    if [ -z "${TERMUX_VERSION+x}" ] && [ ! -d "/data/data/com.termux/files/usr/bin" ]; then
        log_warn "This script is optimized for Android Termux. Continuing installation anyway..."
    else
        log_info "Detected Termux Environment (Version: ${TERMUX_VERSION:-unknown})"
    fi

    if [ "$(id -u)" -eq 0 ]; then
        log_warn "Running as root is not recommended for Termux. Please run without root / sudo."
    fi
}

# Verify Internet Connection
verify_connection() {
    log_info "Verifying internet connection..."
    if ! curl -s --connect-timeout 5 https://www.google.com > /dev/null; then
        if ! wget -q --timeout=5 -O- https://www.google.com > /dev/null; then
            log_error "No internet connection detected. Please connect to the internet and retry."
            exit 1
        fi
    fi
    log_success "Internet connection verified."
}

# Detect system architecture
detect_arch() {
    log_info "Detecting system architecture..."
    local raw_arch
    raw_arch=$(uname -m)
    local arch=""

    case "$raw_arch" in
        aarch64|arm64)
            arch="aarch64"
            ;;
        armv7l|armv8l|arm)
            arch="arm"
            ;;
        x86_64|amd64)
            arch="x86_64"
            ;;
        *)
            log_error "Unsupported CPU architecture: $raw_arch"
            exit 1
            ;;
    esac

    log_success "Architecture detected: $arch"
    printf '%s\n' "$arch"
}

# Get latest release version and download url
get_release_info() {
    local arch=$1
    local latest_version=""
    local download_url=""
    local filename=""

    # Determine filename based on architecture
    case "$arch" in
        aarch64)
            filename="isearch-cli-termux-aarch64.tar.gz"
            ;;
        arm)
            filename="isearch-cli-termux-arm.tar.gz"
            ;;
        x86_64)
            filename="isearch-cli-termux-x64.tar.gz"
            ;;
    esac

    log_info "Fetching latest release details..."

    if [ "$USE_CUSTOM_DOMAIN" = true ]; then
        local manifest_url="${DOWNLOAD_DOMAIN}/releases/latest.json"
        log_info "Fetching manifest from: $manifest_url"
        # Extract tag name / version
        if ! latest_version=$(curl -fsSL "$manifest_url" | grep -o '"tag_name": *"[^"]*"' | head -n 1 | cut -d'"' -f4); then
            log_error "Failed to fetch latest release version from custom domain."
            exit 1
        fi
        latest_version=${latest_version#v}
        download_url="${DOWNLOAD_DOMAIN}/releases/${latest_version}/${filename}"
    else
        local api_url="https://api.github.com/repos/${GITHUB_ORG_REPO}/releases/latest"
        log_info "Querying GitHub Releases API: $api_url"

        local response
        if ! response=$(curl -fsSL "$api_url"); then
            log_error "Failed to query GitHub API. Rate limit reached or DNS error."
            exit 1
        fi

        latest_version=$(echo "$response" | grep -o '"tag_name": *"[^"]*"' | head -n 1 | cut -d'"' -f4)
        latest_version=${latest_version#v}

        # Search for matching asset download url
        download_url=$(echo "$response" | grep -o '"browser_download_url": *"[^"]*"' | grep "$filename" | head -n 1 | cut -d'"' -f4)
    fi

    if [ -z "$latest_version" ] || [ -z "$download_url" ]; then
        log_error "Could not retrieve version metadata or download URL for architecture: $arch."
        exit 1
    fi

    log_success "Latest release version: v${latest_version}"
    echo "$latest_version|$download_url"
}

install_binary() {
    local download_url=$2
    local install_dir="${PREFIX:-/data/data/com.termux/files/usr}/bin"
    local dest_path="${install_dir}/isearch"

    # Create directory if it doesn't exist
    mkdir -p "$install_dir"

    log_info "Downloading binary to temporary path..."
    local temp_bin
    temp_bin=$(mktemp)

    if ! curl -fsSL -o "$temp_bin" "$download_url"; then
        if ! wget -qO "$temp_bin" "$download_url"; then
            log_error "Failed to download binary from: $download_url"
            rm -f "$temp_bin"
            exit 1
        fi
    fi

    log_info "Installing to: $dest_path"
    mv "$temp_bin" "$dest_path"
    chmod +x "$dest_path"

    log_success "Installation of isearch executable completed."
}

verify_installation() {
    local install_dir="${PREFIX:-/data/data/com.termux/files/usr}/bin"
    local dest_path="${install_dir}/isearch"

    log_info "Verifying installation integrity..."
    if [ ! -f "$dest_path" ]; then
        log_error "Executable binary was not found at $dest_path."
        exit 1
    fi

    if [ ! -x "$dest_path" ]; then
        log_error "Binary at $dest_path is not executable."
        exit 1
    fi

    log_success "iSearch CLI™ installation verified successfully!"
}

main() {
    print_banner
    check_environment
    verify_connection
    local arch
    arch=$(detect_arch)

    local release_info
    release_info=$(get_release_info "$arch")

    local version
    local download_url
    version=$(echo "$release_info" | cut -d'|' -f1)
    download_url=$(echo "$release_info" | cut -d'|' -f2)

    install_binary "$version" "$download_url"
    verify_installation

    echo -e "\n${GREEN}🎉 Congratulations! iSearch CLI™ v${version} has been successfully installed!${NC}"
    echo -e "You can launch the application by running:"
    echo -e "  ${CYAN}isearch${NC}\n"
}

main "$@"
