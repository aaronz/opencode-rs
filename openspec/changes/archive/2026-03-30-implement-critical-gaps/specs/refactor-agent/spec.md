## ADDED Requirements

### Requirement: RefactorAgent provides code refactoring functionality
The RefactorAgent SHALL analyze code and provide intelligent refactoring suggestions with the ability to apply them.

#### Scenario: Analyze code for refactoring opportunities
- **WHEN** user requests refactoring analysis
- **THEN** RefactorAgent SHALL:
  - Identify code smells and anti-patterns
  - Suggest specific refactoring techniques
  - Estimate impact of each refactoring
  - Provide before/after code examples

#### Scenario: Apply refactoring automatically
- **WHEN** user approves a refactoring suggestion
- **THEN** RefactorAgent SHALL apply the refactoring to the codebase using write/edit tools

#### Scenario: Preview refactoring changes
- **WHEN** user requests preview of refactoring
- **THEN** RefactorAgent SHALL show diff of proposed changes without applying them

### Requirement: RefactorAgent handles multiple refactoring types
The RefactorAgent SHALL support common refactoring operations.

#### Scenario: Extract method refactoring
- **WHEN** user requests extraction of a code block to a new method
- **THEN** RefactorAgent SHALL:
  - Identify the code block
  - Create a new method with appropriate signature
  - Replace original code with method call

#### Scenario: Rename refactoring
- **WHEN** user requests renaming a symbol (function, variable, class)
- **THEN** RefactorAgent SHALL:
  - Find all references to the symbol
  - Rename consistently across the codebase
  - Handle scoping correctly

### Requirement: RefactorAgent validates changes
The RefactorAgent SHALL validate changes after applying refactoring.

#### Scenario: Verify refactoring correctness
- **WHEN** refactoring is applied
- **THEN** RefactorAgent SHALL:
  - Run build/compilation to verify correctness
  - Run relevant tests if available
  - Report success or rollback if errors occur
