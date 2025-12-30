#!/bin/bash

# Extract path elements with inkscape:label containing star names
grep -E '<path.*inkscape:label="[^"]*- [A-Z][a-z]{3,}"' data/stars.svg | \
while IFS= read -r line; do
    # Extract label
    label=$(echo "$line" | sed -n 's/.*inkscape:label="\([^"]*\)".*/\1/p')
    
    # Extract name (after last dash, if present)
    if echo "$label" | grep -q -- '- '; then
        name=$(echo "$label" | sed 's/.*- //')
    else
        name=$(echo "$label" | awk '{print $1}')
    fi
    
    # Validate name (letters only, at least 3 chars)
    if echo "$name" | grep -qE '^[A-Z][a-z]{2,}$'; then
        # Extract path data
        path_data=$(echo "$line" | sed -n 's/.* d="\([^"]*\)".*/\1/p')
        
        if echo "$path_data" | grep -qE '^m [0-9]'; then
            # Extract coordinates
            x=$(echo "$path_data" | sed -n 's/^m \([0-9.]*\),.*/\1/p')
            y=$(echo "$path_data" | sed -n 's/^m [0-9.]*,\([0-9.]*\).*/\1/p')
            
            # Extract scale
            scale=$(echo "$line" | sed -n 's/.*transform="scale(\([^)]*\))".*/\1/p')
            scale=${scale:-1.0}
            
            # Apply scale
            scaled_x=$(echo "$x * $scale" | bc)
            scaled_y=$(echo "$y * $scale" | bc)
            
            # Convert to RA/Dec
            SVG_WIDTH=2010.7097
            SVG_HEIGHT=1122.5203
            ra=$(echo "scale=4; $scaled_x / $SVG_WIDTH * 24" | bc)
            dec=$(echo "scale=4; ($SVG_HEIGHT / 2 - $scaled_y) / ($SVG_HEIGHT / 2) * 90" | bc)
            
            echo "{\"id\":$id_counter,\"ra\":$ra,\"dec\":$dec,\"magnitude\":2.0,\"name\":\"$name\"}"
            ((id_counter++))
        fi
    fi
done | jq -s '.'
