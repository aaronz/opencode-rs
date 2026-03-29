## ADDED Requirements

### Requirement: SIGINT Interruption
The system SHALL gracefully handle Ctrl+C during LLM generation.

#### Scenario: Generation Interruption
- **WHEN** user presses `Ctrl+C` during LLM generation
- **THEN** system stops the LLM request and displays "[Interrupted]" marker

#### Scenario: Interrupt Recovery
- **WHEN** generation is interrupted
- **THEN** system returns control to the input field immediately

### Requirement: Interrupt State Management
The system SHALL manage interrupt state properly.

#### Scenario: Request Cancellation
- **WHEN** interrupt is triggered
- **THEN** system cancels the pending HTTP request to LLM

#### Scenario: Partial Response Handling
- **WHEN** generation is interrupted mid-stream
- **THEN** system preserves partial response received so far

### Requirement: Interrupt Visual Feedback
The system SHALL provide clear visual feedback for interruptions.

#### Scenario: Interrupt Indicator
- **WHEN** generation is interrupted
- **THEN** system shows "[Interrupted]" in muted color at the end of partial response
