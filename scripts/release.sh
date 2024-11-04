#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

VERSION="$1"

declare -A patterns=(
    ["snforge_std = { git = \"https://github.com/foundry-rs/starknet-foundry.git\", tag = \"<<version>>\" }"]="snforge_std = { git = \"https://github.com/foundry-rs/starknet-foundry.git\", tag = \"v${VERSION}\" }"
    ["snforge_std = <<version>>"]="snforge_std = \"${VERSION}\""
    ["sncast_std = { git = \"https://github.com/foundry-rs/starknet-foundry.git\", tag = \"<<version>>\" }"]="sncast_std = { git = \"https://github.com/foundry-rs/starknet-foundry.git\", tag = \"v${VERSION}\" }"
    ["sncast_std = <<version>>"]="sncast_std = \"${VERSION}\""
)

doc_files=$(find ./docs -name "*.md")

for file in $doc_files; do
    echo "Processing $file..."
    
    temp_file=$(mktemp)
    
    cp "$file" "$temp_file"
    
    for pattern in "${!patterns[@]}"; do
        replacement="${patterns[$pattern]}"
        # Escape special characters in pattern and replacement for sed
        escaped_pattern=$(echo "$pattern" | sed 's/[\/&]/\\&/g')
        escaped_replacement=$(echo "$replacement" | sed 's/[\/&]/\\&/g')
        
        sed -i '' "s/$escaped_pattern/$escaped_replacement/g" "$temp_file"

    done
    
    if ! cmp -s "$file" "$temp_file"; then
        mv "$temp_file" "$file"
        echo "Updated $file"
    else
        rm "$temp_file"
        echo "No changes needed in $file"
    fi
done

echo "Version update complete!"