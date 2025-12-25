#!/usr/bin/env python3
"""ANSI text to PNG converter for tslime frames with color and cursor positioning support."""

import sys
import os
import re
from PIL import Image, ImageDraw, ImageFont


# ANSI 256 color to RGB mapping
ANSI_256_COLORS = [
    # 0-15: Standard colors
    (0, 0, 0),
    (128, 0, 0),
    (0, 128, 0),
    (128, 128, 0),
    (0, 0, 128),
    (128, 0, 128),
    (0, 128, 128),
    (192, 192, 192),
    (128, 128, 128),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (0, 0, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
]

# 16-231: 6x6x6 color cube
for r in range(0, 6):
    for g in range(0, 6):
        for b in range(0, 6):
            ANSI_256_COLORS.append(
                (
                    int(r * 255 / 5),
                    int(g * 255 / 5),
                    int(b * 255 / 5),
                )
            )

# 232-255: Grayscale ramp
for i in range(0, 24):
    v = int(i * 10 + 8)
    ANSI_256_COLORS.append((v, v, v))


def ansi_to_rgb(color_code, is_fg=True):
    """Convert ANSI 256 color code to RGB tuple."""
    if color_code >= 0 and color_code < len(ANSI_256_COLORS):
        return ANSI_256_COLORS[color_code]
    return (128, 128, 128) if is_fg else (0, 0, 0)


def parse_ansi_text_with_position(text):
    """Parse ANSI-colored text with cursor positioning into a grid of cells.

    Returns a dict mapping (row, col) to (char, fg_color, bg_color) tuples.
    Row and col are 1-indexed from cursor positioning, converted to 0-indexed for the grid.
    """
    # ANSI escape sequence patterns
    cursor_pattern = r"\x1b\[(\d+);(\d+)H"
    color_pattern = r"\x1b\[([0-9;]*)m"

    # Track current position and colors
    current_row = 0
    current_col = 0
    fg_color = None
    bg_color = None

    # Build grid of cells
    cells = {}

    # Find all escape sequences and their positions
    escapes = []
    for match in re.finditer(cursor_pattern + "|" + color_pattern, text):
        escapes.append((match.start(), match.end(), match.group(0)))

    # Parse text between escape sequences
    last_pos = 0
    for start, end, escape in escapes:
        # Add plain text before this escape sequence
        plain_text = text[last_pos:start]
        for char in plain_text:
            if char not in ("\n", "\r"):
                # Only store printable characters
                if current_row >= 0 and current_col >= 0:
                    cells[(current_row, current_col)] = (char, fg_color, bg_color)
                current_col += 1

        # Process the escape sequence
        if "H" in escape:
            # Cursor positioning: \x1b[row;colH
            match = re.match(r"\x1b\[(\d+);(\d+)H", escape)
            if match:
                current_row = int(match.group(1)) - 1  # Convert to 0-indexed
                current_col = int(match.group(2)) - 1  # Convert to 0-indexed
        elif "m" in escape:
            # Color sequence: \x1b[...m
            match = re.match(r"\x1b\[([0-9;]*)m", escape)
            if match:
                codes_str = match.group(1)
                if codes_str:
                    codes = [int(c) for c in codes_str.split(";") if c]
                else:
                    codes = []

                for code in codes:
                    if code == 0:  # Reset
                        fg_color = None
                        bg_color = None
                    elif code == 39:  # Default foreground
                        fg_color = None
                    elif code == 49:  # Default background
                        bg_color = None
                    elif 38 in codes:  # Foreground color (256-color)
                        idx = codes.index(38)
                        if idx + 2 < len(codes) and codes[idx + 1] == 5:
                            fg_color = codes[idx + 2]
                            break
                    elif 48 in codes:  # Background color (256-color)
                        idx = codes.index(48)
                        if idx + 2 < len(codes) and codes[idx + 1] == 5:
                            bg_color = codes[idx + 2]
                            break

        last_pos = end

    # Add remaining text after last escape sequence
    plain_text = text[last_pos:]
    for char in plain_text:
        if char not in ("\n", "\r"):
            if current_row >= 0 and current_col >= 0:
                cells[(current_row, current_col)] = (char, fg_color, bg_color)
            current_col += 1

    return cells


def convert_frame_to_png(input_file, output_file, font_size=14, width=80, height=24):
    """Convert an ANSI text file to PNG image with colors and cursor positioning."""
    with open(input_file, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

    cell_width = font_size * 0.6
    cell_height = int(font_size * 1.0)
    img_width = int(width * cell_width)
    img_height = int(height * cell_height)

    img = Image.new("RGB", (img_width, img_height), color=(0, 0, 0))
    draw = ImageDraw.Draw(img)

    try:
        font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", font_size)
    except:
        try:
            font = ImageFont.truetype("/System/Library/Fonts/Monaco.dfont", font_size)
        except:
            font = ImageFont.load_default()

    # Parse cells with their positions
    cells = parse_ansi_text_with_position(content)

    # Render cells at their proper positions
    for (row, col), (char, fg_code, bg_code) in cells.items():
        if row >= height or col >= width:
            continue

        fg_rgb = ansi_to_rgb(fg_code, is_fg=True) if fg_code else (128, 128, 128)
        bg_rgb = ansi_to_rgb(bg_code, is_fg=False) if bg_code else None

        cell_x = int(col * cell_width)
        cell_y = int(row * cell_height)

        # Draw background if set
        if bg_rgb:
            draw.rectangle(
                [cell_x, cell_y, cell_x + int(cell_width), cell_y + cell_height],
                fill=bg_rgb,
            )

        # Draw character
        draw.text((cell_x, cell_y), char, font=font, fill=fg_rgb)

    img.save(output_file, "PNG")
    print(f"Converted {input_file} -> {output_file}")


def main():
    if len(sys.argv) < 3:
        print(
            "Usage: python3 convert_text_to_png.py <input_file> <output_file>",
            file=sys.stderr,
        )
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    if not os.path.exists(input_file):
        print(f"Error: Input file '{input_file}' not found", file=sys.stderr)
        sys.exit(1)

    convert_frame_to_png(input_file, output_file)


if __name__ == "__main__":
    main()
