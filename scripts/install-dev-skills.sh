#!/bin/bash

SOURCE_DIR="/Users/aaronzh/.cache/opencode/node_modules/superpowers/skills"
DEST_DIR="/Users/aaronzh/Documents/GitHub/mycode/.opencode/skills"

if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: Source directory not found: $SOURCE_DIR"
    exit 1
fi

mkdir -p "$DEST_DIR"

for skill in "$SOURCE_DIR"/*; do
    if [ -d "$skill" ]; then
        skill_name=$(basename "$skill")
        echo "Installing $skill_name..."
        rm -rf "$DEST_DIR/$skill_name"
        cp -r "$skill" "$DEST_DIR/"
    fi
done

echo ""
echo "Done! Installed skills:"
ls -la "$DEST_DIR"
