1. OpenCode-RS startup slow: startup TUI of opencode-RS cost more than 1mins. Also need to check whether the logging infra are enough to debug such kind of issues.

2. /exit command does not clear the screen, the tui layout is left on the terminal.

3. the configuration path, installation path should all use opencode-rs specific path, should not conflict with opencode. i see the install.sh has configuration path issues. need to review the path design to see any similar issues.