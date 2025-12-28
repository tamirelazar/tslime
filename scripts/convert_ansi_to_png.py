#!/usr/bin/env python3
"""Simple ANSI text to PNG converter for tslime frames."""

import sys
import os
import re
from PIL import Image, ImageDraw, ImageFont


def ansi256_to_rgb(index):
    """Convert 256-color ANSI index to RGB."""
    if index < 16:
        color_map = [
            (0, 0, 0), (128, 0, 0), (0, 128, 0), (128, 128, 0),
            (0, 0, 128), (128, 0, 128), (0, 128, 128), (192, 192, 192),
            (128, 128, 128), (255, 0, 0), (0, 255, 0), (255, 255, 0),
            (0, 0, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255)
        ]
        return color_map[index]
    elif index < 232:
        index -= 16
        r = ((index // 36) * 51)
        g = (((index % 36) // 6) * 51)
        b = ((index % 6) * 51)
        return (r, g, b)
    else:
        index -= 232
        gray = (index * 10) + 8
        return (gray, gray, gray)


def parse_ansi_line(line, default_color=(0, 255, 0)):
    """Parse a line of ANSI text into (text, color) pairs."""
    ansi_re = re.compile(r'\x1b\[(\d+)(;\d+)?m')
    
    result = []
    current_color = default_color
    
    parts = ansi_re.split(line)
    for i, part in enumerate(parts):
        if i % 2 == 0:
            result.append({'text': part, 'color': current_color})
        else:
            match = ansi_re.search(line)
            if match:
                color_code = int(match.group(1))
                if 30 <= color_code <= 37:
                    current_color = (128, 128, 128)
                elif 38 and ';' in match.group(0) and '5' in match.group(2):
                    parts2 = match.group(0).split(';')
                    if len(parts2) >= 3:
                        idx = int(parts2[2])
                        current_color = ansi256_to_rgb(idx)
                elif match.group(0) == '\x1b[0m':
                    current_color = default_color
    
    return result


def convert_frame_to_png(input_file, output_file, font_size=14, line_height_factor=1.2):
    """Convert an ANSI text file to PNG image."""
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

        parsed = parse_ansi_line(line)
        x_pos = 0
        
        for segment in parsed:
            text = segment.get('text', '')
            color = segment.get('color', (128, 128, 128))
            
            if text:
                for char in text:
                    if x_pos < 80:
                        draw.text(
                            (x_pos * cell_width, y * cell_height),
                            char,
                            font=font,
                            fill=color
                        )
                        x_pos += 1

    img.save(output_file, 'PNG')
    print(f"Converted {input_file} -> {output_file}")


def main():
    if len(sys.argv) < 3:
        print("Usage: python3 convert_ansi_to_png.py <input_file> <output_file>", file=sys.stderr)
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    if not os.path.exists(input_file):
        print(f"Error: Input file '{input_file}' not found", file=sys.stderr)
        sys.exit(1)

    convert_frame_to_png(input_file, output_file)


if __name__ == '__main__':
    main()
