#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="$PROJECT_DIR/dist"
WEB_DIR="$PROJECT_DIR/web"
TARGET="wasm32-unknown-unknown"
RELEASE_DIR="$PROJECT_DIR/target/$TARGET/release"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
print_usage() {
    echo "Cosmic Climber Build Script"
    echo ""
    echo "Usage: ./build.sh [command]"
    echo ""
    echo "Commands:"
    echo "  check      - Check prerequisites (Rust, wasm32 target)"
    echo "  wasm       - Build WASM release with TV-safe flags"
    echo "  package    - Package web/ into dist/"
    echo "  serve      - Serve dist/ on localhost:8080"
    echo "  all        - Run check + wasm + package (default)"
    echo "  clean      - Clean build artifacts"
    echo "  help       - Show this help"
    echo ""
}

cmd_check() {
    echo -e "${YELLOW}==> Checking prerequisites...${NC}"
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        echo -e "${RED}Error: Rust not found. Install from https://rustup.rs/${NC}"
        exit 1
    fi
    
    # Check wasm32 target
    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo -e "${YELLOW}Installing wasm32-unknown-unknown target...${NC}"
        rustup target add "$TARGET"
    fi
    
    echo -e "${GREEN}✓ Prerequisites OK${NC}"
}

cmd_wasm() {
    echo -e "${YELLOW}==> Building WASM (release with TV-safe flags)...${NC}"
    
    # Conservative flags for older TV Chromium
    RUSTFLAGS="-C target-cpu=mvp -C target-feature=-nontrapping-fptoint" \
        cargo build --release --target "$TARGET"
    
    echo -e "${GREEN}✓ WASM build complete${NC}"
    echo "WASM size: $(du -h "$RELEASE_DIR/cosmic-climber-rs.wasm" | cut -f1)"
}

cmd_package() {
    echo -e "${YELLOW}==> Packaging from web/ into dist/...${NC}"
    
    # Create dist directory
    mkdir -p "$DIST_DIR/assets"
    
    # Copy WASM output
    cp "$RELEASE_DIR/cosmic-climber-rs.wasm" "$DIST_DIR/"
    
    # Copy HTML from web/
    cp "$WEB_DIR"/*.html "$DIST_DIR/"
    
    # Copy optional custom JS files in web/
    cp "$WEB_DIR"/*.js "$DIST_DIR/" 2>/dev/null || true
    
    # Copy PAL modules
    if [ -d "$WEB_DIR/pal" ]; then
        cp -r "$WEB_DIR/pal" "$DIST_DIR/"
    fi
    
    # Copy all assets
    cp "$PROJECT_DIR/assets/astronaut.png" "$DIST_DIR/assets/"
    cp "$PROJECT_DIR/assets/Orbitron-Bold.ttf" "$DIST_DIR/assets/"
    
    # Download mq_js_bundle.js if missing
    if [ ! -f "$DIST_DIR/mq_js_bundle.js" ]; then
        echo -e "${YELLOW}==> Fetching mq_js_bundle.js...${NC}"
        curl -fsSL https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js \
            -o "$DIST_DIR/mq_js_bundle.js"
    fi
    
    # Optional wasm-opt optimization (if available)
    if command -v wasm-opt &> /dev/null; then
        echo -e "${YELLOW}==> Running wasm-opt...${NC}"
        wasm-opt -Oz "$DIST_DIR/cosmic-climber-rs.wasm" -o "$DIST_DIR/cosmic-climber-rs.wasm" || true
    fi
    
    echo -e "${GREEN}✓ dist/ ready for deployment${NC}"
}

cmd_serve() {
    echo -e "${YELLOW}==> Serving on http://localhost:8080${NC}"
    echo "Press Ctrl+C to stop"
    
    cd "$DIST_DIR"
    python3 -m http.server 8080 || python -m http.server 8080
}

cmd_clean() {
    echo -e "${YELLOW}==> Cleaning build artifacts...${NC}"
    cargo clean
    rm -rf "$DIST_DIR"
    echo -e "${GREEN}✓ Clean complete${NC}"
}

# Main
COMMAND="${1:-all}"

case "$COMMAND" in
    check)
        cmd_check
        ;;
    wasm)
        cmd_wasm
        ;;
    package)
        cmd_package
        ;;
    serve)
        cmd_serve
        ;;
    all)
        cmd_check
        cmd_wasm
        cmd_package
        ;;
    clean)
        cmd_clean
        ;;
    help|--help|-h)
        print_usage
        ;;
    *)
        echo -e "${RED}Unknown command: $COMMAND${NC}"
        print_usage
        exit 1
        ;;
esac
