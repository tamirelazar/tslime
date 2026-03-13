#!/bin/bash
set -e

echo "Building tslime-wasm..."

# Clean previous builds
rm -rf pkg

# Build for web target with optimizations
wasm-pack build --target web --out-dir pkg

echo "Build complete!"
echo ""
echo "Files created in pkg/:"
ls -lh pkg/
echo ""
echo "To test:"
echo "  cd examples && python3 -m http.server 8000"
echo "  Open http://localhost:8000"
