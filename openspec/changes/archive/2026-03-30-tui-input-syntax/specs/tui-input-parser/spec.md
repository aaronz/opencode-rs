## ADDED Requirements

### Requirement: Input Parser Detects Prefix Types
The input parser SHALL detect the prefix character at the start of user input and classify it into the appropriate input type.

#### Scenario: Plain text input
- **WHEN** user enters text that does not start with `@`, `!`, or `/`
- **THEN** parser SHALL classify it as `Plain` input type

#### Scenario: File reference input
- **WHEN** user enters text that starts with `@`
- **THEN** parser SHALL classify it as `FileRef` input type

#### Scenario: Shell command input
- **WHEN** user enters text that starts with `!`
- **THEN** parser SHALL classify it as `Shell` input type

#### Scenario: Command input
- **WHEN** user enters text that starts with `/`
- **THEN** parser SHALL classify it as `Command` input type

#### Scenario: Whitespace before prefix
- **WHEN** user enters text with leading whitespace before prefix
- **THEN** parser SHALL ignore whitespace and detect prefix

### Requirement: Input Parser Returns Structured Data
The parser SHALL return structured data including the input type, original raw input, and parsed content.

#### Scenario: Parse plain text
- **WHEN** parser receives plain text "Hello world"
- **THEN** it SHALL return `{ type: "Plain", raw: "Hello world", content: "Hello world" }`

#### Scenario: Parse file reference
- **WHEN** parser receives "@src/main.rs"
- **THEN** it SHALL return `{ type: "FileRef", raw: "@src/main.rs", content: "src/main.rs" }`

#### Scenario: Parse shell command
- **WHEN** parser receives "!ls -la"
- **THEN** it SHALL return `{ type: "Shell", raw: "!ls -la", content: "ls -la" }`

#### Scenario: Parse slash command
- **WHEN** parser receives "/help"
- **THEN** it SHALL return `{ type: "Command", raw: "/help", content: "help" }`

#### Scenario: Parse slash command with argument
- **WHEN** parser receives "/model gpt-4"
- **THEN** it SHALL return `{ type: "Command", raw: "/model gpt-4", command: "model", args: ["gpt-4"] }`

### Requirement: Parser Handles Edge Cases
The parser SHALL handle edge cases gracefully without panicking.

#### Scenario: Empty input
- **WHEN** parser receives empty string
- **THEN** it SHALL return `{ type: "Plain", raw: "", content: "" }`

#### Scenario: Only prefix with no content
- **WHEN** parser receives "@" (just the prefix)
- **THEN** it SHALL return `{ type: "FileRef", raw: "@", content: "" }`

#### Scenario: Prefix in middle of text
- **WHEN** parser receives "Hello @world"
- **THEN** it SHALL classify as Plain (prefix only detected at start)
