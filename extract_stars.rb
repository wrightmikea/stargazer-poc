#!/usr/bin/env ruby

require 'json'

# SVG dimensions
SVG_WIDTH = 2010.7097
SVG_HEIGHT = 1122.5203

stars = []
id_counter = 0

# Read entire file and process complete path elements
content = File.read('data/stars.svg')

# Split by <path tags
content.scan(/<path[^>]*inkscape:label="[^"]*"[^>]*\/>/) do |path_elem|
  # Extract label
  label_match = path_elem.match(/inkscape:label="([^"]+)"/)
  next unless label_match
  label = label_match[1]
  
  # Parse label format
  if label =~ /- ([A-Z][a-z]{3,})$/
    name = $1
  elsif label =~ /([A-Z][a-z]{3,})$/
    name = $1
  else
    next
  end
  
  # Skip invalid names
  next if name.length < 3
  next if name =~ /[0-9]/
  
  # Extract path data
  d_match = path_elem.match(/ d="([^"]+)"/)
  next unless d_match
  path_data = d_match[1]
  
  next unless path_data.start_with?('m')
  
  coords_match = path_data[1..].match(/^(\d+\.\d+),(\d+\.\d+)/)
  next unless coords_match
  x = coords_match[1].to_f
  y = coords_match[2].to_f
  
  # Extract scale
  scale_match = path_elem.match(/transform="scale\(([^)]+)\)"/)
  scale = scale_match ? scale_match[1].to_f : 1.0
  
  # Apply scale
  scaled_x = x * scale
  scaled_y = y * scale
  
  # Convert to RA/Dec
  ra = (scaled_x / SVG_WIDTH) * 24.0
  dec = ((SVG_HEIGHT / 2.0) - scaled_y) / (SVG_HEIGHT / 2.0) * 90.0
  
  stars << {
    id: id_counter,
    ra: ra.round(4),
    dec: dec.round(4),
    magnitude: 2.0,
    name: name
  }
  
  id_counter += 1
end

puts "Extracted #{stars.length} named stars"

# Output as compact JSON
File.open('data/stars.json', 'w') do |f|
  f.write(JSON.generate(stars))
end

puts "Saved to data/stars.json"

# Also save human-readable version
File.open('data/stars_pretty.json', 'w') do |f|
  f.write(JSON.pretty_generate(stars))
end

puts "Saved pretty version to data/stars_pretty.json"
