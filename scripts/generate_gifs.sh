#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

COLUMNS=80
LINES=24
FRAME_COUNT=75
FRAME_SKIP=30
FPS=15

TSLIME_BIN="${PROJECT_ROOT}/target/release/tslime"
CONVERT_SCRIPT="${SCRIPT_DIR}/convert_text_to_png.py"
MAGICK_BIN="${MAGICK:-convert}"
GIFSICLE_BIN="${GIFSICLE:-gifsicle}"
FFMPEG_BIN="${FFMPEG:-ffmpeg}"

OUTPUT_DIR="${PROJECT_ROOT}/assets/demos"

DEMO_CONFIGS=(
    "network|network|42||"
    "exploratory|exploratory|42||"
    "tendrils|tendrils|42||"
    "organic|organic|42||"
    "ocean|organic|123|--palette ocean"
    "heat|organic|123|--palette heat"
    "ascii|organic|42|--ascii"
)

mkdir -p "$OUTPUT_DIR"

log() {
    echo -e "\033[36m$*\033[0m" >&2
}

warn() {
    echo -e "\033[33mWarning: $*\033[0m" >&2
}

error() {
    echo -e "\033[31mError: $*\033[0m" >&2
    exit 1
}

check_dependencies() {
    if [ ! -f "$TSLIME_BIN" ]; then
        error "tslime binary not found at $TSLIME_BIN. Run: cargo build --release"
    fi

    if ! command -v "python3" &> /dev/null; then
        error "python3 not found"
    fi

    if ! python3 -c "import PIL" 2>/dev/null; then
        error "Pillow (PIL) not found for Python 3. Install with: pip3 install pillow"
    fi

    if ! command -v "$FFMPEG_BIN" &> /dev/null && ! command -v "$MAGICK_BIN" &> /dev/null; then
        error "Neither ffmpeg nor ImageMagick found. Install one of: brew install ffmpeg / brew install imagemagick"
    fi
}

capture_frames() {
    local name="$1"
    local preset="$2"
    local seed="$3"
    local extra_args="$4"
    local frames_dir

    frames_dir="${OUTPUT_DIR}/frames_${name}"

    log "Capturing $name demo (preset=$preset, seed=$seed, extra='$extra_args')..."
    rm -rf "$frames_dir"

    COLUMNS=$COLUMNS LINES=$LINES "$TSLIME_BIN" \
        --capture-frames \
        --plain-output \
        --frame-count $FRAME_COUNT \
        --frame-skip $FRAME_SKIP \
        --frame-dir "$frames_dir" \
        --seed "$seed" \
        --preset "$preset" \
        $extra_args \
        --verbose

    log "  Captured $FRAME_COUNT frames to $frames_dir"
}

