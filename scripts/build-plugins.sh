#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

PLUGIN_NAME="device"
BUILD_TYPE="release"
INSTALL_PLUGINS=false
BUILD_UNIVERSAL=false
BUILD_STANDALONE_ONLY=false

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Build VST3 and CLAP plugins for Device"
    echo ""
    echo "Options:"
    echo "  -d, --debug       Build debug version instead of release"
    echo "  -i, --install     Install plugins to system plugin directories"
    echo "  -u, --universal   Build universal binary (macOS only, arm64 + x86_64)"
    echo "  -c, --clean       Clean build artifacts before building"
    echo "  -s, --standalone  Build standalone binary only"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Build release VST3 and CLAP"
    echo "  $0 --install          # Build and install plugins"
    echo "  $0 --universal        # Build universal binary (macOS)"
    echo "  $0 --debug            # Build debug version"
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--debug)
            BUILD_TYPE="debug"
            shift
            ;;
        -i|--install)
            INSTALL_PLUGINS=true
            shift
            ;;
        -u|--universal)
            BUILD_UNIVERSAL=true
            shift
            ;;
        -c|--clean)
            log_step "Cleaning build artifacts..."
            cd "$PROJECT_DIR"
            cargo clean
            shift
            ;;
        -s|--standalone)
            BUILD_STANDALONE_ONLY=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

detect_platform() {
    case "$(uname -s)" in
        Darwin*) echo "macos" ;;
        Linux*) echo "linux" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) echo "unknown" ;;
    esac
}

get_plugin_dirs() {
    local platform=$1
    case "$platform" in
        macos)
            VST3_DIR="$HOME/Library/Audio/Plug-Ins/VST3"
            CLAP_DIR="$HOME/Library/Audio/Plug-Ins/CLAP"
            ;;
        linux)
            VST3_DIR="$HOME/.vst3"
            CLAP_DIR="$HOME/.clap"
            ;;
        windows)
            VST3_DIR="/c/Program Files/Common Files/VST3"
            CLAP_DIR="/c/Program Files/Common Files/CLAP"
            ;;
    esac
}

build_plugins() {
    local platform=$1
    local build_flag=""
    if [ "$BUILD_TYPE" = "release" ]; then
        build_flag="--release"
    fi

    cd "$PROJECT_DIR"

    if [ "$BUILD_STANDALONE_ONLY" = true ]; then
        log_step "Building standalone binary ($BUILD_TYPE mode)..."
        cargo build $build_flag --bin device
        return
    fi

    log_step "Building and bundling plugins ($BUILD_TYPE mode)..."

    if [ "$BUILD_UNIVERSAL" = true ] && [ "$platform" = "macos" ]; then
        log_info "Building universal binary (arm64 + x86_64)..."
        cargo run -p xtask -- bundle-universal device $build_flag
    else
        cargo run -p xtask -- bundle device $build_flag
    fi
}

install_plugins() {
    local platform=$1
    get_plugin_dirs "$platform"

    local bundle_dir="$PROJECT_DIR/target/bundled"

    if [ ! -d "$bundle_dir" ]; then
        log_error "Bundle directory not found at $bundle_dir"
        log_error "Build plugins first with: $0"
        exit 1
    fi

    if [ -d "$bundle_dir/$PLUGIN_NAME.vst3" ]; then
        log_info "Installing VST3 to $VST3_DIR..."
        mkdir -p "$VST3_DIR"
        rm -rf "$VST3_DIR/$PLUGIN_NAME.vst3"
        cp -R "$bundle_dir/$PLUGIN_NAME.vst3" "$VST3_DIR/"
        log_info "VST3 installed: $VST3_DIR/$PLUGIN_NAME.vst3"
    fi

    if [ -d "$bundle_dir/$PLUGIN_NAME.clap" ]; then
        log_info "Installing CLAP to $CLAP_DIR..."
        mkdir -p "$CLAP_DIR"
        rm -rf "$CLAP_DIR/$PLUGIN_NAME.clap"
        cp -R "$bundle_dir/$PLUGIN_NAME.clap" "$CLAP_DIR/"
        log_info "CLAP installed: $CLAP_DIR/$PLUGIN_NAME.clap"
    fi
}

show_results() {
    local platform=$1
    local bundle_dir="$PROJECT_DIR/target/bundled"

    echo ""
    log_info "Build complete!"
    echo ""

    if [ "$BUILD_STANDALONE_ONLY" = true ]; then
        local standalone_path="$PROJECT_DIR/target/$BUILD_TYPE/device"
        if [ -f "$standalone_path" ]; then
            echo "Standalone binary:"
            ls -lh "$standalone_path"
        fi
        return
    fi

    if [ -d "$bundle_dir" ]; then
        echo "Built plugins in $bundle_dir:"
        for item in "$bundle_dir"/*; do
            if [ -e "$item" ]; then
                name=$(basename "$item")
                size=$(du -sh "$item" 2>/dev/null | cut -f1)
                echo "  $name ($size)"
            fi
        done
        echo ""
    fi

    if [ "$INSTALL_PLUGINS" = true ]; then
        get_plugin_dirs "$platform"
        echo "Installed to:"
        if [ -d "$VST3_DIR/$PLUGIN_NAME.vst3" ]; then
            echo "  $VST3_DIR/$PLUGIN_NAME.vst3"
        fi
        if [ -d "$CLAP_DIR/$PLUGIN_NAME.clap" ]; then
            echo "  $CLAP_DIR/$PLUGIN_NAME.clap"
        fi
    else
        echo "To install plugins to system directories, run:"
        echo "  $0 --install"
    fi
}

main() {
    local platform=$(detect_platform)
    log_info "Platform: $platform"
    log_info "Build type: $BUILD_TYPE"

    if [ "$platform" = "unknown" ]; then
        log_error "Unsupported platform"
        exit 1
    fi

    if [ "$BUILD_UNIVERSAL" = true ] && [ "$platform" != "macos" ]; then
        log_warn "Universal builds are only supported on macOS, ignoring --universal flag"
        BUILD_UNIVERSAL=false
    fi

    build_plugins "$platform"

    if [ "$INSTALL_PLUGINS" = true ] && [ "$BUILD_STANDALONE_ONLY" = false ]; then
        install_plugins "$platform"
    fi

    show_results "$platform"
}

main
