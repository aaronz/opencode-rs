## ADDED Requirements

### Requirement: command-spellcheck
Check spelling and grammar of markdown file changes.

#### Scenario: Spellcheck markdown changes
- **WHEN** user invokes the spellcheck command
- **THEN** system looks at all unstaged changes to markdown (.md, .mdx) files
- **AND** pulls out the lines that have changed
- **AND** checks for spelling and grammar errors
