#!/usr/bin/env python3
"""Simple text to PNG converter for tslime frames (no ANSI codes)."""

import sys
import os
from PIL import Image, ImageDraw, ImageFont


def convert_frame_to_png(input_file, output_file, font_size=14, line_height_factor=1.2):
    """Convert a plain text file to PNG image."""
    with open(input_file, 'r', encoding='utf-8', errors='ignore') as f:
        content = f.read()

    lines = content.split('\n')
    
    cell_width = font_size * 0.5
    cell_height = int(font_size * 1.0)
    img_width = int(80 * cell_width)
    img_height = int(24 * cell_height)

    img = Image.new('RGB', (img_width, img_height), color=(0, 0, 0))
    draw = ImageDraw.Draw(img)

    try:
        font = ImageFont.truetype('/System/Library/Fonts/Menlo.ttc', font_size)
    except:
        try:
            font = ImageFont.truetype('/System/Library/Fonts/Monaco.dfont', font_size)
        except:
            font = ImageFont.load_default()

    for y, line in enumerate(lines[:24]):
        if not line.strip():
            continue

        x_pos = 0
        for char in line:
            if x_pos < 80:
                draw.text(
                    (x_pos * cell_width, y * cell_height),
                    char,
                    font=font,
                    fill=(128, 128, 128)
                )
                x_pos += 1

    img.save(output_file, 'PNG')
    print(f"Converted {input_file} -> {output_file}")


def main():
    if len(sys.argv) < 3:
        print("Usage: python3 convert_text_to_png.py <input_file> <output_file>", file=sys.stderr)
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    if not os.path.exists(input_file):
        print(f"Error: Input file '{input_file}' not found", file=sys.stderr)
        sys.exit(1)

    convert_frame_to_png(input_file, output_file)


if __name__ == '__main__':
    main()
