#!/bin/bash 

PATH_DIR="/Users/aaronzh/Documents/GitHub/opencode"

read -r -d '' PROMPT <<EOF
对比当前rust实现与代码仓： $PATH_DIR 全面补齐功能差异，达成功能完全一致。过程中不要问我问题，不要中断，不要等待我输入。
EOF

for i in {1..10}; do
  echo "Run #$i"
  opencode run -m opencode/minimax-m2.5-free "$PROMPT"
  # opencode run -m opencode/mimo-v2-pro-free "$PROMPT" 
done