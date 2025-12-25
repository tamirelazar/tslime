#!/usr/bin/env python3
"""ANSI text to PNG converter for tslime frames with color support."""

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


def parse_ansi_text(text):
    """Parse ANSI-colored text into a list of (char, fg_color, bg_color) cells."""
    # ANSI escape sequence pattern
    ansi_pattern = r"\x1b\[([0-9;]*)m"

    # Track current colors
    fg_color = None
    bg_color = None

    # Parse text into tokens
    tokens = []
    last_pos = 0

    for match in re.finditer(ansi_pattern, text):
        # Add plain text before this escape sequence
        plain_text = text[last_pos : match.start()]
        for char in plain_text:
            tokens.append((char, fg_color, bg_color))

        # Parse ANSI codes
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

        last_pos = match.end()

    # Add remaining text
    plain_text = text[last_pos:]
    for char in plain_text:
        tokens.append((char, fg_color, bg_color))

    return tokens


def convert_frame_to_png(input_file, output_file, font_size=14, line_height_factor=1.2):
    """Convert an ANSI text file to PNG image with colors."""
    with open(input_file, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

    lines = content.split("\n")

    cell_width = font_size * 0.6
    cell_height = int(font_size * 1.0)
    img_width = int(80 * cell_width)
    img_height = int(24 * cell_height)

    img = Image.new("RGB", (img_width, img_height), color=(0, 0, 0))
    draw = ImageDraw.Draw(img)

    try:
        font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", font_size)
    except:
        try:
            font = ImageFont.truetype("/System/Library/Fonts/Monaco.dfont", font_size)
        except:
            font = ImageFont.load_default()

    for y, line in enumerate(lines[:24]):
        if not line.strip():
            continue

        tokens = parse_ansi_text(line)

        x_pos = 0
        for char, fg_code, bg_code in tokens:
            if x_pos >= 80:
                break

            if char == "\n" or char == "\r":
                continue

            fg_rgb = ansi_to_rgb(fg_code, is_fg=True) if fg_code else (128, 128, 128)
            bg_rgb = ansi_to_rgb(bg_code, is_fg=False) if bg_code else None

            cell_x = int(x_pos * cell_width)
            cell_y = int(y * cell_height)

            # Draw background if set
            if bg_rgb:
                draw.rectangle(
                    [cell_x, cell_y, cell_x + int(cell_width), cell_y + cell_height],
                    fill=bg_rgb,
                )

            # Draw character
            draw.text((cell_x, cell_y), char, font=font, fill=fg_rgb)

            x_pos += 1

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
