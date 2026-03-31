## ADDED Requirements

### Requirement: ReviewAgent provides code review functionality
The ReviewAgent SHALL analyze code changes and provide AI-powered review feedback including issues, suggestions, and improvement recommendations.

#### Scenario: Review code changes via git diff
- **WHEN** user requests code review with git diff content
- **THEN** ReviewAgent SHALL analyze the diff and return structured review feedback including:
  - Critical issues found
  - Warnings and suggestions
  - Code quality improvements
  - Security considerations

#### Scenario: Review specific file
- **WHEN** user requests review of a specific file
- **THEN** ReviewAgent SHALL read the file content and provide:
  - Overall code quality assessment
  - Specific issues and line numbers
  - Refactoring suggestions

#### Scenario: Review with context
- **WHEN** user requests review with additional context (requirements, constraints)
- **THEN** ReviewAgent SHALL incorporate the provided context into its review analysis

### Requirement: ReviewAgent integrates with permission system
The ReviewAgent SHALL respect the permission system for file access during reviews.

#### Scenario: Permission denied for restricted files
- **WHEN** ReviewAgent attempts to read a file outside permission scope
- **THEN** ReviewAgent SHALL return a permission denied error with clear message

### Requirement: ReviewAgent supports multiple review types
The ReviewAgent SHALL support different types of code review.

#### Scenario: Security-focused review
- **WHEN** user requests security-focused review
- **THEN** ReviewAgent SHALL prioritize security vulnerabilities and best practices

#### Scenario: Performance-focused review
- **WHEN** user requests performance-focused review
- **THEN** ReviewAgent SHALL analyze code for performance issues and optimization opportunities
