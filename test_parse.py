#!/usr/bin/env python3
import re

label = "path1024 Ori- Betelgeuse"

patterns = [
    (r'path\d+\s+[A-Z]{3}\s*-\s+', "Basic pattern"),
    (r'path\d+\s+[A-Z]{3}\s+-\s+', "No space after dash"),
    (r'path\d+\s+[A-Z]{3}-\s+', "No spaces around dash"),
]

for pattern, desc in patterns:
    match = re.match(pattern, label)
    print(f"{desc}: {'MATCH' if match else 'NO MATCH'}")

# Try extracting name
name_pattern = r'-\s+([A-Z][a-z]{3,})$'
name_match = re.search(name_pattern, label)
print(f"\nName extraction: {name_match.group(1) if name_match else 'NO MATCH'}")
