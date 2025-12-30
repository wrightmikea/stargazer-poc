#!/usr/bin/env python3

import json
import re
from collections import defaultdict

SVG_WIDTH = 2010.7097
SVG_HEIGHT = 1122.5203

# Read file
with open('data/stars.svg', 'r') as f:
    lines = f.readlines()

# Find all unique star names by searching for labels
# Look for patterns like "pathNUM CONST- NAME" or just "NAME"
star_pattern = re.compile(r'inkscape:label="[^"]*-\s+([A-Z][a-z]{3,})"')
found_labels = defaultdict(set)

print("Scanning for star names...")
for i, line in enumerate(lines):
    matches = star_pattern.findall(line)
    for name in matches:
        # Skip grid labels and other non-stars
        if name in ['Grids', 'Source', 'Legend', 'Border', 'Axis', 'Ecliptic', 'Equator']:
            continue
        # Skip names with numbers (e.g., "15 LMi")
        if any(c.isdigit() for c in name):
            continue
        found_labels[name].add(i)

print(f"Found {len(found_labels)} unique star names")

# Now extract coordinates for each star
stars = {}
id_counter = 0

for name, line_indices in sorted(found_labels.items()):
    # Use first occurrence
    line_idx = min(line_indices)
    
    # Search backward for path element
    j = line_idx
    path_found = False
    while j >= 0 and j > line_idx - 10:
        if '    <path' in lines[j]:
            # Found path, now search forward for d attribute
            k = j
            while k <= line_idx:
                if ' d="' in lines[k]:
                    # Extract coordinates
                    d_match = re.search(r' d="m ([\d.]+),([\d.]+)', lines[k])
                    if d_match:
                        x = float(d_match.group(1))
                        y = float(d_match.group(2))
                        
                        # Find scale (usually in the label line or nearby)
                        scale_match = re.search(r'transform="scale\(([\d.]+)\)"', lines[line_idx])
                        scale = float(scale_match.group(1)) if scale_match else 1.0
                        
                        # Apply scale
                        scaled_x = x * scale
                        scaled_y = y * scale
                        
                        # Convert to RA/Dec
                        ra = round((scaled_x / SVG_WIDTH) * 24.0, 4)
                        dec = round(((SVG_HEIGHT / 2.0) - scaled_y) / (SVG_HEIGHT / 2.0) * 90.0, 4)
                        
                        # Estimate magnitude (simplified - could be improved)
                        magnitude = 2.0
                        
                        # Determine tile (for segmentation)
                        tile_x = int(scaled_x / (SVG_WIDTH / 4))  # 4 columns
                        tile_y = int(scaled_y / (SVG_HEIGHT / 3))  # 3 rows
                        tile_id = tile_y * 4 + tile_x
                        
                        stars[name] = {
                            'id': id_counter,
                            'name': name,
                            'ra': ra,
                            'dec': dec,
                            'magnitude': magnitude,
                            'tile_id': tile_id,
                            'tile_coords': {'x': tile_x, 'y': tile_y},
                            'svg_pos': {'x': round(scaled_x, 2), 'y': round(scaled_y, 2)}
                        }
                        
                        id_counter += 1
                        path_found = True
                        break
                k += 1
            if path_found:
                break
        j -= 1

print(f"Extracted {len(stars)} stars with coordinates")

# Save all stars
star_list = list(stars.values())
star_list.sort(key=lambda x: x['name'])

with open('data/stars.json', 'w') as f:
    json.dump(star_list, f, separators=(',', ':'))

print(f"Saved {len(star_list)} stars to data/stars.json")

# Save with pretty formatting
with open('data/stars_pretty.json', 'w') as f:
    json.dump(star_list, f, indent=2)

print("Saved to data/stars_pretty.json")

# Stats by tile
tile_counts = defaultdict(int)
for star in star_list:
    tile_counts[star['tile_id']] += 1

print("\nStars per tile (3 rows x 4 cols):")
for tile in range(12):
    row, col = divmod(tile, 4)
    print(f"  Tile {tile:2d} (row {row}, col {col}): {tile_counts[tile]:3d} stars")
