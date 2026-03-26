#!/bin/bash 

PATH_DIR="/Users/aaronzh/Documents/GitHub/opencode"

read -r -d '' PROMPT <<EOF
对比当前rust实现与代码仓： $PATH_DIR 全面补齐功能差异，达成功能完全一致。
EOF

for i in {1..10}; do
  echo "Run #$i"
  # opencode run -m opencode/glm-4.7-free "$PROMPT"
  opencode run -m opencode/mimo-v2-pro-free "$PROMPT" 
done