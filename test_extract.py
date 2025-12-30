#!/usr/bin/env python3

import re

with open('data/stars.svg', 'r') as f:
    content = f.read()

# Test different patterns
patterns = [
    r'inkscape:label="path\d+\s+[A-Z]{3}-\s+([A-Z][a-z]{3,})"',
    r'inkscape:label="path\d+\s+[A-Z]{3}-([A-Z][a-z]{3,})"',
    r'inkscape:label="[A-Z]{3}-([A-Z][a-z]{3,})"',
    r'inkscape:label="[^"]*([A-Z][a-z]{4,})"',
]

for i, pattern in enumerate(patterns):
    matches = re.findall(pattern, content)
    print(f"Pattern {i}: {len(matches)} matches")
    if matches:
        print(f"  Sample: {matches[:5]}")
        break