convert_frames_to_png() {
    local name="$1"
    local frames_dir
    local pngs_dir

    frames_dir="${OUTPUT_DIR}/frames_${name}"
    pngs_dir="${frames_dir}/png"

    log "Converting frames to PNG..."

    mkdir -p "$pngs_dir"

    for frame_file in "$frames_dir"/frame_*.txt; do
        if [ -f "$frame_file" ]; then
            filename=$(basename "$frame_file" .txt)
            python3 "$CONVERT_SCRIPT" \
                "$frame_file" \
                "$pngs_dir/${filename}.png" 2>/dev/null || true

            if [ ! -f "$pngs_dir/${filename}.png" ]; then
                warn "  Failed to convert $frame_file"
            fi
        fi
    done

    png_count=$(ls -1 "$pngs_dir"/*.png 2>/dev/null | wc -l)
    log "  Converted $png_count frames to PNG"
}

create_gif_with_ffmpeg() {
    local name="$1"
    local frames_dir
    local pngs_dir
    local output_gif

    frames_dir="${OUTPUT_DIR}/frames_${name}"
    pngs_dir="${frames_dir}/png"
    output_gif="${OUTPUT_DIR}/${name}.gif"

    log "Creating GIF using ffmpeg..."

    "$FFMPEG_BIN" \
        -framerate $FPS \
        -pattern_type glob \
        -i "$pngs_dir"/*.png \
        -vf "scale=960:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" \
        -y \
        "$output_gif" 2>/dev/null

    log "  Created $output_gif ($(ls -lh "$output_gif" | awk '{print $5}'))"
}

create_gif_with_magick() {
    local name="$1"
    local frames_dir
    local pngs_dir
    local output_gif

    frames_dir="${OUTPUT_DIR}/frames_${name}"
    pngs_dir="${frames_dir}/png"
    output_gif="${OUTPUT_DIR}/${name}.gif"

    log "Creating GIF using ImageMagick..."

    magick "$pngs_dir"/frame_*.png \
        -delay $((100 / FPS)) \
        -loop 0 \
        -colors 256 \
        "$output_gif" 2>/dev/null

    log "  Created $output_gif ($(ls -lh "$output_gif" | awk '{print $5}'))"
}

generate_demo() {
    local name="$1"
    local preset="$2"
    local seed="$3"
    local extra_args="${4:-}"

    capture_frames "$name" "$preset" "$seed" "$extra_args"
    convert_frames_to_png "$name"

    if command -v "$MAGICK_BIN" &> /dev/null; then
        create_gif_with_magick "$name"
    else
        create_gif_with_ffmpeg "$name"
    fi
}

generate_all_demos() {
    log "Generating all demo GIFs..."
    log "Configuration: ${COLUMNS}x${LINES} terminal, ${FRAME_COUNT} frames @ ${FPS}fps"
    log ""

    for config in "${DEMO_CONFIGS[@]}"; do
        IFS='|' read -r name preset seed extra_args <<< "$config"
        generate_demo "$name" "$preset" "$seed" "$extra_args"
        log ""
    done

    log "All demos generated in $OUTPUT_DIR"
    ls -lh "$OUTPUT_DIR"/*.gif | awk '{print $9, $5}'
}

generate_single_demo() {
    local demo_name="$1"
    local found=false

    for config in "${DEMO_CONFIGS[@]}"; do
        IFS='|' read -r name preset seed extra_args <<< "$config"
        if [ "$name" = "$demo_name" ]; then
            generate_demo "$name" "$preset" "$seed" "$extra_args"
            found=true
            break
        fi
    done

    if [ "$found" = false ]; then
        error "Unknown demo: $demo_name. Available: $(echo "${DEMO_CONFIGS[@]}" | tr ' ' '\n' | cut -d'|' -f1 | tr '\n' ' ')"
    fi
}

cleanup() {
    log "Cleaning up frames directories..."
    for name in $(echo "${DEMO_CONFIGS[@]}" | tr ' ' '\n' | cut -d'|' -f1); do
        frames_dir="${OUTPUT_DIR}/frames_${name}"
        [ -d "$frames_dir" ] && rm -rf "$frames_dir"
    done
    log "Cleanup complete"
}

show_help() {
    cat << EOF
Usage: $0 [OPTIONS] [DEMO_NAME]

Generate demo GIFs for tslime.

Arguments:
  DEMO_NAME    Generate only this demo (network, exploratory, tendrils, organic, ocean, heat, ascii)

Options:
  --clean       Clean up frames directories after generation
  --help        Show this help message

Available demos:
EOF
    for config in "${DEMO_CONFIGS[@]}"; do
        IFS='|' read -r name preset seed extra_args <<< "$config"
        if [ -n "$extra_args" ]; then
            echo "  - $name (preset=$preset, seed=$seed, extra='$extra_args')"
        else
            echo "  - $name (preset=$preset, seed=$seed)"
        fi
    done
}

main() {
    check_dependencies

    local clean=false
    local demo_name=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                clean=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                demo_name="$1"
                shift
                ;;
        esac
    done

    if [ -n "$demo_name" ]; then
        generate_single_demo "$demo_name"
    else
        generate_all_demos
    fi

    if [ "$clean" = true ]; then
        cleanup
    fi
}

main "$@"
